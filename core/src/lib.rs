pub mod error;

use failure::Error;
use lz4::block::{compress, decompress, CompressionMode};
use rayon::prelude::*;
use rmp_serde::{encode::write_named as mp_to_writer, from_read as mp_from_reader};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Emoji {
	pub value: String,
	pub name: HashMap<String, String>,
	pub keywords: HashMap<String, Vec<String>>,
	pub svg: String,

	#[serde(skip)]
	pub rank: f32,
}

fn get_by_lang<T, S: AsRef<str>>(h: &HashMap<String, T>, lang: S) -> Option<&T> {
	let lang = lang.as_ref();
	if let Some(v) = h.get(lang) {
		return Some(v);
	}

	if lang.len() > 3 {
		// fall back to "parent" lang. E.g. for "en_GB" try "en"
		let (parent, _) = lang.split_at(2);
		if let Some(v) = h.get(parent) {
			return Some(v);
		}
	}

	// last resort, English
	h.get("en")
}

impl Emoji {
	pub fn name<S: AsRef<str>>(&self, lang: S) -> &str {
		match get_by_lang(&self.name, lang) {
			Some(v) => v.as_str(),
			None => "Unknown",
		}
	}

	pub fn update_rank<S: AsRef<str>, T: AsRef<str>>(&mut self, lang: S, query: T) {
		self.rank = 0.0;

		for slice in query.as_ref().split_whitespace() {
			if let Some(name) = self.name.get(lang.as_ref()) {
				self.rank += rank_similarity(slice, name);
			}

			if let Some(keywords) = get_by_lang(&self.keywords, &lang) {
				for keyword in keywords {
					for kw_slice in keyword.split_whitespace() {
						self.rank += rank_similarity(slice, kw_slice);
					}
				}
			}
		}
	}

	pub fn get_image(
		&self,
		area_width: usize,
		area_height: usize,
	) -> Result<image::RgbaImage, Error> {
		let opts = usvg::Options::default();
		let tree = usvg::Tree::from_str(&self.svg, &opts.to_ref())?;
		let fit = usvg::FitTo::Size(area_width as u32, area_height as u32);
		let fit_size = fit
			.fit_to(tree.svg_node().size.to_screen_size())
			.ok_or_else(|| failure::err_msg("fit failure"))?;
		let mut pixmap = tiny_skia::Pixmap::new(fit_size.width(), fit_size.height())
			.ok_or_else(|| failure::err_msg("zero sized image"))?;
		resvg::render(&tree, fit, tiny_skia::Transform::default(), pixmap.as_mut())
			.ok_or_else(|| failure::err_msg("failed to fit svg to size"))?;
		let buf = pixmap.data().to_vec();
		image::ImageBuffer::from_raw(pixmap.width(), pixmap.height(), buf)
			.ok_or_else(|| failure::err_msg("buffer not large enough"))
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
	pub emojis: Vec<Arc<Emoji>>,
	pub locale_codes: Vec<String>,
}

impl Index {
	pub fn from_bytes(bytes: &[u8]) -> Result<Index, Error> {
		let uncompressed: Vec<u8> = decompress(bytes, None)?;

		Ok(mp_from_reader(&*uncompressed)?)
	}

	pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
		let mut uncompressed: Vec<u8> = Vec::new();
		mp_to_writer(&mut uncompressed, self)?;

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
			.map(|e| format!("{}: {}", e.value, e.name(lang.as_ref())))
			.collect()
	}

	pub fn search<S: AsRef<str>, T: AsRef<str>>(&mut self, lang: S, query: T) {
		let lang = lang.as_ref().to_string();
		let query = query.as_ref().to_string();

		self.emojis
			.as_mut_slice()
			.par_iter_mut()
			.for_each(move |ref mut emoji| Arc::make_mut(emoji).update_rank(&lang, &query));

		self.emojis
			.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap());
	}
}
