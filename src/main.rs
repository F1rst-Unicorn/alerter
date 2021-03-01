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
pub mod backoff;
pub mod cli_parser;
pub mod config;
pub mod daemon;
pub mod listener;
pub mod logging;
pub mod matrix;
pub mod message;
pub mod slack;
pub mod spool_dispatcher;
pub mod spooler;
pub mod systemd;
pub mod terminator;
pub mod util;

use daemon::Daemon;

use log::debug;
use log::error;

fn main() {
    let arguments = cli_parser::parse_arguments();
    logging::initialise(
        arguments
            .value_of(cli_parser::FLAG_LOG_CONFIG)
            .expect("Missing default value in cli_parser"),
    );

    debug!("Starting up");

    let config_path = arguments
        .value_of(cli_parser::FLAG_CONFIG)
        .expect("Missing default value in cli_parser");
    debug!("Config is at {}", config_path);

    let config = config::parse_config(config_path);

    match Daemon::new(config) {
        None => {}
        Some(daemon) => match daemon.run() {
            Err(e) => {
                error!("Failed to start runtime: {}", e);
            }
            Ok(()) => {}
        },
    };
}
