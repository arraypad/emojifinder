mod ui;

use clipboard::{ClipboardContext, ClipboardProvider};
use failure::Error;

use emojifinder_core::{error::Error as EmojiError, Index};

pub struct Config {
	lang: &'static str,
}

fn main() {
	env_logger::init();
	match run() {
		Ok(_) => {}
		Err(e) => {
			eprintln!("Error: {}", e);
		}
	}
}

pub fn set_clipboard<S: AsRef<str>>(value: S) -> Result<(), Error> {
	let mut clip: ClipboardContext = match ClipboardProvider::new() {
		Ok(clip) => clip,
		Err(e) => {
			return Err(
				EmojiError::clipboard(format!("failed constructing provider: {:?}", e)).into(),
			)
		}
	};

	if let Err(e) = clip.set_contents(value.as_ref().to_string()) {
		return Err(EmojiError::clipboard(format!("failed writing to clipboard: {:?}", e)).into());
	}

	Ok(())
}

fn run() -> Result<(), Error> {
	let index = Index::from_bytes(include_bytes!("../data/index.bin"))?;

	let config = Config { lang: "en" };

	let mut app = ui::load(index, config)?;
	Ok(app.run()?)
}
