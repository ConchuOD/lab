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

#[derive(Clone)]
pub struct Board {
	pub name: String,
	pub yk_serial_number: String,
	pub yk_port_number: String,
	pub power_source: String
}

impl Default for Board {
	fn default() -> Board
	{
		return Board {
			name: "n/a".to_string(),
			yk_serial_number: "n/a".to_string(),
			yk_port_number: "n/a".to_string(),
			power_source: "n/a".to_string()
		}
	}
}

fn populate_board(mut board: &mut Board, board_config: Value)
-> Result<(),Box<dyn std::error::Error>>
{
	board.yk_serial_number = board_config
		.get("serial")
		.ok_or_else(|| return ConfigParsingError::new("No serial number found"))?
		.as_str()
		.ok_or_else(|| return ConfigParsingError::new("Serial number was not a string"))?
		.to_owned();

	board.yk_port_number = board_config
		.get("port")
		.ok_or_else(|| return ConfigParsingError::new("No port number found"))?
		.as_str()
		.ok_or_else(|| return ConfigParsingError::new("Port number was not a string"))?
		.to_owned();

	board.power_source = board_config
		.get("type")
		.ok_or_else(|| return ConfigParsingError::new("No type found"))?
		.as_str()
		.ok_or_else(|| return ConfigParsingError::new("Type was not a string"))?
		.to_owned();

	return Ok(());
}

pub fn get_all_boards_from_config(input_file: String)
-> Result<Vec<Board>,Box<dyn std::error::Error>>
{
	let boards: Vec<Board> = Vec::new();
	let contents = fs::read_to_string(input_file)?;

	let config: Value = serde_yaml::from_str(&contents)?;

	let board_configs = config
		.get("boards")
		.ok_or_else(|| return ConfigParsingError::new("No boards found"))?
		.as_mapping();
	
	let board_configs_iter = board_configs
		.unwrap()
		.iter();

	for board_config in board_configs_iter {
		let mut board = Board::default();
		board.name = board_config.0
			.as_str()
			.ok_or_else(|| return ConfigParsingError::new("name was not a string"))?
			.to_string();
		populate_board(&mut board, board_config.1.to_owned())?;
	}

	return Ok(boards.clone());
}

pub fn get_board_from_config(board_name: String, input_file: String)
-> Result<Board, Box<dyn std::error::Error>>
{
	let contents = fs::read_to_string(input_file)?;

	let config: Value = serde_yaml::from_str(&contents)?;

	let board_config = config
		.get("boards")
		.ok_or_else(|| return ConfigParsingError::new("No boards found"))?
		.get(board_name.clone())
		.ok_or_else(|| return ConfigParsingError::new("Requested board not found"))?;

	let mut board = Board::default();
	board.name = board_name.clone();
	populate_board(&mut board, board_config.to_owned())?;

	return Ok(board.clone());
}

