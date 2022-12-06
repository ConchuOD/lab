// SPDX-License-Identifier: LGPL-3.0-only

#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use clap::Parser;

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
	
	/// command (reset, off, goodnight)
	#[clap(short, long, default_value = "interactive")]
	function: String,
}

mod ykcmd;
mod boards;
mod ui;

fn main() -> Result<(),Box<dyn std::error::Error>> {
	let args = Args::parse();
	let input_file = args.config;
	let board = args.board;
	stderrlog::new()
		.module(module_path!())
		.init()
		.unwrap();

	match args.function.as_str() {
		"off" => return ykcmd::power_off_board(board, input_file),
		"on" => return ykcmd::power_on_board(board, input_file),
		"reset" => return ykcmd::reboot_board(board, input_file),
		"goodnight" => return ykcmd::goodnight(input_file),
		"interactive" => return ui::run_interactively(input_file),
		_ => return Err(Box::new(ykcmd::YkmdError::new("Invalid function"))),
	}
}

