/*  alerter: Alerter to chat servers
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

use serde::Deserialize;

use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use tera::Context;
use tera::Tera;

use log::debug;
use log::warn;

pub struct Matrix {
    client: Client,

    username: String,

    password: String,

    channel: String,

    tera: Tera,

    spooler: Receiver<Message>,

    send_reporter: Sender<Option<Message>>,

    terminator: tokio::sync::broadcast::Receiver<()>,
}

#[derive(Error, Debug)]
#[allow(clippy::large_enum_variant)]
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

    #[error("matrix server resolution failed")]
    MatrixWellKnown(#[from] reqwest::Error),

    #[error("message template is invalid: {0:#?}")]
    Tera(#[from] tera::Error),
}

impl Matrix {
    pub fn new(
        user: &str,
        password: &str,
        channel: &str,
        message_template: &str,
        spooler: Receiver<Message>,
        send_reporter: Sender<Option<Message>>,
        terminator: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<Self, Error> {
        let mut iter = user.splitn(2, ':');
        let username = iter.next().ok_or(Error::InvalidUser)?;
        let server = iter.next().ok_or(Error::InvalidUser)?;

        let homeserver_url = resolve_well_known(server)?;
        let homeserver_url = Url::parse(&homeserver_url).map_err(|_| Error::InvalidHomeServer)?;

        let config = ClientConfig::new().store_path("./");

        let client = Client::new_with_config(homeserver_url, config)?;

        let mut tera = Tera::default();
        tera.add_raw_template("", message_template)?;

        Ok(Matrix {
            client,
            username: username.to_string(),
            password: password.to_string(),
            channel: channel.to_string(),
            tera,
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

        let room = TryFrom::try_from(channel).map_err(|_| ())?;

        let html = self.render(message).map_err(|_| ())?;

        match self.client.room_send(&room, html, None).await {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Error while sending: {:#?}", e);
                Err(())
            }
        }
    }

    fn render(&self, message: &Message) -> Result<AnyMessageEventContent, ()> {
        let mut context = Context::default();
        context.insert("m", message);
        context.insert("level_color", &String::from(message.level.clone()));

        let html = self.tera.render("", &context).map_err(|_| ())?;

        Ok(AnyMessageEventContent::RoomMessage(
            MessageEventContent::Text(
                matrix_sdk::events::room::message::TextMessageEventContent::html("", html),
            ),
        ))
    }
}

#[derive(Deserialize)]
struct WellKnown {
    #[serde(rename = "m.homeserver")]
    homeserver_url: Server,
}

#[derive(Deserialize)]
struct Server {
    base_url: String,
}

pub fn resolve_well_known(matrix_id: &str) -> Result<String, Error> {
    let url = format!("https://{}/.well-known/matrix/client", matrix_id);

    let response = reqwest::blocking::get(&url)?.json::<WellKnown>()?;

    Ok(response.homeserver_url.base_url)
}
