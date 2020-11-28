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

use std::num::Wrapping;

use log::info;

#[derive(Default)]
pub struct Backoff {
    backoff: Wrapping<u64>,
}

impl Backoff {
    pub fn new() -> Self {
        Backoff {
            backoff: Wrapping(1u64),
        }
    }

    pub fn reset(&mut self) {
        self.backoff = Wrapping(1u64);
    }

    pub fn get_backoff(&self) -> u64 {
        self.backoff.0
    }

    pub fn backoff(&mut self) {
        self.backoff *= Wrapping(2);
        info!("Increasing backoff, now at {}", self.backoff);
    }
}
