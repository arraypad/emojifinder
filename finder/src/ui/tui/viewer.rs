use failure::Error;
use image::{DynamicImage, FilterType, RgbaImage};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::{Block, Widget};

pub enum ColorMode {
	Luma,
	Rgb,
}

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
}

impl<'a> Viewer<'a> {
	pub fn with_img(img: RgbaImage) -> Viewer<'a> {
		Viewer {
			block: None,
			style: Default::default(),
			img: Some(img),
			img_fn: None,
			color_mode: ColorMode::Luma,
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

		let bg_rgb = match self.style.bg {
			Color::Black => vec![0f32, 0f32, 0f32],
			Color::White => vec![1f32, 1f32, 1f32],
			Color::Rgb(r, g, b) => vec![r as f32 / 255f32, g as f32 / 255f32, b as f32 / 255f32],
			_ => vec![0f32, 0f32, 0f32],
		};

		// downsample image in Y axis since

		let (orig_w, orig_h) = {
			let rgba = img.as_rgba8().unwrap();
			(rgba.width(), rgba.height())
		};

		let img = img
			.resize_exact(orig_w, orig_h / 2, FilterType::Lanczos3)
			.to_rgba();

		let ox = (area.width - img.width() as u16) / 2;
		let oy = (area.height - img.height() as u16) / 2;

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
							1 => '\u{2591}',
							2 => '\u{2592}',
							3 => '\u{2593}',
							_ => '\u{2588}',
						});
					}
					ColorMode::Rgb => {
						cell.set_char('\u{2588}').set_fg(Color::Rgb(
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
