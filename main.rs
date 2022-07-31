// SPDX-License-Identifier: LGPL-3.0-only

#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use clap::Parser;
use serde_yaml::Value;
use std::{fs,process};

fn get_board_from_config(board: String, input_file: String, serial: &mut String, port: &mut String)
-> Result<(), Box<dyn std::error::Error>>
{
	let contents = fs::read_to_string(input_file)?;

	let config: Value = serde_yaml::from_str(&contents)?;

	let board_config = config["boards"][board].clone();
	if board_config == serde_yaml::Value::Null {
		return Err(Box::new(std::fmt::Error))
	}

	let serial_temp = board_config.get("serial");
	if serial_temp.is_none() {
		return Err(Box::new(std::fmt::Error))
	}

	if let Some(x) = serial_temp.unwrap().as_str() {
		*serial = x.to_owned();
	} else {
		return Err(Box::new(std::fmt::Error))
	}

	let port_temp = board_config.get("port");
	if port_temp.is_none() {
		return Err(Box::new(std::fmt::Error))
	}

	if let Some(x) = port_temp.unwrap().as_str() {
		*port = x.to_owned();
	} else {
		return Err(Box::new(std::fmt::Error))
	}
	
	return Ok(())
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
}
fn main() -> Result<(),Box<dyn std::error::Error>> {
	let args = Args::parse();
	let input_file = args.config;
	let board = args.board;
	let mut serial: String = String::new();
	let mut port: String = String::new();
	
	get_board_from_config(board.clone(), input_file, &mut serial, &mut port)?;

	println!("{} attached to {}@{}", board, serial, port);
	return Ok(())
}