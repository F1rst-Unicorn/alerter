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

use crate::backoff::Backoff;
use crate::message::Message;
use crate::spooler::Spooler;

use std::time::Duration;

use log::debug;

use tokio::select;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::time::interval;
use tokio::time::Interval;

pub struct SpoolDispatcher {
    spooler: Spooler,

    sender: Sender<Message>,

    receiver: Receiver<Option<Message>>,

    backoff: Backoff,

    terminator: tokio::sync::broadcast::Receiver<()>,
}

impl SpoolDispatcher {
    pub fn new(
        spooler: Spooler,
        sender: Sender<Message>,
        receiver: Receiver<Option<Message>>,
        terminator: tokio::sync::broadcast::Receiver<()>,
    ) -> Self {
        SpoolDispatcher {
            spooler,
            sender,
            receiver,
            backoff: Backoff::new(),
            terminator,
        }
    }

    pub async fn run(mut self) {
        self.spooler.load().await;

        loop {
            let mut ticker = self.setup_ticker().await;

            select! {
                _ = ticker.tick() => {
                    if let Some(message) = self.spooler.pop_message() {
                        if let (Err(_),_) = tokio::join!(
                            self.sender.send(message),
                            self.spooler.store()
                        ) {
                            debug!("Spool dispatcher shutting down");
                            return;
                        }
                    }
                }
                work = self.receiver.recv() => {
                    match work {
                        Some(Some(message)) => {
                            self.spooler.queue(message);
                            self.spooler.store().await;
                            self.backoff.backoff();
                        }
                        Some(None) => {
                            self.backoff.reset();
                        }
                        None => {
                            debug!("Spool dispatcher shutting down");
                            return;
                        }
                    }
                }
                _ = self.terminator.recv() => {
                    debug!("Spool dispatcher shutting down");
                    return;
                }
            }
        }
    }

    async fn setup_ticker(&self) -> Interval {
        let seconds = if self.spooler.is_empty() {
            86400
        } else {
            self.backoff.get_backoff()
        };

        debug!("ticker at {}s", seconds);

        let mut ticker = interval(Duration::from_secs(seconds));
        ticker.tick().await;
        ticker
    }
}
