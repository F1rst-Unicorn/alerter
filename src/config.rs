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
use std::io;
use std::io::Read;
use std::process::exit;

use serde_derive::Deserialize;

use log::error;

const EXIT_CODE: i32 = 1;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Config {
    pub socket_path: String,

    pub webhook: Option<String>,
}

pub fn parse_config(file_path: &str) -> Config {
    let raw_config = read_file(file_path);

    if let Err(e) = raw_config {
        error!("Could not read config: {}", e);
        exit(EXIT_CODE);
    }

    let raw_config = raw_config.unwrap();
    let parse_result = serde_yaml::from_str(raw_config.as_str());

    match parse_result {
        Err(e) => {
            error!("Could not parse config: {:#?}", e);
            exit(EXIT_CODE);
        }
        Ok(r) => r,
    }
}

pub fn read_file(file_path: &str) -> Result<String, io::Error> {
    let mut file = File::open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}
