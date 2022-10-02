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
	layout::{Constraint, Direction, Layout, Rect},
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
	fn with_items(items: Vec<T>) -> StatefulList<T> {
		return StatefulList {
			state: ListState::default(),
			items,
		}
	}

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

type Action = fn(&boards::Board, String) -> Result<(), Box<dyn std::error::Error>>;

#[derive(Clone)]
struct UIState<'a> {
	boards: StatefulList<&'a boards::Board>,
	show_popup: bool,
	actions: StatefulList<
		(&'a str, Action)
	>,
	action_items: List<'a>,
}

impl<'a> UIState<'a> {
	fn new() -> UIState<'a> {
		return UIState {
			boards: StatefulList::default(),
			show_popup: false,
			actions: StatefulList::default(),
			action_items: List::new(Vec::new()),
		}
	}

	fn selected_board(self) -> Option<&'a boards::Board>
	{
		return Some(self.boards.items[self.boards.state.selected()?])
	}

	fn selected_action(self) -> Option<Action>
	{
		let selected_action = self.actions.state.selected()?;
		return Some(self.actions.items[selected_action].1);
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

fn create_centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
	let popup_layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints(
			[
				Constraint::Percentage((100 - percent_y) / 2),
				Constraint::Percentage(percent_y),
				Constraint::Percentage((100 - percent_y) / 2),
			]
			.as_ref(),
		)
		.split(r);

	return Layout::default()
		.direction(Direction::Horizontal)
		.constraints(
			[
				Constraint::Percentage((100 - percent_x) / 2),
				Constraint::Percentage(percent_x),
				Constraint::Percentage((100 - percent_x) / 2),
			]
			.as_ref(),
		)
		.split(popup_layout[1])[1]
}

fn action_menu(ui_state: &mut UIState)
{
	let selected_board = ui_state.clone().selected_board();

	if selected_board.is_none() {
		return;
	}

	ui_state.actions = StatefulList::with_items(vec![
			("Switch power", toggle_power_state),
			("Reboot", toggle_power_state),
			("Boot test", toggle_power_state),
		]);

	let action_items: Vec<ListItem> = ui_state.actions.items.iter()
		.map(|i| {
			return ListItem::new(i.0)
				.style(
					Style::default().fg(Color::Red)
				)
		})
		.collect();

	ui_state.action_items = List::new(action_items)
		.block(Block::default().borders(Borders::ALL).title("List"))
		.highlight_style(
			Style::default()
				.bg(Color::White)
				.add_modifier(Modifier::BOLD),
		)
		.highlight_symbol(">> ");

	ui_state.show_popup = true;
}

fn perform_action(ui_state: UIState, input_file: String) -> Result<(), Box<dyn std::error::Error>>
{
	let board = ui_state.clone().selected_board();

	if board.is_none() {
		return Ok(())
	}

	let action = ui_state.clone().selected_action();

	if action.is_none() {
		toggle_power_state(board.unwrap(), input_file)?;
	} else {
		action.unwrap()(board.unwrap(), input_file)?;
	}

	return Ok(());
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
						Style::default().fg(colour)
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

		let entire_window =
			Layout::default()
			.direction(Direction::Horizontal)
			.constraints(
				[
					Constraint::Percentage(25),
					Constraint::Percentage(75),
				]
				.as_ref(),
			);

		let mut useable_window: Vec<Rect> = Vec::new();

		if event::poll(Duration::from_millis(30))? {
			/* don't ask me how much I hate this */
			if !ui_state.show_popup {
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
						KeyCode::Enter => action_menu(&mut ui_state),
						_ => {}
					}
				}
			} else if let Event::Key(key) = event::read()? {
				match key.code {
					KeyCode::Char('q') => {
						terminal.clear()?;
						if disable_raw_mode().is_err() {
							panic!("Failed to clean up terminal");
						}
						break;
					}
					KeyCode::Left => ui_state.actions.deselect(),
					KeyCode::Down => ui_state.actions.next(),
					KeyCode::Up => ui_state.actions.previous(),
					KeyCode::Enter => {
						let _err = perform_action(ui_state.clone(),
									  input_file.clone());
					},
					_ => {}
				}
			}
		}

		terminal.draw(|frame| {
			useable_window = entire_window.split(frame.size());

			frame.render_stateful_widget(items.clone(), useable_window[0],
						     &mut ui_state.boards.state);
			if ui_state.show_popup {

				let popup = create_centered_rect(80, 80, useable_window[0]);

				frame.render_widget(tui::widgets::Clear, useable_window[0]);
				frame.render_stateful_widget(ui_state.action_items.clone(), popup,
							     &mut ui_state.actions.state);
			}

		})?;

	}

	return Ok(());
}

