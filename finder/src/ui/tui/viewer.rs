use failure::Error;
use image::{DynamicImage, FilterType, RgbaImage};
use tui::buffer::Buffer;
use tui::layout::{Alignment, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Widget};

pub enum ColorMode {
	Luma,
	Rgb,
}

const FILLED_BLOCK_1_OF_4: char = '\u{2591}';
const FILLED_BLOCK_2_OF_4: char = '\u{2592}';
const FILLED_BLOCK_3_OF_4: char = '\u{2593}';
const FILLED_BLOCK_4_OF_4: char = '\u{2588}';

pub struct Viewer<'a> {
	/// A block to wrap the widget in
	block: Option<Block<'a>>,
	/// Widget style
	style: Style,
	/// Image to display
	img: Option<RgbaImage>,
	/// Function returning image to display
	img_fn: Option<Box<dyn Fn(f32, f32, f32) -> Result<RgbaImage, Error>>>,
	/// Color mode
	color_mode: ColorMode,
	/// Alignment of the image
	alignment: Alignment,
}

impl<'a> Viewer<'a> {
	pub fn with_img(img: RgbaImage) -> Viewer<'a> {
		Viewer {
			block: None,
			style: Default::default(),
			img: Some(img),
			img_fn: None,
			color_mode: ColorMode::Luma,
			alignment: Alignment::Center,
		}
	}

	pub fn with_img_fn(
		img_fn: impl Fn(f32, f32, f32) -> Result<RgbaImage, Error> + 'static,
	) -> Viewer<'a> {
		Viewer {
			block: None,
			style: Default::default(),
			img: None,
			img_fn: Some(Box::new(img_fn)),
			color_mode: ColorMode::Luma,
			alignment: Alignment::Center,
		}
	}

	pub fn block(mut self, block: Block<'a>) -> Viewer<'a> {
		self.block = Some(block);
		self
	}

	pub fn color_mode(mut self, color_mode: ColorMode) -> Viewer<'a> {
		self.color_mode = color_mode;
		self
	}

	pub fn style(mut self, style: Style) -> Viewer<'a> {
		self.style = style;
		self
	}

	pub fn alignment(mut self, alignment: Alignment) -> Viewer<'a> {
		self.alignment = alignment;
		self
	}
}

impl<'a> Widget for Viewer<'a> {
	fn draw(&mut self, area: Rect, buf: &mut Buffer) {
		let area = match self.block {
			Some(ref mut b) => {
				b.draw(area, buf);
				b.inner(area)
			}
			None => area,
		};

		if area.height < 1 {
			return;
		}

		self.background(area, buf, self.style.bg);

		let img = match self.img {
			Some(ref img) => DynamicImage::ImageRgba8(img.clone()),
			None => match self.img_fn {
				Some(ref img_fn) => match img_fn(area.width as f32, area.height as f32, 2.0) {
					Ok(img) => DynamicImage::ImageRgba8(img.clone()),
					Err(_) => return,
				},
				None => return,
			},
		};

		// TODO: add other fixed colours
		let bg_rgb = match self.style.bg {
			Color::Black => vec![0f32, 0f32, 0f32],
			Color::White => vec![1f32, 1f32, 1f32],
			Color::Rgb(r, g, b) => vec![r as f32 / 255f32, g as f32 / 255f32, b as f32 / 255f32],
			_ => vec![0f32, 0f32, 0f32],
		};

		// resample to half vertical resolution

		let (orig_w, orig_h) = {
			let rgba = img.as_rgba8().unwrap();
			(rgba.width(), rgba.height())
		};

		let img = img
			.resize_exact(orig_w, orig_h / 2, FilterType::Lanczos3)
			.to_rgba();

		// calc offset

		let ox = match self.alignment {
			Alignment::Center => (area.width - img.width() as u16) / 2,
			Alignment::Left => 0,
			Alignment::Right => area.width - img.width() as u16,
		};
		let oy = (area.height - img.height() as u16) / 2;

		// draw

		for y in oy..(oy + img.height() as u16) {
			for x in ox..(ox + img.width() as u16) {
				let p = img.get_pixel((x - ox) as u32, (y - oy) as u32);

				// composite onto background
				let a = p.data[3] as f32 / 255.0;
				let r = p.data[0] as f32 * a / 255.0 + bg_rgb[0] * (1f32 - a);
				let g = p.data[1] as f32 * a / 255.0 + bg_rgb[1] * (1f32 - a);
				let b = p.data[2] as f32 * a / 255.0 + bg_rgb[2] * (1f32 - a);

				let cell = buf.get_mut(area.left() + x, area.top() + y);

				match self.color_mode {
					ColorMode::Luma => {
						let luma = r * 0.3 + g * 0.59 + b * 0.11;
						let luma_u8 = (5.0 * luma) as u8;
						if luma_u8 == 0 {
							continue;
						}

						cell.set_char(match luma_u8 {
							1 => FILLED_BLOCK_1_OF_4,
							2 => FILLED_BLOCK_2_OF_4,
							3 => FILLED_BLOCK_3_OF_4,
							_ => FILLED_BLOCK_4_OF_4,
						});
					}
					ColorMode::Rgb => {
						cell.set_char(FILLED_BLOCK_4_OF_4).set_fg(Color::Rgb(
							(255.0 * r) as u8,
							(255.0 * g) as u8,
							(255.0 * b) as u8,
						));
					}
				}
			}
		}
	}
}
