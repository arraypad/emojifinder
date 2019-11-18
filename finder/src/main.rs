use emojifinder_core::{Emoji, Index};

fn main() {
	env_logger::init();

	let index = Index::from_bytes(include_bytes!("../data/index.bin"));
	println!("Hello, world! {:?}", index);
}
