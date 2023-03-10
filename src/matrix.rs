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
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::message::Message;
use crate::message::Sas;
use crate::util;

use matrix_sdk::instant::Duration;
use matrix_sdk::ruma::events::key::verification::ShortAuthenticationString;
use matrix_sdk::ruma::events::key::verification::VerificationMethod;
use matrix_sdk::ruma::events::room::message::MessageEventContent;
use matrix_sdk::ruma::events::room::message::MessageType;
use matrix_sdk::ruma::events::AnyMessageEventContent;
use matrix_sdk::ruma::events::AnySyncMessageEvent;
use matrix_sdk::ruma::events::AnySyncRoomEvent;
use matrix_sdk::ruma::events::AnyToDeviceEvent;
use matrix_sdk::verification::SasVerification as RemoteSas;
use matrix_sdk::verification::Verification;
use matrix_sdk::Client;
use matrix_sdk::ClientConfig;
use matrix_sdk::LoopCtrl;
use matrix_sdk::SyncSettings;

use matrix_sdk_crypto::AcceptSettings;

use thiserror::Error;

use url::Url;

use serde::Deserialize;

use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

use tera::Context;
use tera::Tera;

use log::debug;
use log::info;
use log::trace;
use log::warn;

pub struct Matrix {
    client: Client,

    username: String,

    password: String,

    channel: String,

    tera: Tera,

    spooler: Receiver<Message>,

    send_reporter: Sender<Option<Message>>,

    verifier: Option<UnboundedReceiver<Sas>>,
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
        verifier: UnboundedReceiver<Sas>,
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
            verifier: Some(verifier),
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
        let client_for_syncer = syncer.clone();

        let (to_verifier, from_matrix) = tokio::sync::mpsc::unbounded_channel();

        let mut verifier = Verifier {
            local_receiver: self.verifier.take().unwrap(),
            remote_receiver: from_matrix,
        };

        tokio::spawn(async move { verifier.run().await });

        tokio::spawn(async move {
            let client = client_for_syncer;
            let client = &client;
            let to_verifier = to_verifier;
            let to_verifier_ref = &to_verifier;
            let settings = SyncSettings::new().timeout(Duration::from_secs(300));

            let initial_call_flag = Arc::new(AtomicBool::from(true));
            let initial_call_flag_ref = &initial_call_flag;

            syncer
                .sync_with_callback(settings, |response| async move {
                    for raw_event in response.to_device.events {
                        let event = match raw_event.deserialize() {
                            Err(e) => {
                                warn!("failed to deserialise event: {}", e);
                                continue;
                            }
                            Ok(v) => v,
                        };
                        Self::handle_to_device_event(client, event, to_verifier_ref).await;
                    }

                    if !initial_call_flag_ref.load(Ordering::SeqCst) {
                        for (_room_id, room_info) in response.rooms.join {
                            for event in room_info
                                .timeline
                                .events
                                .iter()
                                .filter_map(|e| e.event.deserialize().ok())
                            {
                                Self::handle_room_event(client, event, to_verifier_ref).await;
                            }
                        }
                    }

                    initial_call_flag_ref.store(false, Ordering::SeqCst);

                    LoopCtrl::Continue
                })
                .await
        });

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
            MessageEventContent::text_html("", html),
        ))
    }

    async fn handle_to_device_event(
        client: &Client,
        event: AnyToDeviceEvent,
        channel: &UnboundedSender<RemoteSas>,
    ) {
        match event {
            AnyToDeviceEvent::KeyVerificationRequest(e) => {
                debug!("ToDevice Verification requested");
                match client
                    .get_verification_request(&e.sender, &e.content.transaction_id)
                    .await
                {
                    Some(request) => {
                        info!("Starting verification with {}", request.other_user_id());
                        request
                            .accept_with_methods(vec![VerificationMethod::SasV1])
                            .await
                            .unwrap();
                    }
                    _ => debug!("No verification request found"),
                }
            }
            AnyToDeviceEvent::KeyVerificationStart(e) => {
                debug!("ToDevice Verification started");
                match client
                    .get_verification(&e.sender, &e.content.transaction_id)
                    .await
                {
                    Some(Verification::SasV1(sas)) => {
                        info!(
                            "Starting verification with {} {}",
                            &sas.other_device().user_id(),
                            &sas.other_device().device_id()
                        );
                        sas.accept_with_settings(AcceptSettings::with_allowed_methods(vec![
                            ShortAuthenticationString::Decimal,
                        ]))
                        .await
                        .unwrap();
                    }
                    v => debug!("Unknown variant of KeyVerificationStart: {:#?}", v),
                }
            }

            AnyToDeviceEvent::KeyVerificationKey(e) => {
                debug!("ToDevice Verification key obtained");
                match client
                    .get_verification(&e.sender, &e.content.transaction_id)
                    .await
                {
                    Some(Verification::SasV1(sas)) => {
                        if let Err(e) = channel.send(sas) {
                            warn!("Failed to send to verifier: {}", e);
                        }
                    }
                    v => debug!("Unknown variant of KeyVerificationKey: {:#?}", v),
                }
            }

            AnyToDeviceEvent::KeyVerificationMac(e) => {
                debug!("ToDevice Verification mac obtained");
                match client
                    .get_verification(&e.sender, &e.content.transaction_id)
                    .await
                {
                    Some(Verification::SasV1(sas)) => {
                        if sas.is_done() {
                            info!(
                                "Successfully verified device {} {}",
                                sas.other_device().user_id(),
                                sas.other_device().device_id(),
                            );
                        }
                    }
                    v => debug!("Unknown variant of KeyVerificationMac: {:#?}", v),
                }
            }
            v => trace!("Unknown event: {:#?}", v),
        }
    }

    async fn handle_room_event(
        client: &Client,
        event: AnySyncRoomEvent,
        channel: &UnboundedSender<RemoteSas>,
    ) {
        if let AnySyncRoomEvent::Message(event) = event {
            match event {
                AnySyncMessageEvent::RoomMessage(m) => {
                    if let MessageType::VerificationRequest(_) = &m.content.msgtype {
                        debug!("SyncRoom Verification requested");
                        match client
                            .get_verification_request(&m.sender, &m.event_id)
                            .await
                        {
                            Some(request) => {
                                info!("Starting verification with {}", request.other_user_id());
                                request
                                    .accept_with_methods(vec![VerificationMethod::SasV1])
                                    .await
                                    .unwrap();
                            }
                            _ => debug!("No verification request found"),
                        }
                    } else {
                        debug!("Got unknown room message {:#?}", m);
                    }
                }
                AnySyncMessageEvent::KeyVerificationKey(e) => {
                    debug!("SyncRoom Verification key obtained");
                    match client
                        .get_verification(&e.sender, e.content.relates_to.event_id.as_str())
                        .await
                    {
                        Some(Verification::SasV1(sas)) => {
                            if let Err(e) = channel.send(sas) {
                                warn!("Failed to send to verifier: {}", e);
                            }
                        }
                        _ => debug!("No verification information found"),
                    }
                }
                AnySyncMessageEvent::KeyVerificationMac(e) => {
                    debug!("SyncRoom Verification mac obtained");
                    match client
                        .get_verification(&e.sender, e.content.relates_to.event_id.as_str())
                        .await
                    {
                        Some(Verification::SasV1(sas)) => {
                            if sas.is_done() {
                                info!(
                                    "Successfully verified device {} {}",
                                    sas.other_device().user_id(),
                                    sas.other_device().device_id(),
                                );
                            }
                        }
                        _ => debug!("No verification information found"),
                    }
                }
                e => debug!("Unknown room event {:#?}", e),
            }
        }
    }
}

struct Verifier {
    local_receiver: UnboundedReceiver<Sas>,

    remote_receiver: UnboundedReceiver<RemoteSas>,
}

impl Verifier {
    async fn run(&mut self) {
        loop {
            match tokio::join!(self.local_receiver.recv(), self.remote_receiver.recv()) {
                (Some(local_sas), Some(remote_sas)) => {
                    self.check_equality(local_sas, remote_sas).await;
                }
                (None, None) => break,
                (_, None) => {
                    warn!("Received no remote SAS");
                }
                (None, _) => {
                    warn!("Received no local SAS");
                }
            }
        }
    }

    async fn check_equality(&self, local_sas: Sas, remote_sas: RemoteSas) {
        let remote = match remote_sas.decimals() {
            None => {
                warn!("Remote SAS contains no decimals");
                return;
            }
            Some(v) => v,
        };

        let remote = format!("{},{},{}", remote.0, remote.1, remote.2);

        debug!("Remote: '{}'", remote);
        debug!("Local: '{}'", local_sas.input);

        if local_sas.input == remote {
            info!("Verification successful");
            if let Err(e) = remote_sas.confirm().await {
                warn!("Failed to tell remote about successful verification: {}", e);
            }
        } else {
            warn!(
                "Verification failed. Local: '{}'. Remote: '{}'",
                local_sas.input, remote
            );
            if let Err(e) = remote_sas.cancel().await {
                warn!("Failed to tell remote about failed verification: {}", e);
            }
        }
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

    let response = reqwest::blocking::get(url)?.json::<WellKnown>()?;

    Ok(response.homeserver_url.base_url)
}
