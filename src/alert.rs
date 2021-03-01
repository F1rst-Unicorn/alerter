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

pub mod alert_cli_parser;
pub mod config;
pub mod logging;
pub mod message;
pub mod util;

use crate::message::Level;
use crate::message::Message;

use std::collections::BTreeMap;
use std::io::Write;
use std::os::unix::net::UnixStream;

use chrono::Local;

use log::debug;
use log::error;
use log::warn;

fn main() {
    let arguments = alert_cli_parser::parse_arguments();
    logging::initialise(
        arguments
            .value_of(alert_cli_parser::FLAG_LOG_CONFIG)
            .expect("Missing default value in cli_parser"),
    );

    debug!("Starting up");

    let config_path = arguments
        .value_of(alert_cli_parser::FLAG_CONFIG)
        .expect("Missing default value in alert_cli_parser");
    debug!("Config is at {}", config_path);

    let config = config::parse_config(config_path);

    let message = compose_message_from_arguments(arguments);

    send_message(&config.socket_path, message);
}

fn compose_message_from_arguments(args: clap::ArgMatches) -> Message {
    Message {
        title: args
            .value_of(alert_cli_parser::FLAG_TITLE)
            .map(str::to_string)
            .unwrap_or_default(),

        text: args
            .value_of(alert_cli_parser::FLAG_TEXT)
            .map(str::to_string)
            .unwrap_or_default(),

        channel: args
            .value_of(alert_cli_parser::FLAG_CHANNEL)
            .map(str::to_string),

        level: args
            .value_of(alert_cli_parser::FLAG_LEVEL)
            .map(serde_yaml::from_str::<Level>)
            .map(Result::ok)
            .flatten()
            .unwrap_or_default(),

        link: args
            .value_of(alert_cli_parser::FLAG_TITLE_LINK)
            .map(str::to_string),

        version: env!("CARGO_BIN_NAME").to_string() + " v" + env!("CARGO_PKG_VERSION"),

        timestamp: Local::now(),

        fields: parse_additional_fields(args.values_of(alert_cli_parser::FLAG_FIELD)),
    }
}

fn parse_additional_fields(values: Option<clap::Values>) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::default();
    if let Some(values) = values {
        for item in values {
            let mut split = item.splitn(2, ':');
            if let Some(key) = split.next() {
                if let Some(value) = split.next() {
                    fields.insert(key.to_string(), value.to_string());
                } else {
                    warn!("Skipping field '{}' because of missing ':'", item);
                }
            }
        }
    }
    fields
}

fn send_message(socket_path: &str, message: Message) {
    let mut stream = match UnixStream::connect(socket_path) {
        Err(e) => {
            error!("Failed to open socket: {}", e);
            return;
        }
        Ok(v) => v,
    };

    let raw_message = match serde_json::to_string(&message) {
        Err(e) => {
            error!("Failed to message to string: {}", e);
            return;
        }
        Ok(v) => v,
    };

    debug!("Sending {}", raw_message);

    if let Err(e) = stream.write_all(&raw_message.into_bytes()) {
        error!("Failed to hand over message to daemon: {}", e);
    }
}
