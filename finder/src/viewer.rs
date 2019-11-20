use image::{DynamicImage, FilterType};
use tui::buffer::Buffer;
use tui::layout::{Alignment, Rect};
use tui::style::Style;
use tui::symbols::block;
use tui::widgets::{Block, Text, Widget};

pub struct Viewer<'a> {
	/// A block to wrap the widget in
	block: Option<Block<'a>>,
	/// Widget style
	style: Style,
	/// SVG content
	svg: Option<String>,
}

impl<'a> Viewer<'a> {
	pub fn new(svg: Option<String>) -> Viewer<'a> {
		Viewer {
			block: None,
			style: Default::default(),
			svg,
		}
	}

	pub fn block(mut self, block: Block<'a>) -> Viewer<'a> {
		self.block = Some(block);
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
				let pf: Vec<f32> = p.data.iter().map(|c| *c as f32 / 255.0).collect();
				let luma = (pf[0] * 0.3 + pf[1] * 0.59 + pf[2] * 0.11) * pf[3];
				let luma_u8 = (5.0 * luma) as u8;
				if luma_u8 == 0 {
					continue;
				}

				buf.get_mut(area.left() + x, area.top() + y)
					.set_char(match luma_u8 {
						1 => '\u{2591}',
						2 => '\u{2592}',
						3 => '\u{2593}',
						_ => '\u{2588}',
					});
			}
		}
	}
}
