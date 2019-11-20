mod event;
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

use crate::{set_clipboard, Config};
use emojifinder_core::Index;
use self::event::{Event, Events};
use self::viewer::{ColorMode, Viewer};
use super::Ui;

pub struct Tui {
	index: Index,
	config: Config,
	flash: bool,
	query: String,
	selected: usize,
}

impl Tui {
	pub fn new(index: Index, config: Config) -> Self {
		Tui {
			index,
			config,
			flash: false,
			query: String::new(),
			selected: 0,
		}
	}
}

impl Ui for Tui {
	fn run(&mut self) -> Result<(), Error> {
		let stdout = std::io::stdout().into_raw_mode()?;
		let stdout = AlternateScreen::from(stdout);
		let backend = TermionBackend::new(stdout);
		let mut terminal = Terminal::new(backend)?;
		terminal.hide_cursor()?;

		let events = Events::new();

		loop {
			let mut prompt = String::from("Search: ");
			if self.query.is_empty() {
				prompt += "[Start typing to find an Emoji]";
			} else {
				prompt += self.query.as_str();
				self.index.search(self.config.lang, self.query.as_str());
			};

			let items = self.index.items(self.config.lang);

			let svg = self.index.emojis[self.selected].svg.clone();

			let style = if self.flash {
				self.flash = false;
				Style::default().bg(Color::White).fg(Color::Black)
			} else {
				Style::default().bg(Color::Black).fg(Color::White)
			};

			terminal.draw(|mut f| {
				let chunks = Layout::default()
					.direction(Direction::Vertical)
					.margin(1)
					.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
					.split(f.size());

				Viewer::new(Some(svg))
					.color_mode(ColorMode::Rgb)
					.block(
						Block::default()
							.borders(Borders::ALL)
							.title("Preview: ")
							.style(style),
					)
					.style(style)
					.render(&mut f, chunks[0]);

				SelectableList::default()
					.block(
						Block::default()
							.borders(Borders::ALL)
							.title(prompt.as_str())
							.style(style),
					)
					.items(items.as_slice())
					.select(Some(self.selected))
					.style(style)
					.highlight_style(Style::default().modifier(Modifier::ITALIC))
					.highlight_symbol(">")
					.render(&mut f, chunks[1]);
			})?;

			match events.next()? {
				Event::Input(input) => match input {
					Key::Down => {
						self.selected = if self.selected >= items.len() - 1 {
							0
						} else {
							self.selected + 1
						}
					}
					Key::Up => {
						self.selected = if self.selected > 0 {
							self.selected - 1
						} else {
							items.len() - 1
						}
					}
					Key::Esc => {
						break;
					}
					Key::Backspace => {
						self.query.truncate(self.query.len() - 1);
						self.selected = 0;
					}
					Key::Char(c) => {
						if c == '\n' {
							self.flash = true;
							set_clipboard(&self.index.emojis[self.selected].value)?;
						} else {
							self.query += c.to_string().as_str();
							self.selected = 0;
						}
					}
					_ => {}
				},
				Event::Tick => {
					// currently only used to reset flash and handle resizing
				}
			}
		}

		Ok(())
	}
}
