/*  alerter: Alerter to Slack
 *  Copyright (C) 2019 The alerter developers
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use nix::errno;
use nix::sys::epoll;
use nix::sys::signal;
use nix::sys::signalfd;
use nix::sys::socket;
use nix::sys::stat;

use systemd::daemon::notify;

use std::ffi::CString;
use std::fs::remove_file;
use std::io::Error;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;
use std::os::unix::io::RawFd;
use std::process::exit;
use std::convert::TryFrom;

use log::{debug, error, info, warn};

use crate::backoff::Backoff;
use crate::message::Message;
use crate::slack::Slack;
use crate::spooler::Spooler;

const EXIT_CODE: i32 = 2;

pub struct Daemon {
    spooler: Spooler,

    backoff: Backoff,

    socket_fd: RawFd,

    epoll_fd: RawFd,

    signal_fd: signalfd::SignalFd,

    slack: Slack,

    keep_running: bool,
}

impl Daemon {
    pub fn new(socket_path: &str, spool_path: &str, slack: Slack) -> Self {
        let signal_fd = Daemon::setup_signal_handler();
        if let Err(e) = signal_fd {
            error!("Could not setup signals: {}", e);
            exit(EXIT_CODE);
        }
        let signal_fd = signal_fd.unwrap();

        let socket_fd = Daemon::setup_socket(socket_path);
        if let Err(e) = socket_fd {
            error!("Could not setup socket: {}", e);
            exit(EXIT_CODE);
        }
        let socket_fd = socket_fd.unwrap();

        let epoll_fd = Daemon::setup_epoll_fd(signal_fd.as_raw_fd(), socket_fd);
        if let Err(e) = epoll_fd {
            error!("Could not setup epoll: {}", e);
            exit(EXIT_CODE);
        }
        let epoll_fd = epoll_fd.unwrap();

        let spooler = Spooler::new(spool_path);

        Daemon {
            spooler,
            backoff: Backoff::new(),
            socket_fd,
            epoll_fd,
            signal_fd,
            slack,
            keep_running: true,
        }
    }

    pub fn run(&mut self) {
        info!("Ready for connections");
        notify_systemd(&[("READY", "1")]);
        while self.keep_running {
            self.dispatch_epoll();
            self.flush_spooler();
        }
        info!("Shutting down");
    }

    fn dispatch_epoll(&mut self) {
        let mut event_buffer = [epoll::EpollEvent::empty(); 10];
        let epoll_result = epoll::epoll_wait(self.epoll_fd, &mut event_buffer, 1000);
        match epoll_result {
            Ok(0) => {}
            Ok(count) => {
                debug!("epoll got {} events", count);
                for event in event_buffer.iter().take(count) {
                    self.handle_event(*event);
                }
            }
            Err(error) => {
                error!("Could not complete epoll: {:#?}", error);
            }
        }
    }

    fn handle_event(&mut self, event: epoll::EpollEvent) {
        if event.events().contains(epoll::EpollFlags::EPOLLIN) {
            let fd = event.data() as RawFd;
            if fd == self.signal_fd.as_raw_fd() {
                self.handle_signal();
            } else if fd == self.socket_fd {
                self.handle_request();
            } else {
                warn!("Unknown fd: {}", fd);
            }
        } else {
            warn!("Received unknown event");
        }
    }

    fn handle_request(&mut self) {
        if let Err(e) = self.handle_request_internally() {
            warn!("Failed to handle request: {:#?}", e);
        }
    }

    fn handle_request_internally(&mut self) -> Result<(), nix::Error> {
        let file = unsafe { std::fs::File::from_raw_fd(socket::accept(self.socket_fd)?) };

        let message: Result<Message, serde_json::error::Error> = serde_json::from_reader(file);
        if let Err(e) = message {
            warn!("Could  not read request: {}", e);
            return Ok(());
        }
        let message = message.unwrap();

        if let Err(e) = self.slack.send(&message) {
            warn!("Could not send message, queuing. Reason: {:#?}", e);
            self.spooler.queue(message);
            self.spooler.store();
        }
        Ok(())
    }

    fn flush_spooler(&mut self) {
        let mut sent_messages = false;

        if !self.spooler.is_empty() {
            if self.backoff.is_ready() {
                while let Some(m) = self.spooler.pop_message() {
                    warn!("Retransmitting message");
                    if let Err(e) = self.slack.send(&m) {
                        warn!(
                            "Could not send queued message, queuing again. Reason: {:#?}",
                            e
                        );
                        self.spooler.queue_front(m);
                        self.backoff.backoff();
                        break;
                    } else {
                        info!("Queued message released");
                        sent_messages = true;
                        self.backoff.reset();
                    }
                }
            } else {
                self.backoff.inc();
            }
        }

        if sent_messages {
            self.spooler.store();
        }
    }

    fn handle_signal(&mut self) {
        match self.signal_fd.read_signal() {
            Ok(Some(signal)) => {
                match signal::Signal::try_from(signal.ssi_signo as i32).unwrap() {
                    signal::SIGINT | signal::SIGQUIT | signal::SIGTERM => {
                        self.initiate_shutdown();
                    }
                    other => {
                        debug!("Received unknown signal: {:?}", other);
                    }
                }
            }
            Ok(None) => {
                debug!("No signal received");
            }
            Err(other) => {
                debug!("Received unknown signal: {:?}", other);
            }
        }
    }

    fn initiate_shutdown(&mut self) {
        info!("Received termination signal");
        self.keep_running = false;
        notify_systemd(&[("STOPPING", "1")]);
    }

    fn setup_signal_handler() -> Result<signalfd::SignalFd, nix::Error> {
        debug!("Setting up signal handler");
        let mut signals = signalfd::SigSet::empty();
        signals.add(signalfd::signal::SIGINT);
        signals.add(signalfd::signal::SIGTERM);
        signals.add(signalfd::signal::SIGQUIT);
        signal::sigprocmask(signal::SigmaskHow::SIG_BLOCK, Some(&signals), None)?;
        signalfd::SignalFd::with_flags(&signals, signalfd::SfdFlags::SFD_CLOEXEC)
    }

    fn setup_socket(socket_path: &str) -> Result<RawFd, nix::Error> {
        debug!("Setting up socket");
        match remove_file(socket_path).map_err(map_to_errno) {
            Err(nix::Error::Sys(nix::errno::Errno::ENOENT)) => Ok(()),
            e => e,
        }?;

        let listener = socket::socket(
            socket::AddressFamily::Unix,
            socket::SockType::Stream,
            socket::SockFlag::SOCK_CLOEXEC,
            None,
        )?;

        socket::bind(
            listener,
            &socket::SockAddr::Unix(socket::UnixAddr::new(socket_path)?),
        )?;

        let mut flags = stat::Mode::empty();
        flags.insert(stat::Mode::S_IRWXU);
        flags.insert(stat::Mode::S_IRWXG);
        flags.insert(stat::Mode::S_IRWXO);
        stat::fchmod(listener, flags)?;

        unsafe {
            let raw_path = CString::new(socket_path).expect("could not build cstring");
            let res = libc::chmod(raw_path.into_raw(), 0o777);
            if res == -1 {
                return Err(nix::Error::Sys(errno::Errno::from_i32(errno::errno())));
            }
        }

        socket::listen(listener, 0)?;

        debug!("Input uds open");
        Ok(listener)
    }

    fn setup_epoll_fd(signal_fd: RawFd, socket: RawFd) -> Result<RawFd, nix::Error> {
        debug!("Setting up epoll");
        let epoll_fd = epoll::epoll_create1(epoll::EpollCreateFlags::EPOLL_CLOEXEC)?;
        epoll::epoll_ctl(
            epoll_fd,
            epoll::EpollOp::EpollCtlAdd,
            signal_fd,
            &mut epoll::EpollEvent::new(epoll::EpollFlags::EPOLLIN, signal_fd as u64),
        )?;
        epoll::epoll_ctl(
            epoll_fd,
            epoll::EpollOp::EpollCtlAdd,
            socket,
            &mut epoll::EpollEvent::new(epoll::EpollFlags::EPOLLIN, socket as u64),
        )?;
        Ok(epoll_fd)
    }
}

fn notify_systemd(message: &[(&str, &str)]) {
    let result = notify(false, message.iter());
    match result {
        Ok(false) => warn!("systemd hasn't been notified"),
        Err(e) => error!("error notifying systemd: {}", e),
        _ => ()
    }
}

pub fn map_to_errno(error: Error) -> nix::Error {
    let raw_error = error.raw_os_error();
    std::mem::drop(error);
    match raw_error {
        Some(errno) => nix::Error::Sys(nix::errno::Errno::from_i32(errno)),
        _ => nix::Error::Sys(nix::errno::Errno::UnknownErrno),
    }
}
