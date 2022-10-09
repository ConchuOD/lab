// SPDX-License-Identifier: LGPL-3.0-only

#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use serde_yaml::Value;
use std::{fs, fmt};
use crate::ykcmd;

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
#[derive(Debug)]
pub struct Board {
	pub name: String,
	pub yk_serial_number: String,
	pub yk_port_number: String,
	pub power_source: String,
	pub powered: bool,
	pub primary_uart: String,
}

impl Default for Board {
	fn default() -> Board
	{
		return Board {
			name: "n/a".to_string(),
			yk_serial_number: "n/a".to_string(),
			yk_port_number: "n/a".to_string(),
			power_source: "n/a".to_string(),
			powered: false,
			primary_uart: "n/a".to_string(),
		}
	}
}

pub trait Status {
	fn is_powered(&self) -> Result<bool, Box<dyn std::error::Error>>;
}

impl Status for Board {
	fn is_powered(&self) -> Result<bool, Box<dyn std::error::Error>>
	{
		return ykcmd::is_powered(self)
	}
}

pub trait Ops {
	fn power_off(&self) -> Result<(), Box<dyn std::error::Error>>;
	fn power_on(&self) -> Result<(), Box<dyn std::error::Error>>;
	fn reboot(&self) -> Result<(), Box<dyn std::error::Error>>;
	fn expect_boot(&self) -> Result<(), Box<dyn std::error::Error>>;
	fn expect_shutdown(&self) -> Result<(), Box<dyn std::error::Error>>;
}

impl Ops for Board {
	fn power_off(&self) -> Result<(), Box<dyn std::error::Error>>
	{
		return ykcmd::power_off(self.name.clone(),
					self.yk_serial_number.clone(),
					self.yk_port_number.clone(),
					self.power_source.clone());
	}

	fn power_on(&self) -> Result<(), Box<dyn std::error::Error>>
	{
		return ykcmd::power_on(self.name.clone(),
				       self.yk_serial_number.clone(),
				       self.yk_port_number.clone(),
				       self.power_source.clone());
	}

	fn reboot(&self) -> Result<(), Box<dyn std::error::Error>>
	{
		return ykcmd::reboot(self.name.clone(),
				     self.yk_serial_number.clone(),
				     self.yk_port_number.clone(),
				     self.power_source.clone());
	}

	fn expect_boot(&self) -> Result<(), Box<dyn std::error::Error>>
	{
		let uart = &self.primary_uart;
		let port = serialport::new(uart, 115_200).open()?;
		let read_port = port.try_clone()?;
		let write_port = port.try_clone()?;

		let mut stream = rexpect::session::spawn_stream(read_port, write_port, Some(120000));

		dbg!("expecting on uart with path {}", self.primary_uart.clone());
		stream.exp_regex(".*U-Boot.*")?;
		dbg!("Found U-Boot!}");
		stream.exp_regex(".*Linux version.*")?;
		dbg!("Found Linux!}");
		stream.exp_regex(".*init.*")?;
		dbg!("Found init!}");
		stream.exp_regex(".*login: .*")?;
		stream.send_line("root")?;
		stream.exp_regex(".*assword: ")?;
		dbg!("Waiting for password!");
		stream.send_line("fedora_rocks!")?;
		stream.exp_regex(".*#.*")?;
		dbg!("Logged in!");

		return Ok(())
	}

	fn expect_shutdown(&self) -> Result<(), Box<dyn std::error::Error>>
	{
		let uart = &self.primary_uart;
		let port = serialport::new(uart, 115_200).open()?;
		let read_port = port.try_clone()?;
		let write_port = port.try_clone()?;

		let mut stream = rexpect::session::spawn_stream(read_port, write_port, Some(120000));

		dbg!("expecting on uart with path {}", self.primary_uart.clone());
		stream.send_line("poweroff")?;
		dbg!("Powering off!");
		stream.exp_regex(".*reboot: System halted.*")?;
		dbg!("Shut down!");

		return Ok(())
	}

}

fn populate_board(board: &mut Board, board_config: Value)
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

	let _who_cares = populate_uart(board, board_config);

	return Ok(());
}

fn populate_uart(board: &mut Board, board_config: Value)
-> Result<(),Box<dyn std::error::Error>>
{
	board.yk_serial_number = board_config
		.get("serial")
		.ok_or_else(|| return ConfigParsingError::new("No serial number found"))?
		.as_str()
		.ok_or_else(|| return ConfigParsingError::new("Serial number was not a string"))?
		.to_owned();
	
	let uart_config = board_config
		.get("uart")
		.ok_or_else(|| return ConfigParsingError::new("No uart config found"))?;

	let uart_by_id = uart_config
		.get("pattern")
		.ok_or_else(|| return ConfigParsingError::new("No uart pattern found"))?
		.as_str()
		.ok_or_else(|| return ConfigParsingError::new("uart pattern was not a string"))?
		.to_owned();

	let uart_primary = uart_config
		.get("primary")
		.ok_or_else(|| return ConfigParsingError::new("No uart primary found"))?
		.as_str()
		.ok_or_else(|| return ConfigParsingError::new("uart primary was not a string"))?
		.to_owned();
	
	board.primary_uart = format!("/dev/serial/by-id/{}-{}", uart_by_id, uart_primary);
	dbg!("uart found with path {}", board.primary_uart.clone());

	return Ok(());
}

pub fn get_all_boards_from_config(input_file: String)
-> Result<Vec<Board>,Box<dyn std::error::Error>>
{
	let mut boards: Vec<Board> = Vec::new();
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
		let mut board = Board {
			name: board_config.0
				.as_str()
				.ok_or_else(|| return ConfigParsingError::new("name was not a string"))?
				.to_string(),
			..Default::default()
		};
		populate_board(&mut board, board_config.1.to_owned())?;
		boards.push(board);
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

	let mut board = Board {
		name: board_name,
		..Default::default()
	};
	populate_board(&mut board, board_config.to_owned())?;

	return Ok(board.clone());
}

