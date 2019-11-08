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
pub mod cli_parser;
pub mod config;
pub mod daemon;
pub mod logging;
pub mod message;
pub mod slack;

use daemon::Daemon;
use log::{debug, error};
use slack::Slack;

use std::process::exit;

fn main() {
    let arguments = cli_parser::parse_arguments();
    logging::initialise(arguments.occurrences_of(cli_parser::FLAG_VERBOSE));

    debug!("Starting up");

    let config_path = arguments
        .value_of(cli_parser::FLAG_CONFIG)
        .expect("Missing default value in cli_parser");
    debug!("Config is at {}", config_path);

    let config = config::parse_config(config_path);

    if config.webhook.is_none() {
        error!("No webhook configured");
        exit(1);
    }

    let slack = Slack::new(config.webhook.unwrap());

    let mut daemon = Daemon::new(&config.socket_path, slack);

    daemon.run();
}
