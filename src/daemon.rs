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

use crate::config::Backend;
use crate::config::Config;
use crate::config::Matrix as MatrixConfig;
use crate::config::Slack as SlackConfig;
use crate::listener::Listener;
use crate::matrix::Matrix;
use crate::slack::Slack;
use crate::spool_dispatcher::SpoolDispatcher;
use crate::spooler::Spooler;

use tokio::sync::broadcast::Sender;

use log::error;

pub struct Daemon {
    listener: Listener,

    spool_dispatcher: SpoolDispatcher,

    slack: Option<Slack>,

    matrix: Option<Matrix>,

    terminator: Sender<()>,
}

impl Daemon {
    pub fn new(config: Config) -> Option<Self> {
        let (to_matrix, matrix_receiver) = tokio::sync::mpsc::channel(5);
        let (to_spooler, spooler_receiver) = tokio::sync::mpsc::channel(5);
        let (terminator, terminatee) = tokio::sync::broadcast::channel(1);
        let (to_verifier, verifier_receiver) = tokio::sync::mpsc::unbounded_channel();

        let spooler = Spooler::new(&config.spool_path);

        let spool_dispatcher = SpoolDispatcher::new(
            spooler,
            to_matrix.clone(),
            spooler_receiver,
            terminator.subscribe(),
        );

        let listener = Listener::new(
            &config.socket_path,
            to_matrix,
            to_verifier,
            terminator.subscribe(),
        );

        let (slack, matrix) = match config.backend {
            Backend::Slack(SlackConfig { webhook }) => {
                let slack = Slack::new(matrix_receiver, to_spooler, webhook, terminatee);

                (Some(slack), None)
            }
            Backend::Matrix(MatrixConfig {
                user,
                password,
                room,
                message_template,
            }) => {
                let matrix = match Matrix::new(
                    &user,
                    &password,
                    &room,
                    &message_template,
                    matrix_receiver,
                    to_spooler,
                    verifier_receiver,
                ) {
                    Err(e) => {
                        error!("{}", e);
                        return None;
                    }
                    Ok(v) => v,
                };

                (None, Some(matrix))
            }
        };

        Some(Self {
            listener,
            spool_dispatcher,
            slack,
            matrix,
            terminator,
        })
    }

    pub fn run(self) -> Result<(), tokio::io::Error> {
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(3)
            .enable_all()
            .thread_name("tokio runtime")
            .build()?;

        let listener = self.listener;
        let mut listener = match tokio_runtime.block_on(async move { listener.start() }) {
            Err(e) => {
                error!("Failed to start UNIX domain socket listener: {:#?}", e);
                return Ok(());
            }
            Ok(v) => v,
        };

        let matrix = self.matrix;
        if let Some(matrix) = matrix {
            let mut matrix = match tokio_runtime.block_on(matrix.login()) {
                Err(e) => {
                    error!("{}", e);
                    return Ok(());
                }
                Ok(v) => v,
            };
            tokio_runtime.spawn(async move {
                matrix.run().await;
            });
        }

        let slack = self.slack;
        if let Some(mut slack) = slack {
            tokio_runtime.spawn(async move {
                slack.send_messages().await;
            });
        }

        let spool_dispatcher = self.spool_dispatcher;
        tokio_runtime.spawn(spool_dispatcher.run());

        tokio_runtime.spawn(async move {
            listener.handle_new_messages().await;
        });

        tokio_runtime.spawn(crate::systemd::watchdog());

        tokio_runtime.spawn(crate::systemd::notify_about_start());

        tokio_runtime.block_on(crate::terminator::terminator(self.terminator))?;

        Ok(())
    }
}
