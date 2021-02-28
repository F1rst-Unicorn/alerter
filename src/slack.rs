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

use std::collections::BTreeMap;

use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use log::debug;
use log::warn;

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct BackendMessage {
    pub channel: Option<String>,

    pub username: String,

    pub attachments: Vec<Attachment>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Attachment {
    pub color: String,

    pub title: String,

    pub title_link: Option<String>,

    pub text: String,

    pub fields: Vec<Field>,

    pub footer: String,

    pub ts: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Field {
    pub title: String,

    pub value: String,

    pub short: bool,
}

impl From<&Message> for BackendMessage {
    fn from(m: &Message) -> Self {
        let attachment = Attachment {
            title: m.title.to_string(),
            title_link: m.link.clone(),
            text: m.text.to_string(),
            color: m.level.clone().into(),
            footer: m.version.to_string(),
            ts: m.timestamp.timestamp(),
            fields: transform_fields(&m.fields),
        };

        Self {
            username: crate::util::hostname(),
            channel: m.channel.clone(),
            attachments: vec![attachment],
        }
    }
}

fn transform_fields(fields: &BTreeMap<String, String>) -> Vec<Field> {
    fields
        .iter()
        .map(|(k, v)| Field {
            title: k.to_string(),
            value: v.to_string(),
            short: true,
        })
        .collect()
}

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
        let backend_message = BackendMessage::from(message);

        let client = reqwest::Client::builder().build().map_err(|_| ())?;

        let response = client
            .post(&self.webhook_url)
            .json(&backend_message)
            .send()
            .await;

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
