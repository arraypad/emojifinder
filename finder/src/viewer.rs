use image::{DynamicImage, FilterType};
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
	/// SVG content
	svg: Option<String>,
	/// Color mode
	color_mode: ColorMode,
}

impl<'a> Viewer<'a> {
	pub fn new(svg: Option<String>) -> Viewer<'a> {
		Viewer {
			block: None,
			style: Default::default(),
			svg,
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

		let bg_color = match self.style.bg {
			Color::Black => vec![0f32, 0f32, 0f32],
			Color::White => vec![1f32, 1f32, 1f32],
			Color::Rgb(r, g, b) => vec![r as f32 / 255f32, g as f32 / 255f32, b as f32 / 255f32],
			_ => vec![0f32, 0f32, 0f32],
		};

		let svg = match self.svg {
			None => return,
			Some(ref svg) => nsvg::parse_str(svg, nsvg::Units::Pixel, 96.0).unwrap(),
		};

		let area_aspect = area.width as f32 / area.height as f32;
		let svg_aspect = svg.width() / svg.height();

		let scale = if area_aspect > svg_aspect {
			area.height as f32 / svg.height()
		} else {
			area.width as f32 / svg.width()
		};

		let img = svg.rasterize(scale * 2.0).unwrap();
		let orig_w = img.width();
		let orig_h = img.height();

		let img = DynamicImage::ImageRgba8(img)
			.resize_exact(orig_w, orig_h / 2, FilterType::Lanczos3)
			.to_rgba();

		let ox = (area.width - img.width() as u16) / 2;
		let oy = (area.height - img.height() as u16) / 2;

		for y in oy..(oy + img.height() as u16) {
			for x in ox..(ox + img.width() as u16) {
				let p = img.get_pixel((x - ox) as u32, (y - oy) as u32);

				// convert u8 to floats in range 0-1
				let mut pf: Vec<f32> = p.data.iter().map(|c| *c as f32 / 255.0).collect();

				// composite onto background
				pf[0] = pf[0] * pf[3] + bg_color[0] * (1f32 - pf[3]);
				pf[1] = pf[1] * pf[3] + bg_color[1] * (1f32 - pf[3]);
				pf[2] = pf[2] * pf[3] + bg_color[2] * (1f32 - pf[3]);

				let cell = buf.get_mut(area.left() + x, area.top() + y);

				match self.color_mode {
					ColorMode::Luma => {
						let luma = (pf[0] * 0.3 + pf[1] * 0.59 + pf[2] * 0.11) * pf[3];
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
							(255.0 * pf[0]) as u8,
							(255.0 * pf[1]) as u8,
							(255.0 * pf[2]) as u8,
						));
					}
				}
			}
		}
	}
}
