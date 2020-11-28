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

use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use log::debug;
use log::warn;

pub struct Slack {
    webhook_url: String,

    spooler: Receiver<Message>,

    send_reporter: Sender<Option<Message>>,

    terminator: tokio::sync::broadcast::Receiver<()>,
}

#[derive(Debug)]
pub enum Error {
    ReqwestError(reqwest::Error),

    StatusCode(u16),
}

impl Slack {
    pub fn new(
        spooler: Receiver<Message>,
        send_reporter: Sender<Option<Message>>,
        webhook_url: String,
        terminator: tokio::sync::broadcast::Receiver<()>,
    ) -> Self {
        Slack {
            webhook_url,
            spooler,
            send_reporter,
            terminator,
        }
    }

    pub async fn send_messages(&mut self) {
        loop {
            tokio::select! {
                next = self.spooler.recv() => {
                   if let Some(message) = next {
                        debug!("Sending message");
                        if self.send_message(&message).await.is_err() {
                            if self.send_reporter.send(Some(message)).await.is_err() {
                                debug!("Slack shutting down");
                                return;
                            }
                        } else if self.send_reporter.send(None).await.is_err() {
                            debug!("Slack shutting down");
                            return;
                        }
                    } else {
                        debug!("Slack shutting down");
                        return;
                    }
                }
                _ = self.terminator.recv() => {
                    debug!("Slack shutting down");
                    return;
                }
            }
        }
    }

    async fn send_message(&self, message: &Message) -> Result<(), ()> {
        let client = reqwest::Client::builder().build().map_err(|_| ())?;

        let response = client.post(&self.webhook_url).json(message).send().await;

        match response {
            Ok(r) => match r.status().as_u16() {
                200 => Ok(()),
                _ => {
                    warn!("Upstream reported error: {:#?}", r);
                    Err(())
                }
            },
            Err(e) => {
                warn!("Error while sending: {}", e);
                Err(())
            }
        }
    }
}
