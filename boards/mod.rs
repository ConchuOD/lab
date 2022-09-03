// SPDX-License-Identifier: LGPL-3.0-only

#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use serde_yaml::Value;
use std::fs;
use std::fmt;
//use crate::boards;

#[derive(Debug)]
pub struct ConfigParsingError {
	details: String
}

impl ConfigParsingError {
	pub fn new(msg: &str) -> ConfigParsingError {
		return ConfigParsingError{details: msg.to_string()}
	}
}

impl fmt::Display for ConfigParsingError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		return write!(f, "Invalid Yaml Config: {}", self.details)
	}
}

impl std::error::Error for ConfigParsingError {
	fn description(&self) -> &str {
		return &self.details
	}
}

pub fn get_board_from_config(board: String, input_file: String, serial: &mut String,
			     port: &mut String, power_source: &mut String)
-> Result<(), Box<dyn std::error::Error>>
{
	let contents = fs::read_to_string(input_file)?;

	let config: Value = serde_yaml::from_str(&contents)?;

	let board_config = config
		.get("boards")
		.ok_or_else(|| return ConfigParsingError::new("No boards found"))?
		.get(board)
		.ok_or_else(|| return ConfigParsingError::new("Requested board not found"))?;

	*serial = board_config
		.get("serial")
		.ok_or_else(|| return ConfigParsingError::new("No serial number found"))?
		.as_str()
		.ok_or_else(|| return ConfigParsingError::new("Serial number was not a string"))?
		.to_owned();

	*port = board_config
		.get("port")
		.ok_or_else(|| return ConfigParsingError::new("No port number found"))?
		.as_str()
		.ok_or_else(|| return ConfigParsingError::new("Port number was not a string"))?
		.to_owned();

	*power_source = board_config
		.get("type")
		.ok_or_else(|| return ConfigParsingError::new("No type found"))?
		.as_str()
		.ok_or_else(|| return ConfigParsingError::new("Type was not a string"))?
		.to_owned();

	return Ok(());
}
