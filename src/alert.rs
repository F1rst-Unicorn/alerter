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

pub mod alert_cli_parser;
pub mod config;
pub mod logging;
pub mod message;

use crate::message::Attachment;
use crate::message::Field;
use crate::message::Message;

use std::io::Write;
use std::os::unix::net::UnixStream;
use std::process::exit;

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
    let mut result: Message = Default::default();

    result.username = config::read_file("/etc/hostname")
        .map(|v| v.trim().to_string())
        .ok();
    result.channel = args
        .value_of(alert_cli_parser::FLAG_CHANNEL)
        .map(str::to_string);

    let color = parse_color(args.value_of(alert_cli_parser::FLAG_LEVEL));

    let mut attachment: Attachment = Default::default();
    attachment.title = args
        .value_of(alert_cli_parser::FLAG_TITLE)
        .map(str::to_string);
    attachment.title_link = args
        .value_of(alert_cli_parser::FLAG_TITLE_LINK)
        .map(str::to_string);
    attachment.text = args
        .value_of(alert_cli_parser::FLAG_TEXT)
        .map(str::to_string);
    attachment.color = color;
    attachment.footer = Some("alert v".to_string() + env!("CARGO_PKG_VERSION"));
    attachment.ts = Some(chrono::Utc::now().timestamp());
    attachment.fields = parse_additional_fields(args.values_of(alert_cli_parser::FLAG_FIELD));

    result.attachments = Some(vec![attachment]);
    result
}

fn parse_color(color: Option<&str>) -> Option<String> {
    let color = color
        .map(serde_yaml::from_str::<message::Level>)
        .map(Result::ok)
        .flatten()
        .map(From::from);

    if color.is_none() {
        error!("Invalid level given");
        exit(1);
    }
    color
}

fn parse_additional_fields(values: Option<clap::Values>) -> Option<Vec<Field>> {
    let mut fields = Vec::new();
    if let Some(values) = values {
        for item in values {
            let mut split = item.splitn(2, ':');
            if let Some(key) = split.next() {
                if let Some(value) = split.next() {
                    let field = Field {
                        title: key.to_string(),
                        value: value.to_string(),
                        short: true,
                    };
                    fields.push(field);
                } else {
                    warn!("Skipping field '{}' because of missing ':'", item);
                }
            }
        }
    }
    Some(fields)
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
