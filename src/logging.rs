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

use log4rs;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;

use log::LevelFilter;

pub fn initialise(verbosity_level: u64) {
    let stdout = log4rs::append::console::ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{level} {m}{n}")))
        .build();

    let level = match verbosity_level {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    let mut config =
        Config::builder().appender(Appender::builder().build("stdout", Box::new(stdout)));

    for module in &["tokio_reactor", "hyper", "mio", "want", "reqwest"] {
        config = config.logger(
            Logger::builder()
                .additive(false)
                .build(*module, LevelFilter::Info),
        )
    }

    let config = config
        .build(Root::builder().appender("stdout").build(level))
        .expect("Could not configure logging");

    log4rs::init_config(config).expect("Could not apply log config");
}
