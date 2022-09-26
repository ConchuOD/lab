// SPDX-License-Identifier: LGPL-3.0-only

#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use clap::Parser;
use crossterm::{
	event::{self, Event, KeyCode},
	terminal::{disable_raw_mode, enable_raw_mode},
};
use std::time::Duration;
use std::io;
use tui::{
	backend::CrosstermBackend,
	Frame,
	layout::{Constraint, Direction, Layout, Rect},
	style::{Color, Modifier, Style},
	text::Span, Terminal,
	widgets::{Block, Borders, Paragraph, Cell, Row, Table, List, ListItem, ListState},
	widgets::canvas::{Canvas, Rectangle},
};

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
	#[clap(short, long, default_value = "interactive")]
	function: String,
}

mod ykcmd;
mod boards;
use boards::Status;

#[derive(Clone)]
struct StatefulList<T> {
	state: ListState,
	items: Vec<T>,
	// StatefulList is under an MIT License at:
	// https://github.com/fdehau/tui-rs/blob/master/examples/list.rs
}

impl<T> Default for StatefulList<T> {
	fn default() -> StatefulList<T> {
		return StatefulList {
			state: ListState::default(),
			items: Vec::new()
		}
	}
}

impl<T> StatefulList<T> {
	fn next(&mut self)
	{
		let i = match self.state.selected() {
			Some(i) => {
				if i >= self.items.len() - 1 {
					0
				} else {
					i + 1
				}
			}
			None => 0,
		};
		self.state.select(Some(i));
	}

	fn previous(&mut self)
	{
		let i = match self.state.selected() {
			Some(i) => {
				if i == 0 {
					self.items.len() - 1
				} else {
					i - 1
				}
			}
			None => 0,
		};
		self.state.select(Some(i));
	}

	fn deselect(&mut self)
	{
		self.state.select(None);
	}
}

#[derive(Clone)]
struct UIState<'a> {
	boards: StatefulList<&'a boards::Board>
}

impl<'a> UIState<'a> {
	fn new() -> UIState<'a> {
		return UIState {
			boards: StatefulList::default()
		}
	}

	fn selected (self) -> Option<&'a boards::Board>
	{
		return Some(self.boards.items[self.boards.state.selected()?])
	}
}

fn run_interactively(input_file: String) -> Result<(), Box<dyn std::error::Error>>
{
	// open the config file to figure out what boards we have
	let boards = boards::get_all_boards_from_config(input_file.clone())?;
	let mut ui_state = UIState::new();
	// set up the ui in a terminal
	let stdout = io::stdout();
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;
	let mut input: String = String::new();

	terminal.clear()?;
	enable_raw_mode()?;
	terminal.clear()?;

	for board in boards.iter() {
		ui_state.boards.items.push(&*board);
	}

	loop {
	
		let items: Vec<ListItem> = ui_state
			.boards.items.iter()
			.map(|i| {
				let mut colour = Color::Gray;
				let status = i.is_powered();
				if status.is_ok()
				{
					if status.unwrap() {
						colour = Color::Blue;
					}
				}

				return ListItem::new(i.name.clone())
					.style(
						Style::default()
							.fg(colour)
					)
			})
			.collect();
	
		let items = List::new(items)
			.block(Block::default().borders(Borders::ALL).title("List"))
			.highlight_style(
				Style::default()
					.bg(Color::White)
					.add_modifier(Modifier::BOLD),
			)
			.highlight_symbol(">> ");

		terminal.draw(|frame| {
			let entire_window =
				Layout::default()
				.direction(Direction::Horizontal)
				.constraints(
					[
						Constraint::Percentage(85),
						Constraint::Percentage(15),
					]
					.as_ref(),
				)
				.split(frame.size());

			frame.render_stateful_widget(items.clone(), entire_window[0],
						     &mut ui_state.boards.state);
		})?;

		if event::poll(Duration::from_millis(30))? {
			if let Event::Key(key) = event::read()? {
				match key.code {
					KeyCode::Char('q') => {
						terminal.clear()?;
						if disable_raw_mode().is_err() {
							panic!("Failed to clean up terminal");
						}
						break;
					}
					KeyCode::Left => ui_state.boards.deselect(),
					KeyCode::Down => ui_state.boards.next(),
					KeyCode::Up => ui_state.boards.previous(),
					KeyCode::Enter => {
						let selected = ui_state.clone().selected();

						if selected.is_none() {
							continue;
						}

						ykcmd::reboot_board(selected.unwrap()
								    .name.to_string(),
								    input_file.clone())?;
					}
					_ => {}
				}
				if false {
				match key.code {
					KeyCode::Char(c) => {
						input.push(c);
					}
					KeyCode::Backspace => {
						input.pop();
					}
					KeyCode::Esc => {
						terminal.clear()?;
						if disable_raw_mode().is_err() {
							panic!("Failed to clean up terminal");
						}
						break;
					}
					KeyCode::Enter => {
						//messages.push(input.drain(..).collect());
					}
					_ => {}
				}}
			}
		}

		//let input = handle_messages(&mut messages);
		//if let Some(command) = input.clone() {
		//}
		//next_state = states::get_next_state(next_state, &mut board, input);
	}

	// allow selection of a given board w/ directional keys
	// allow issuance of on/off/reboot/goodnight commands
	return Ok(());
}

fn main() -> Result<(),Box<dyn std::error::Error>> {
	let args = Args::parse();
	let input_file = args.config;
	let board = args.board;

	match args.function.as_str() {
		"off" => return ykcmd::turn_off_board(board, input_file),
		"on" | "reboot" => return ykcmd::reboot_board(board, input_file),
		"goodnight" => return ykcmd::goodnight(input_file),
		"interactive" => return run_interactively(input_file),
		_ => return Err(Box::new(ykcmd::YkmdError::new("Invalid function"))),
	}
}

