// SPDX-License-Identifier: LGPL-3.0-only

#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use crossterm::{
	event::{self, Event, KeyCode},
	terminal::{disable_raw_mode, enable_raw_mode},
};
use std::time::Duration;
use std::io;
use tui::{
	backend::CrosstermBackend,
	layout::{Constraint, Direction, Layout},
	style::{Color, Modifier, Style},
	Terminal,
	widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::ykcmd;
use crate::boards;
use crate::boards::Status;
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

fn toggle_power_state(board: &boards::Board, input_file: String)
-> Result<(), Box<dyn std::error::Error>>
{
	if !board.is_powered()? {
		return ykcmd::turn_on_board(board.name.to_string(), input_file);
	}

	return ykcmd::turn_off_board(board.name.to_string(), input_file);
}


pub fn run_interactively(input_file: String) -> Result<(), Box<dyn std::error::Error>>
{
	let boards = boards::get_all_boards_from_config(input_file.clone())?;
	let mut ui_state = UIState::new();
	let stdout = io::stdout();
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

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
				if status.is_ok() && status.unwrap() {
					colour = Color::Blue;
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
						let _err = toggle_power_state(selected.unwrap(),
									      input_file.clone());
					}
					_ => {}
				}
			}
		}
	}

	return Ok(());
}


