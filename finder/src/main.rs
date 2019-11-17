use emojifinder_index::Emoji;

fn main() {
	env_logger::init();

	let e = Emoji::default();
	println!("Hello, world! {:?}", e);
}
