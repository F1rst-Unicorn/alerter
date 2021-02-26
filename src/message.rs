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

use std::collections::BTreeMap;

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub title: String,

    pub text: String,

    pub level: Level,

    pub link: Option<String>,

    pub fields: BTreeMap<String, String>,

    pub channel: Option<String>,

    pub timestamp: i64,

    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Level {
    OK,
    WARN,
    ERROR,
    UNKNOWN,
}

impl Default for Level {
    fn default() -> Self {
        Level::UNKNOWN
    }
}

impl From<Level> for String {
    fn from(v: Level) -> Self {
        match v {
            Level::OK => "#44bb77",
            Level::WARN => "#ffaa44",
            Level::ERROR => "#ff5566",
            _ => "#aa44ff",
        }
        .to_string()
    }
}
