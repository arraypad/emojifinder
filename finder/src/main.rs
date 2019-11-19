mod util;

use failure::Error;
use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Widget, Block, Borders, SelectableList};
use tui::layout::{Layout, Constraint, Direction};

use emojifinder_core::{Emoji, Index};
use util::event::{Event, Events};


fn main() {
	env_logger::init();
	match run() {
		Ok(_) => {},
		Err(e) => {
			eprintln!("Error: {}", e);
		},
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
	let items= app.index.items(lang);
	println!("Items: {:?}", items);

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

		terminal.draw(|mut f| {
			let chunks = Layout::default()
				.direction(Direction::Vertical)
				.margin(1)
				.constraints(
					[
						Constraint::Percentage(50),
						Constraint::Percentage(50),
					].as_ref()
				)
				.split(f.size());

			Block::default()
				.title("Preview: ")
				.borders(Borders::ALL)
				.render(&mut f, chunks[0]);

			SelectableList::default()
				.block(Block::default().borders(Borders::ALL).title(prompt.as_str()))
				.items(app.index.items(lang).as_slice())
				.select(app.selected)
				.style(Style::default().fg(Color::White))
				.highlight_style(Style::default().modifier(Modifier::ITALIC))
				.highlight_symbol(">")
				.render(&mut f, chunks[1]);
		})?;

		match events.next()? {
			Event::Input(input) => match input {
				Key::Esc => {
					break;
				},
				Key::Backspace => {
					app.query.truncate(app.query.len() - 1);
				},
				Key::Char(c) => {
					app.query += c.to_string().as_str();
				},
				_ => {},
			},
			Event::Tick => {
				// unused - could be used for non-input updates later
			},
		}
	}

	Ok(())
}
