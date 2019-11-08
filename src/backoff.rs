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

use log::{debug, info};

#[derive(Default)]
pub struct Backoff {
    backoff: Wrapping<u64>,

    counter: u64,
}

impl Backoff {
    pub fn new() -> Self {
        Backoff {
            backoff: Wrapping(1u64),
            counter: 0,
        }
    }

    pub fn reset(&mut self) {
        self.backoff = Wrapping(1u64);
        self.counter = 0;
    }

    pub fn inc(&mut self) {
        self.counter += 1;
        debug!("Backoff {}/{}", self.counter, self.backoff);
    }

    pub fn is_ready(&self) -> bool {
        Wrapping(self.counter) == self.backoff
    }

    pub fn backoff(&mut self) {
        self.backoff *= Wrapping(2);
        self.counter = 0;
        info!("Increasing backoff, now at {}", self.backoff);
    }
}
