mod util;
mod viewer;

use failure::Error;
use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, SelectableList, Widget};
use tui::Terminal;

use emojifinder_core::{Emoji, Index};
use util::event::{Event, Events};
use viewer::Viewer;

fn main() {
	env_logger::init();
	match run() {
		Ok(_) => {}
		Err(e) => {
			eprintln!("Error: {}", e);
		}
	}
}

struct App {
	pub index: Index,
	pub selected: Option<usize>,
	pub query: String,
}

impl App {
	pub fn new() -> Result<App, Error> {
		Ok(App {
			index: Index::from_bytes(include_bytes!("../data/index.bin"))?,
			selected: None,
			query: String::new(),
		})
	}
}

fn run() -> Result<(), Error> {
	let lang = "en";
	let mut app = App::new()?;

	let stdout = std::io::stdout().into_raw_mode()?;
	let stdout = AlternateScreen::from(stdout);
	let backend = TermionBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;
	terminal.hide_cursor()?;

	let events = Events::new();

	loop {
		let mut prompt = String::from("Search: ");
		if app.query.is_empty() {
			prompt += "[Start typing to find an Emoji]";
		} else {
			prompt += app.query.as_str();
		};

		let items = app.index.items(lang);

		let svg = match app.selected {
			Some(i) => Some(app.index.emojis[i].svg.clone()),
			None => None,
		};

		terminal.draw(|mut f| {
			let chunks = Layout::default()
				.direction(Direction::Vertical)
				.margin(1)
				.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
				.split(f.size());

			Viewer::new(svg)
				.block(Block::default().borders(Borders::ALL).title("Preview: "))
				.render(&mut f, chunks[0]);

			SelectableList::default()
				.block(
					Block::default()
						.borders(Borders::ALL)
						.title(prompt.as_str()),
				)
				.items(items.as_slice())
				.select(app.selected)
				.style(Style::default().fg(Color::White))
				.highlight_style(Style::default().modifier(Modifier::ITALIC))
				.highlight_symbol(">")
				.render(&mut f, chunks[1]);
		})?;

		match events.next()? {
			Event::Input(input) => match input {
				Key::Down => {
					app.selected = if let Some(selected) = app.selected {
						if selected >= items.len() - 1 {
							Some(0)
						} else {
							Some(selected + 1)
						}
					} else {
						Some(0)
					}
				}
				Key::Up => {
					app.selected = if let Some(selected) = app.selected {
						if selected > 0 {
							Some(selected - 1)
						} else {
							Some(items.len() - 1)
						}
					} else {
						Some(0)
					}
				}
				Key::Esc => {
					break;
				}
				Key::Backspace => {
					app.query.truncate(app.query.len() - 1);
				}
				Key::Char(c) => {
					app.query += c.to_string().as_str();
				}
				_ => {}
			},
			Event::Tick => {
				// unused - could be used for non-input updates later
			}
		}
	}

	Ok(())
}
