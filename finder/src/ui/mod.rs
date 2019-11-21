mod tui;

use failure::Error;

use self::tui::Tui;
use crate::Config;
use emojifinder_core::Index;

pub trait Ui {
	fn run(&mut self) -> Result<(), Error>;
}

pub fn load(index: Index, config: Config) -> Result<Box<dyn Ui>, Error> {
	Ok(Box::new(Tui::new(index, config)))
}
