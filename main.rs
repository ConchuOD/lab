// SPDX-License-Identifier: LGPL-3.0-only

#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use clap::Parser;
use serde_yaml::Value;
use std::{fs,process::Command};

fn get_board_from_config(board: String, input_file: String, serial: &mut String, port: &mut String)
-> Result<(), Box<dyn std::error::Error>>
{
	let contents = fs::read_to_string(input_file)?;

	let config: Value = serde_yaml::from_str(&contents)?;

	let board_config = config
		.get("boards")
		.ok_or_else(|| return std::fmt::Error)?
		.get(board)
		.ok_or_else(|| return std::fmt::Error)?;

	*serial = board_config
		.get("serial")
		.ok_or_else(|| return std::fmt::Error)?
		.as_str()
		.ok_or_else(|| return std::fmt::Error)?
		.to_owned();

	*port = board_config
		.get("port")
		.ok_or_else(|| return std::fmt::Error)?
		.as_str()
		.ok_or_else(|| return std::fmt::Error)?
		.to_owned();

	return Ok(());
}

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
	
	/// shut down all boards
	#[clap(long)]
	goodnight: bool,

	/// boot or reboot a specific board
	#[clap(short, long)]
	reboot: bool,
}

fn main() -> Result<(),Box<dyn std::error::Error>> {
	let args = Args::parse();
	let input_file = args.config;
	let board = args.board;
	let mut serial: String = String::new();
	let mut port: String = String::new();
	
	get_board_from_config(board.clone(), input_file, &mut serial, &mut port)?;

	let output = Command::new("sh")
		.arg("-c")
		.arg("ykushcmd ykush -l ")
		.output()
		.expect("failed to execute process");

	let stdout = match String::from_utf8(output.stdout) {
		Ok(v) => v,
		Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
	};
	if !stdout.clone().contains(&serial) {
		return Err(Box::new(std::fmt::Error))
	}
	
	println!("{} attached to {}@{}", board, serial, port);

	return Ok(())
}