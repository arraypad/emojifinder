mod tui;

use failure::Error;

use self::tui::Tui;
use crate::Config;
use emojifinder_core::Index;

pub fn load(index: Index, config: Config) -> Result<Tui, Error> {
	Ok(Tui::new(index, config)?)
}
