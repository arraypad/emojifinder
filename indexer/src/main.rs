use emojifinder_index::Emoji;

fn main() {
	env_logger::init();

	let e = Emoji::default();
	println!("Let's build an index! {:?}", e);
}
