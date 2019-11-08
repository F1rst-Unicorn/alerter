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

use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::ErrorKind;
use std::io::Write;
use std::process::exit;

use log::{debug, error, info, warn};

use crate::message::Message;

const EXIT_CODE: i32 = 2;

pub struct Spooler {
    spool_path: String,

    queue: Vec<Message>,
}

impl Spooler {
    pub fn new(spool_path: &str) -> Self {
        let mut result = Spooler {
            spool_path: spool_path.to_string(),

            queue: Vec::new(),
        };

        result.load();
        result
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn queue(&mut self, m: Message) {
        debug!("Queueing message");
        self.queue.push(m);
    }

    pub fn queue_front(&mut self, m: Message) {
        debug!("Queueing message at the front");
        self.queue.insert(0, m);
    }

    pub fn pop_message(&mut self) -> Option<Message> {
        if self.queue.is_empty() {
            None
        } else {
            Some(self.queue.remove(0))
        }
    }

    pub fn store(&self) {
        match self.queue.len() {
            0 => info!("Clearing stored message queue"),
            1 => warn!("Storing 1 queued message"),
            i => warn!("Storing {} queued messages", i),
        }

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.spool_path);
        if let Err(e) = file {
            error!("spooler failed to open queue file: {}", e);
            return;
        }

        let mut writer = BufWriter::new(file.unwrap());
        let results = self
            .queue
            .iter()
            .map(serde_json::to_string)
            .map(Result::unwrap)
            .map(|mut s| {
                s.push('\n');
                s
            })
            .map(String::into_bytes)
            .map(|s| writer.write(&s))
            .collect::<Vec<_>>();

        for result in results {
            if let Err(e) = result {
                error!("Failed to store message '{}'", e);
            }
        }
    }

    fn load(&mut self) {
        debug!("Loading queued messages");

        let file = File::open(&self.spool_path);
        if let Err(e) = file {
            if e.kind() == ErrorKind::NotFound {
                debug!("No queued messages");
                return;
            } else {
                error!("spooler failed to open queue file: {}", e);
                exit(EXIT_CODE);
            }
        }

        let reader = BufReader::new(file.unwrap());
        let results = reader
            .lines()
            .filter(Result::is_ok)
            .map(Result::unwrap)
            .map(|s| serde_json::from_str::<Message>(&s))
            .collect::<Vec<_>>();

        for result in results {
            match result {
                Ok(m) => self.queue.push(m),
                Err(e) => error!("Failed to load message with error '{}'", e),
            }
        }

        match self.queue.len() {
            0 => debug!("There are no queued messages"),
            1 => warn!("There is 1 queued message"),
            i => warn!("There are {} queued messages", i),
        }
    }
}
