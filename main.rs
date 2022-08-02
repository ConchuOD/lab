// SPDX-License-Identifier: LGPL-3.0-only

#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use clap::Parser;
use serde_yaml::Value;
use std::{fs,process::Command};
use std::{fmt, thread, time};

/// lab
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
	/// input yaml config file
	#[clap(short, long, default_value = "config.yaml")]
	config: String,

	/// board to operate on
	#[clap(short, long, default_value = "icicle")]
	board: String,
	
	/// command (reboot, off, goodnight)
	#[clap(short, long, default_value = "off")]
	function: String,
}

#[derive(Debug)]
struct ConfigParsingError {
	details: String
}

impl ConfigParsingError {
	fn new(msg: &str) -> ConfigParsingError {
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

#[derive(Debug)]
struct YkmdError {
	details: String
}

impl YkmdError {
	fn new(msg: &str) -> YkmdError {
		return YkmdError{details: msg.to_string()}
	}
}

impl fmt::Display for YkmdError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		return write!(f, "ykushcmd failed: {}", self.details)
	}
}

impl std::error::Error for YkmdError {
	fn description(&self) -> &str {
		return &self.details
	}
}

fn get_board_from_config(board: String, input_file: String, serial: &mut String,
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

fn power(board: String, serial: String, port: String, direction: String, command: String)
-> Result<(), Box<dyn std::error::Error>>
{
	let output = Command::new("sh")
		.arg("-c")
		.arg(
			&format!("{} -s {} -{} {}",
				command,
				serial,
				direction.chars().next().unwrap(),
				port)
		)
		.output()
		.expect("failed to execute process");

	if !output.status.success() {
		return Err(Box::new(YkmdError::new("failed to power direction")));
	}

	println!("{} attached to {}@{} powered {}.", board, serial, port, direction);
	return Ok(())
}

fn format_command(yk_board_type: String, command: &mut String)
-> Result<(), Box<dyn std::error::Error>>
{
	match yk_board_type.as_str() {
		"usb" => *command = "ykushcmd ykush".to_string(),
		"relay" => *command = "ykurcmd".to_string(),
		_ => return Err(Box::new(YkmdError::new("Unsupported yk board type"))),
	}

	return Ok(())
}

fn reboot_board(board: String, input_file: String) -> Result<(), Box<dyn std::error::Error>>
{
	let mut power_source: String = String::new();
	let mut command: String = String::new();
	let mut serial: String = String::new();
	let mut port: String = String::new();

	get_board_from_config(board.clone(), input_file, &mut serial, &mut port,
			      &mut power_source)?;

	format_command(power_source, &mut command)?;

	let output = Command::new("sh")
		.arg("-c")
		.arg(format!("{} -l ", command))
		.output()
		.expect("failed to execute process");

	let stdout = match String::from_utf8(output.stdout) {
		Ok(v) => v,
		Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
	};
	if !stdout.contains(&serial) {
		return Err(Box::new(YkmdError::new(&format!(
			"board with serial {} not found", serial))))
	}
	
	println!("{} attached to {}@{}", board, serial, port);
	power(board.clone(), serial.clone(), port.clone(), "down".to_string(), command.clone())?;
	thread::sleep(time::Duration::from_millis(1000));
	power(board, serial, port, "up".to_string(), command)?;

	return Ok(())
}

fn turn_off_board(board: String, input_file: String) -> Result<(), Box<dyn std::error::Error>>
{
	let mut power_source: String = String::new();
	let mut command: String = String::new();
	let mut serial: String = String::new();
	let mut port: String = String::new();

	get_board_from_config(board.clone(), input_file, &mut serial, &mut port,
			      &mut power_source)?;

	format_command(power_source, &mut command)?;

	let output = Command::new("sh")
		.arg("-c")
		.arg(format!("{} -l ", command))
		.output()
		.expect("failed to execute process");

	let stdout = match String::from_utf8(output.stdout) {
		Ok(v) => v,
		Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
	};
	if !stdout.contains(&serial) {
		return Err(Box::new(YkmdError::new(&format!(
			"board with serial {} not found", serial))))
	}
	
	println!("{} attached to {}@{}", board, serial, port);
	power(board, serial.clone(), port.clone(), "down".to_string(), command)?;

	return Ok(())
}

fn goodnight(input_file: String) -> Result<(), Box<dyn std::error::Error>>
{
	let contents = fs::read_to_string(input_file.clone())?;

	let config: Value = serde_yaml::from_str(&contents)?;

	let board_configs = config
		.get("boards")
		.ok_or_else(|| return ConfigParsingError::new("No boards found"))?
		.as_mapping();
	
	if board_configs.is_none() {
		return Err(Box::new(YkmdError::new("No boards found")))
	}
	for board in board_configs.unwrap().iter() {
		let board_name = board.0.as_str().unwrap();
		println!("Trying to power down {}", board_name);
		let _ = turn_off_board(String::from(board_name), input_file.clone());
	}
	
	return Ok(())
}

fn main() -> Result<(),Box<dyn std::error::Error>> {
	let args = Args::parse();
	let input_file = args.config;
	let board = args.board;

	match args.function.as_str() {
		"off" => return turn_off_board(board, input_file),
		"on" | "reboot" => return reboot_board(board, input_file),
		"goodnight" => return goodnight(input_file),
		_ => return Err(Box::new(YkmdError::new("Invalid function"))),
	}
}