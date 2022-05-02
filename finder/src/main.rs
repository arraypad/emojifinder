mod ui;

use clipboard::{ClipboardContext, ClipboardProvider};
use failure::Error;
use locale_config::LanguageRange;

use emojifinder_core::{error::Error as EmojiError, Index};
use ui::Ui;

pub const NOTICE: &str = "Emojifinder
Copyright 2022 Bobweb Ltd. https://github.com/arraypad/emojifinder

This application contains:
* SVG assets from the NotoColorEmoji font (copyright Google Inc.) distributed under the Apache License, Version 2.0. See: https://github.com/googlefonts/noto-emoji/blob/master/LICENSE
* Annotations from the Unicode Common Locale Data Repository (copyright Unicode, Inc) distributed under the Unicode Terms of Use. See: https://www.unicode.org/copyright.html";

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
	let index = Index::from_bytes(include_bytes!("index.bin"))?;

	let config = Config {
		lang: find_language(&index)?,
	};

	let mut app = Ui::new(index, config)?;
	app.run()
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
