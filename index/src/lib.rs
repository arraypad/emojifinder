#[derive(Default, Debug)]
pub struct Emoji {
	codepoint: u32,
	value: String,
	name: String,
	keywords: Vec<String>,
	image: Vec<u8>,
}
