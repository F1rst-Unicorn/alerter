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
use reqwest;
use futures::executor::block_on;

use log::{debug, warn};

pub struct Slack {
    webhook_url: String,
}

#[derive(Debug)]
pub enum Error {
    ReqwestError(reqwest::Error),

    StatusCode(u16),

    General(reqwest::Error),
}

impl Slack {
    pub fn new(webhook_url: String) -> Self {
        Slack { webhook_url }
    }

    pub fn send(&self, message: &Message) -> Result<(), Error> {
        debug!("Sending message");

        let client = reqwest::Client::builder()
            .build()
            .map_err(Error::ReqwestError)?;

        let response = client.post(&self.webhook_url).json(message).send();
        let response = block_on(response);

        match response {
            Ok(r) => match r.status().as_u16() {
                200 => Ok(()),
                c => {
                    warn!("Upstream reported error: {:#?}", r);
                    Err(Error::StatusCode(c))
                }
            },
            Err(e) => Err(Error::General(e)),
        }
    }
}
