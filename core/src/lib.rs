pub mod error;

use serde_derive::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Emoji {
	codepoint: u32,
	value: String,
	name: HashMap<String, String>,
	keywords: HashMap<String, Vec<String>>,
	svg: String,
}
