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

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub channel: Option<String>,

    pub username: Option<String>,

    pub text: Option<String>,

    pub icon_emoji: Option<String>,

    pub attachments: Option<Vec<Attachment>>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Attachment {
    pub fallback: Option<String>,

    pub color: Option<String>,

    pub pretext: Option<String>,

    pub author_name: Option<String>,

    pub author_link: Option<String>,

    pub author_icon: Option<String>,

    pub title: Option<String>,

    pub title_link: Option<String>,

    pub text: Option<String>,

    pub fields: Option<Vec<Field>>,

    pub image_url: Option<String>,

    pub thumb_url: Option<String>,

    pub footer: Option<String>,

    pub footer_icon: Option<String>,

    pub ts: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Level {
    OK,
    WARN,
    ERROR,
    UNKNOWN,
}

impl Level {
    pub fn into_string(self) -> String {
        match self {
            Level::OK => "#44bb77",
            Level::WARN => "#ffaa44",
            Level::ERROR => "#ff5566",
            _ => "#aa44ff",
        }
        .to_string()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Field {
    pub title: String,

    pub value: String,

    pub short: bool,
}
