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

use tokio::fs::File;
use tokio::fs::OpenOptions;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::io::BufWriter;
use tokio::io::ErrorKind;
use tokio::stream::StreamExt;

use log::{debug, error, info, warn};

use crate::message::Message;

pub struct Spooler {
    spool_path: String,

    queue: Vec<Message>,
}

impl Spooler {
    pub fn new(spool_path: &str) -> Self {
        Spooler {
            spool_path: spool_path.to_string(),

            queue: Vec::new(),
        }
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

    pub async fn store(&self) {
        match self.queue.len() {
            0 => info!("Clearing stored message queue"),
            1 => warn!("Storing 1 queued message"),
            i => warn!("Storing {} queued messages", i),
        }

        let file = match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.spool_path)
            .await
        {
            Err(e) => {
                error!("spooler failed to open queue file: {}", e);
                return;
            }
            Ok(v) => v,
        };

        let mut writer = BufWriter::new(file);
        let text = self
            .queue
            .iter()
            .map(serde_json::to_string)
            .map(Result::unwrap)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";

        if let Err(e) = writer.write_all(text.as_bytes()).await {
            error!("Failed to store message '{}'", e);
        }

        if let Err(e) = writer.shutdown().await {
            error!("Failed to store message '{}'", e);
        }
    }

    pub async fn load(&mut self) {
        debug!("Loading queued messages");

        let file = match File::open(&self.spool_path).await {
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    debug!("No queued messages");
                    return;
                } else {
                    error!("spooler failed to open queue file: {}", e);
                }
                return;
            }
            Ok(v) => v,
        };

        let reader = BufReader::new(file);
        let result = reader
            .lines()
            .filter(Result::is_ok)
            .map(Result::unwrap)
            .map(|s| serde_json::from_str::<Message>(&s))
            .map(|v| match v {
                Ok(m) => self.queue.push(m),
                Err(e) => error!("Failed to load message with error '{}'", e),
            })
            .collect::<Vec<_>>();

        result.await;

        match self.queue.len() {
            0 => debug!("There are no queued messages"),
            1 => warn!("There is 1 queued message"),
            i => warn!("There are {} queued messages", i),
        }
    }
}
