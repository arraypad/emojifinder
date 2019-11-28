mod ui;

use clipboard::{ClipboardContext, ClipboardProvider};
use failure::Error;
use locale_config::LanguageRange;

use emojifinder_core::{error::Error as EmojiError, Index};
use ui::Ui;

#[derive(Debug)]
pub struct Config {
	lang: String,
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

fn run() -> Result<(), Error> {
	println!("
Emojifinder is open source software, distributed under the MIT license, see:
	https://github.com/arraypad/emojifinder/blob/master/LICENSE.md

Emojifinder contains:
* SVG assets from Google's NotoColorEmoji project distributed under the Apache License, Version 2.0. See: https://github.com/arraypad/emojifinder/blob/master/LICENSE.noto-color-emoji
* Annotations from the Unicode Common Locale Data Repository distributed under the Unicode Terms of Use. See: https://github.com/arraypad/emojifinder/blob/master/LICENSE.unicode
");

	let index = Index::from_bytes(include_bytes!("../data/index.bin"))?;

	let config = Config {
		lang: find_language(&index)?,
	};

	let mut app = Ui::new(index, config)?;
	Ok(app.run()?)
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

fn find_language(index: &Index) -> Result<String, Error> {
	let index_langs: Vec<LanguageRange> = index
		.locale_codes
		.iter()
		.filter_map(|code| LanguageRange::from_unix(code).ok())
		.collect();

	let locale = locale_config::Locale::current();
	for lang in locale.tags_for("messages") {
		if index_langs.contains(&lang) {
			return Ok(format!("{}", lang));
		}
	}

	Ok("en".to_string())
}
