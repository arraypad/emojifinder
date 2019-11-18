use failure::Error;
use roxmltree::Document;

use emojifinder_core::error::Error as EmojiError;

const ANNO_PATH: &'static str = "builder/data/cldr/common/annotations/en.xml";
const IMAGE_DIR: &'static str = "builder/data/noto-emoji/svg";
const IMAGE_PREFIX: &'static str = "emoji_u";

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
	let anno_str = std::fs::read_to_string(ANNO_PATH)?;
	let doc = Document::parse(&anno_str)?;
	let root = doc.root_element();
	let annos = root.children()
		.filter(|n| n.has_tag_name("annotations"))
		.flat_map(|n| n.children())
		.filter(|n| n.has_tag_name("annotation"));

	for anno in annos {
		println!("Anno: {:?} = {:?}", &anno, anno.text());
	}

	for image_path in std::fs::read_dir(IMAGE_DIR)? {
		let path = image_path?.path();
		let stem = match path.file_stem() {
			Some(stem) => stem.to_string_lossy(),
			None => continue,
		};

		if stem.starts_with(IMAGE_PREFIX) {
			let (_, code_points_str) = stem.split_at(IMAGE_PREFIX.len());
			let code_points: Result<Vec<u32>, _> = code_points_str.split("_")
				.map(|p| u32::from_str_radix(p, 16))
				.collect();

			let chars: Option<Vec<char>> = code_points?
				.iter()
				.map(|p| std::char::from_u32(*p))
				.collect();

			if let Some(chars) = chars {
				let s: String = chars.iter().collect();
				println!("\t{:?}: {}", &path, &s);
			} else {
				return Err(EmojiError::parse(format!("Invalid char: {:?}", code_points_str)).into());
			}
		}
	}
	Ok(())
}
