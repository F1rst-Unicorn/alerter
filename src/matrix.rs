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

use std::convert::TryFrom;

use crate::message::Message;
use crate::util;

use matrix_sdk::events::room::message::MessageEventContent;
use matrix_sdk::events::AnyMessageEventContent;
use matrix_sdk::Client;
use matrix_sdk::ClientConfig;
use matrix_sdk::SyncSettings;

use thiserror::Error;

use url::Url;

use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use log::debug;
use log::warn;

pub struct Matrix {
    client: Client,

    username: String,

    password: String,

    channel: String,

    spooler: Receiver<Message>,

    send_reporter: Sender<Option<Message>>,

    terminator: tokio::sync::broadcast::Receiver<()>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid well-known entry for home server")]
    InvalidHomeServer,

    #[error("invalid user format")]
    InvalidUser,

    #[error("username or password wrong")]
    InvalidLogin,

    #[error("room name wrong")]
    InvalidRoom,

    #[error("matrix error")]
    Matrix(#[from] matrix_sdk::Error),
}

impl Matrix {
    pub fn new(
        user: &str,
        password: &str,
        channel: &str,
        spooler: Receiver<Message>,
        send_reporter: Sender<Option<Message>>,
        terminator: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<Self, Error> {
        let mut iter = user.splitn(2, ':');
        let username = iter.next().ok_or(Error::InvalidUser)?;
        let server = iter.next().ok_or(Error::InvalidUser)?;

        let homeserver_url = resolve_well_known(server);
        let homeserver_url = Url::parse(&homeserver_url).map_err(|_| Error::InvalidHomeServer)?;

        let config = ClientConfig::new().store_path("./");

        let client = Client::new_with_config(homeserver_url, config)?;

        Ok(Matrix {
            client,
            username: username.to_string(),
            password: password.to_string(),
            channel: channel.to_string(),
            spooler,
            send_reporter,
            terminator,
        })
    }

    pub async fn login(self) -> Result<Self, Error> {
        let device_id = util::hostname();
        if self
            .client
            .login(
                &self.username,
                &self.password,
                Some(&device_id),
                Some(&device_id),
            )
            .await
            .is_err()
        {
            return Err(Error::InvalidLogin);
        }

        Ok(self)
    }

    pub async fn run(&mut self) {
        let syncer = self.client.clone();
        tokio::spawn(async move { syncer.sync(SyncSettings::new()).await });

        loop {
            tokio::select! {
                next = self.spooler.recv() => {
                   if let Some(message) = next {
                        debug!("Sending message");
                        if self.send_message(&message).await.is_err() {
                            if self.send_reporter.send(Some(message)).await.is_err() {
                                debug!("Matrix shutting down");
                                return;
                            }
                        } else if self.send_reporter.send(None).await.is_err() {
                            debug!("Matrix shutting down");
                            return;
                        }
                    } else {
                        debug!("Matrix shutting down");
                        return;
                    }
                }
                _ = self.terminator.recv() => {
                    debug!("Matrix shutting down");
                    return;
                }
            }
        }
    }

    pub async fn send_message(&self, message: &Message) -> Result<(), ()> {
        let channel = message
            .channel
            .clone()
            .unwrap_or_else(|| self.channel.to_string());

        dbg!(&channel);

        let room = dbg!(TryFrom::try_from(channel)).map_err(|_| ())?;

        dbg!(&message);
        dbg!(&room);

        match self.client.room_send(&room, message, None).await {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Error while sending: {:#?}", e);
                Err(())
            }
        }
    }
}

impl From<&Message> for AnyMessageEventContent {
    fn from(_: &Message) -> Self {
        AnyMessageEventContent::RoomMessage(MessageEventContent::Text(
            matrix_sdk::events::room::message::TextMessageEventContent::html("hello", "world"),
        ))
    }
}

pub fn resolve_well_known(matrix_id: &str) -> String {
    matrix_id.to_string()
}
