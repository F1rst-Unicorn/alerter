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

use crate::message::Message;

use nix::errno;
use nix::sys::stat;

use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;
use tokio::stream::StreamExt;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;

use std::ffi::CString;
use std::fs::remove_file;
use std::os::unix::io::AsRawFd;

use log::debug;
use log::error;
use log::warn;

pub struct Listener {
    socket_path: String,

    listener: Option<tokio::net::UnixListener>,

    slack: Sender<Message>,

    terminator: Receiver<()>,
}

#[derive(Debug)]
pub enum Error {
    StdIoError(std::io::Error),
    NixError(nix::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::StdIoError(e)
    }
}

impl From<nix::Error> for Error {
    fn from(e: nix::Error) -> Self {
        Error::NixError(e)
    }
}

impl Listener {
    pub fn new(socket_path: &str, slack: Sender<Message>, terminator: Receiver<()>) -> Self {
        Self {
            socket_path: socket_path.to_string(),
            listener: None,
            slack,
            terminator,
        }
    }

    pub fn start(mut self) -> Result<Self, Error> {
        self.listener = Some(Self::build_listener(&self.socket_path)?);
        Ok(self)
    }

    pub async fn handle_new_messages(&mut self) {
        loop {
            tokio::select! {
                next = self.listener.as_mut().unwrap().next() => {
                    match next {
                        None => {
                            debug!("Listener shutting down");
                            return;
                        }
                        Some(Ok(mut stream)) => {

                            let mut string = String::new();
                            if let Err(e) = stream.read_to_string(&mut string).await {
                                error!("Failed to read from socket: {}", e);
                                continue;
                            }

                            if let Err(e) = self.transmit_message(string).await {
                                error!("Failed to transmit message: {:#?}", e);
                                continue;
                            }
                        }
                        Some(Err(e)) => {
                            error!("Failed to get stream: {}", e);
                        }
                    }
                }
                _ = self.terminator.recv() => {
                    debug!("Listener shutting down");
                    return;
                }
            }
        }
    }

    async fn transmit_message(&mut self, message: String) -> Result<(), Error> {
        let message: Result<Message, serde_json::error::Error> = serde_json::from_str(&message);
        if let Err(e) = message {
            warn!("Could not read request: {}", e);
            return Ok(());
        }
        let message = message.unwrap();

        if let Err(e) = self.slack.send(message).await {
            warn!("Could not send message: {:#?}", e);
        }
        Ok(())
    }

    fn build_listener(socket_path: &str) -> Result<UnixListener, Error> {
        debug!("Setting up socket");
        remove_file(socket_path)?;

        let listener = UnixListener::bind(socket_path)?;

        let mut flags = stat::Mode::empty();
        flags.insert(stat::Mode::S_IRWXU);
        flags.insert(stat::Mode::S_IRWXG);
        flags.insert(stat::Mode::S_IRWXO);
        stat::fchmod(listener.as_raw_fd(), flags)?;

        unsafe {
            let raw_path = CString::new(socket_path).expect("could not build cstring");
            let res = libc::chmod(raw_path.into_raw(), 0o777);
            if res == -1 {
                return Err(nix::Error::Sys(errno::Errno::from_i32(errno::errno())).into());
            }
        }

        debug!("Input uds open");
        Ok(listener)
    }
}