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

use std::collections::BTreeMap;

use serde_derive::Deserialize;
use serde_derive::Serialize;

use chrono::DateTime;
use chrono::Local;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Packet {
    Message(Message),

    Sas(Sas),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sas {
    pub input: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub title: String,

    pub text: String,

    pub level: Level,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,

    pub fields: BTreeMap<String, String>,

    pub channel: Option<String>,

    #[serde(with = "chrono_serde")]
    pub timestamp: DateTime<Local>,

    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum Level {
    #[serde(rename = "OK")]
    Ok,
    #[serde(rename = "WARN")]
    Warn,
    #[serde(rename = "ERROR")]
    Error,
    #[default]
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

impl From<Level> for String {
    fn from(v: Level) -> Self {
        match v {
            Level::Ok => "#44bb77",
            Level::Warn => "#ffaa44",
            Level::Error => "#ff5566",
            _ => "#aa44ff",
        }
        .to_string()
    }
}

mod chrono_serde {
    use chrono::{DateTime, Local, TimeZone};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.timestamp();
        serializer.serialize_i64(s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = i64::deserialize(deserializer)?;
        Ok(Local.timestamp(secs, 0))
    }
}
