use failure::Error;
use indexmap::IndexMap;
use roxmltree::Document;
use std::collections::HashMap;
use std::sync::Arc;

use emojifinder_core::error::Error as EmojiError;
use emojifinder_core::{Emoji, Index};

const ANNO_DIR: &str = "builder/data/cldr/common/annotations";
const SVG_DIR: &str = "builder/data/noto-emoji/svg";
const SVG_PREFIX: &str = "emoji_u";
const OUTPUT_PATH: &str = "finder/src/index.bin";

fn main() {
	env_logger::init();
	std::process::exit(match run() {
		Ok(_) => exitcode::OK,
		Err(e) => {
			eprintln!("Error: {}", e);
			exitcode::IOERR
		}
	})
}

fn run() -> Result<(), Error> {
	let mut map: IndexMap<String, Emoji> = IndexMap::new();
	let mut locales: Vec<String> = Vec::new();

	// Load Noto SVGs

	for svg_path in std::fs::read_dir(SVG_DIR)? {
		let path = svg_path?.path();
		let stem = match path.file_stem() {
			Some(stem) => stem.to_string_lossy(),
			None => continue,
		};

		if !stem.starts_with(SVG_PREFIX) {
			continue;
		}

		let (_, code_points_str) = stem.split_at(SVG_PREFIX.len());
		let code_points: Result<Vec<u32>, _> = code_points_str
			.split('_')
			.map(|p| u32::from_str_radix(p, 16))
			.collect();

		let mut chars: Option<Vec<char>> = code_points?
			.iter()
			.map(|p| std::char::from_u32(*p))
			.collect();

		if let Some(ref mut chars) = chars {
			if chars.len() > 1 {
				// skip emojis with modifiers for now
				continue;
			}

			let value = chars[0].to_string();
			map.insert(
				value.clone(),
				Emoji {
					value,
					name: HashMap::new(),
					keywords: HashMap::new(),
					rank: 0.0,
					svg: std::fs::read_to_string(&path)?,
				},
			);
		} else {
			return Err(EmojiError::parse(format!("Invalid char: {:?}", code_points_str)).into());
		}
	}

	// Populate names and keywords using CLDR annotations

	for anno_path in std::fs::read_dir(ANNO_DIR)? {
		let path = anno_path?.path();
		let name = match path.file_name() {
			Some(name) => name.to_string_lossy(),
			None => continue,
		};

		if !name.ends_with(".xml") {
			continue;
		}

		let (lang, _) = name.split_at(name.len() - 4);
		locales.push(lang.to_string());

		let anno_str = std::fs::read_to_string(&path)?;
		let doc = Document::parse(&anno_str)?;
		let root = doc.root_element();
		let annos = root
			.children()
			.filter(|n| n.has_tag_name("annotations"))
			.flat_map(|n| n.children())
			.filter(|n| n.has_tag_name("annotation"));

		for anno in annos {
			let value = match anno.attribute("cp") {
				Some(value) => value,
				None => continue,
			};

			let emoji = match map.get_mut(value) {
				Some(emoji) => emoji,
				None => continue,
			};

			let value = anno.text().unwrap_or("");

			if anno.attribute("type") == Some("tts") {
				emoji.name.insert(lang.to_string(), value.to_string());
			} else {
				emoji.keywords.insert(
					lang.to_string(),
					value.split(" | ").map(|s| s.into()).collect(),
				);
			}
		}
	}

	let index = Index {
		emojis: map.into_iter().map(|(_, v)| Arc::new(v)).collect(),
		locale_codes: locales,
	};

	index.to_file(OUTPUT_PATH)?;

	Ok(())
}
