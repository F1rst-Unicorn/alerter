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

use clap::App;
use clap::Arg;

pub const FLAG_LOG_CONFIG: &str = "log-config";
pub const FLAG_CONFIG: &str = "config";

pub const FLAG_TITLE: &str = "TITLE";
pub const FLAG_TITLE_LINK: &str = "TITLE_LINK";
pub const FLAG_TEXT: &str = "TEXT";
pub const FLAG_CHANNEL: &str = "CHANNEL";
pub const FLAG_LEVEL: &str = "LEVEL";
pub const FLAG_FIELD: &str = "FIELD";
pub const FLAG_VERIFY: &str = "VERIFY";

pub fn parse_arguments() -> clap::ArgMatches {
    App::new("alert")
        .version(concat!(
            env!("CARGO_PKG_VERSION"),
            " ",
            env!("VERGEN_GIT_SHA"),
            " ",
            env!("VERGEN_BUILD_TIMESTAMP")
        ))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new(FLAG_TITLE)
                .help("The title of the message")
                .value_name("TITLE")
                .required_unless_present(FLAG_VERIFY),
        )
        .arg(
            Arg::new(FLAG_TEXT)
                .help("The content of the message")
                .value_name("TEXT")
                .required_unless_present(FLAG_VERIFY),
        )
        .arg(
            Arg::new(FLAG_VERIFY)
                .short('V')
                .long("verify")
                .help("Verify this matrix device. E.g. 1234,1234,1234")
                .value_name("SAS")
                .conflicts_with(FLAG_TITLE)
                .conflicts_with(FLAG_TEXT),
        )
        .arg(
            Arg::new(FLAG_CONFIG)
                .short('C')
                .long(FLAG_CONFIG)
                .value_name("PATH")
                .help("The config file or directory to run with")
                .takes_value(true)
                .default_value("/etc/alerter/alert.yml"),
        )
        .arg(
            Arg::new(FLAG_CHANNEL)
                .short('c')
                .long("channel")
                .value_name("channel")
                .help("The channel to send to"),
        )
        .arg(
            Arg::new(FLAG_LEVEL)
                .short('l')
                .long("level")
                .value_name("level")
                .help("One of OK, WARN, ERROR, UNKNOWN")
                .default_value("UNKNOWN"),
        )
        .arg(
            Arg::new(FLAG_TITLE_LINK)
                .short('t')
                .long("title-link")
                .value_name("link")
                .help("A link to further information"),
        )
        .arg(
            Arg::new(FLAG_FIELD)
                .short('f')
                .long("field")
                .value_name("field")
                .help("More key-value pairs as key:value")
                .multiple_occurrences(true),
        )
        .arg(
            Arg::new(FLAG_LOG_CONFIG)
                .short('v')
                .long(FLAG_LOG_CONFIG)
                .help("The log4rs logging configuration")
                .takes_value(true)
                .default_value("/etc/alerter/log4rs.yml"),
        )
        .get_matches()
}
