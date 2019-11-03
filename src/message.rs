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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub channel: Option<String>,

    pub username: Option<String>,

    pub text: String,

    pub icon_emoji: Option<String>,

    pub attachments: Option<Vec<Attachment>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attachment {
    pub fallback: String,

    pub color: String,

    pub pretext: String,

    pub author_name: String,

    pub author_link: String,

    pub author_icon: String,

    pub title: String,

    pub title_link: String,

    pub text: String,

    pub fields: Vec<Field>,

    pub image_url: String,

    pub thumb_url: String,

    pub footer: String,

    pub footer_icon: String,

    pub ts: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Field {
    pub title: String,

    pub value: String,

    pub short: bool,
}
