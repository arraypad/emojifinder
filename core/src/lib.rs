pub mod error;

use failure::Error;
use image::RgbaImage;
use lz4::block::{compress, decompress, CompressionMode};
use rayon::prelude::*;
use rmp_serde::{encode::write_named as mp_to_writer, from_read as mp_from_reader};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
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

	pub fn update_rank<S: AsRef<str>, T: AsRef<str>>(&mut self, lang: S, query: T) {
		self.rank = 0.0;

		for slice in query.as_ref().split_whitespace() {
			if let Some(name) = self.name.get(lang.as_ref()) {
				self.rank += rank_similarity(slice, name);
			}

			if let Some(keywords) = self.keywords.get(lang.as_ref()) {
				for keyword in keywords {
					for kw_slice in keyword.split_whitespace() {
						self.rank += rank_similarity(slice, kw_slice);
					}
				}
			}
		}
	}

	pub fn get_image(&self, area_width: usize, area_height: usize) -> Result<RgbaImage, Error> {
		let svg = nsvg::parse_str(&self.svg, nsvg::Units::Pixel, 96.0)?;

		let area_aspect = area_width as f32 / area_height as f32;
		let svg_aspect = svg.width() / svg.height();

		let scale = if area_aspect > svg_aspect {
			area_height as f32 / svg.height()
		} else {
			area_width as f32 / svg.width()
		};

		Ok(svg.rasterize(scale)?)
	}
}

fn rank_similarity<S: AsRef<str>, T: AsRef<str>>(query: S, subject: T) -> f32 {
	let query = query.as_ref();
	let subject = subject.as_ref();

	if query == subject {
		return 5.0;
	}

	if subject.starts_with(query) {
		return 3.0;
	}

	trigram::similarity(query, subject)
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
		let lang = lang.as_ref().to_string();
		let query = query.as_ref().to_string();

		self.emojis.as_mut_slice()
			.par_iter_mut()
			.for_each(move |emoji| emoji.update_rank(&lang, &query));

		self.emojis
			.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap());
	}
}
