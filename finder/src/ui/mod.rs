mod event;

use failure::Error;
use termion::event::Key;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, SelectableList, Widget};
use tui::Terminal;
use tui_image::{ColorMode, Image};

use self::event::{Event, Events};
use crate::{set_clipboard, Config};
use emojifinder_core::Index;

pub struct Ui {
	terminal: Terminal<TermionBackend<AlternateScreen<RawTerminal<std::io::Stdout>>>>,
	events: Events,
	index: Index,
	config: Config,
	flash: bool,
	query: String,
	last_query: String,
	selected: usize,
}

impl Ui {
	pub fn new(index: Index, config: Config) -> Result<Self, Error> {
		let stdout = std::io::stdout().into_raw_mode()?;
		let stdout = AlternateScreen::from(stdout);
		let backend = TermionBackend::new(stdout);
		let mut terminal = Terminal::new(backend)?;
		terminal.hide_cursor()?;

		Ok(Ui {
			terminal,
			events: Events::new(),
			index,
			config,
			flash: false,
			query: String::new(),
			last_query: String::new(),
			selected: 0,
		})
	}

	pub fn run(&mut self) -> Result<(), Error> {
		Ok(loop {
			self.draw()?;

			match self.events.next()? {
				Event::Input(input) => match input {
					Key::Down => {
						if self.selected < self.index.emojis.len() - 1 {
							self.selected += 1;
						}
					}
					Key::Up => {
						if self.selected > 0 {
							self.selected -= 1;
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
		})
	}

	fn draw(&mut self) -> Result<(), Error> {
		let mut prompt = String::from("Search: ");
		if self.query.is_empty() {
			prompt += "[Start typing to find an Emoji]";
		} else {
			prompt += self.query.as_str();
			if self.query != self.last_query {
				self.index.search(&self.config.lang, self.query.as_str());
				self.last_query = self.query.clone();
			}
		};

		let items = self.index.items(&self.config.lang);
		let emoji = self.index.emojis[self.selected].clone();
		let selected = self.selected;

		let style = if self.flash {
			self.flash = false;
			Style::default().bg(Color::White).fg(Color::Black)
		} else {
			Style::default().bg(Color::Black).fg(Color::White)
		};

		Ok(self.terminal.draw(|mut f| {
			let chunks = Layout::default()
				.direction(Direction::Vertical)
				.margin(1)
				.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
				.split(f.size());

			Image::with_img_fn(move |w, h| emoji.get_image(w, h))
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
				.select(Some(selected))
				.style(style)
				.highlight_style(Style::default().modifier(Modifier::ITALIC))
				.highlight_symbol(">")
				.render(&mut f, chunks[1]);
		})?)
	}
}
