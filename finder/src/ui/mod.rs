mod tui;

use failure::Error;

use crate::Config;
use emojifinder_core::{error::Error as EmojiError, Emoji, Index};
use self::tui::Tui;

pub trait Ui {
	fn run(&mut self) -> Result<(), Error>;
}

pub fn load(index: Index, config: Config) -> Result<Box<dyn Ui>, Error> {
	Ok(Box::new(Tui::new(index, config)))
}
