pub mod error;

use failure::Error;
use lz4::block::{compress, decompress, CompressionMode};
use rmp_serde::{encode::write_named as mp_to_writer, from_read as mp_from_reader};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Emoji {
	pub value: String,
	pub name: HashMap<String, String>,
	pub keywords: HashMap<String, Vec<String>>,
	pub svg: String,

	#[serde(skip)]
	pub rank: f32,
}

impl Emoji {
	pub fn name<'a, S: AsRef<str>>(&'a self, lang: S) -> &'a str {
		match self.name.get(lang.as_ref()) {
			Some(name) => name.as_str(),
			None => "Unknown",
		}
	}
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Index {
	pub emojis: Vec<Emoji>,
}

impl Index {
	pub fn from_bytes(bytes: &[u8]) -> Result<Index, Error> {
		let uncompressed: Vec<u8> = decompress(bytes, None)?;

		Ok(Index {
			emojis: mp_from_reader(&*uncompressed)?,
		})
	}

	pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
		let mut uncompressed: Vec<u8> = Vec::new();
		mp_to_writer(&mut uncompressed, &self.emojis)?;

		let compressed = compress(
			uncompressed.as_slice(),
			Some(CompressionMode::HIGHCOMPRESSION(9)),
			true,
		)?;

		Ok(std::fs::write(path, compressed)?)
	}

	pub fn items<S: AsRef<str>>(&self, lang: S) -> Vec<String> {
		self.emojis
			.iter()
			.map(|e| format!("{:?}: {}", e.value, e.name(lang.as_ref())))
			.collect()
	}

	pub fn search<S: AsRef<str>, T: AsRef<str>>(&mut self, lang: S, query: T) {
		let lang = lang.as_ref();
		for ref mut emoji in &mut self.emojis {
			emoji.rank = trigram::similarity(query.as_ref(), emoji.name(lang));
		}

		self.emojis
			.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap());
	}
}
