use failure::Error;
use image::{imageops::resize, imageops::FilterType, RgbaImage};
use std::cmp::{max, min};
use tui::buffer::Buffer;
use tui::layout::{Alignment, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Widget};

pub enum ColorMode {
	Luma,
	Rgb,
}

const BLOCK_LIGHT: char = '\u{2591}';
const BLOCK_MEDIUM: char = '\u{2592}';
const BLOCK_DARK: char = '\u{2593}';
const BLOCK_FULL: char = '\u{2588}';
const BLOCK_UPPER_HALF: char = '\u{2580}';

/// A tui-rs Widget which displays an image.
pub struct Image<'a> {
	/// A block to wrap the widget in
	block: Option<Block<'a>>,
	/// Widget style
	style: Style,
	/// Image to display
	img: Option<RgbaImage>,
	/// Function returning image to display
	img_fn: Option<Box<dyn Fn(usize, usize) -> Result<RgbaImage, Error>>>,
	/// Color mode
	color_mode: ColorMode,
	/// Alignment of the image
	alignment: Alignment,
}

impl<'a> Image<'a> {
	/// Construct an Image widget with a single image.
	pub fn with_img(img: RgbaImage) -> Image<'a> {
		Image {
			block: None,
			style: Default::default(),
			img: Some(img),
			img_fn: None,
			color_mode: ColorMode::Luma,
			alignment: Alignment::Center,
		}
	}

	/// Construct an Image widget with a function which can be called to obtain an image of the correct size.
	pub fn with_img_fn(
		img_fn: impl Fn(usize, usize) -> Result<RgbaImage, Error> + 'static,
	) -> Image<'a> {
		Image {
			block: None,
			style: Default::default(),
			img: None,
			img_fn: Some(Box::new(img_fn)),
			color_mode: ColorMode::Luma,
			alignment: Alignment::Center,
		}
	}

	/// Set the widget to use the provided block.
	pub fn block(mut self, block: Block<'a>) -> Image<'a> {
		self.block = Some(block);
		self
	}

	/// Set the color mode used to render the image.
	pub fn color_mode(mut self, color_mode: ColorMode) -> Image<'a> {
		self.color_mode = color_mode;
		self
	}

	/// Set the widget style.
	pub fn style(mut self, style: Style) -> Image<'a> {
		self.style = style;
		self
	}

	/// Set the widget alignment.
	pub fn alignment(mut self, alignment: Alignment) -> Image<'a> {
		self.alignment = alignment;
		self
	}

	fn draw_img(&self, area: Rect, buf: &mut Buffer, img: &RgbaImage) {
		// TODO: add other fixed colours
		let bg_rgb = match self.style.bg {
			Some(Color::Black) => vec![0f32, 0f32, 0f32],
			Some(Color::White) => vec![1f32, 1f32, 1f32],
			Some(Color::Rgb(r, g, b)) => {
				vec![r as f32 / 255f32, g as f32 / 255f32, b as f32 / 255f32]
			}
			_ => vec![0f32, 0f32, 0f32],
		};

		// calc offset

		let ox = max(
			0,
			min(
				area.width as i32 - 1,
				match self.alignment {
					Alignment::Center => (area.width as i32 - img.width() as i32) / 2i32,
					Alignment::Left => 0i32,
					Alignment::Right => area.width as i32 - img.width() as i32,
				},
			),
		) as u16;
		let oy = max(
			0,
			min(
				(2 * area.height) - 1,
				((2 * area.height) - img.height() as u16) / 2,
			),
		) as u16;

		// draw

		for y in oy..min(oy + img.height() as u16, (2 * area.height) - 1) {
			for x in ox..min(ox + img.width() as u16, area.width - 1) {
				let p = img.get_pixel((x - ox) as u32, (y - oy) as u32);

				// composite onto background
				let a = p[3] as f32 / 255.0;
				let r = p[0] as f32 * a / 255.0 + bg_rgb[0] * (1f32 - a);
				let g = p[1] as f32 * a / 255.0 + bg_rgb[1] * (1f32 - a);
				let b = p[2] as f32 * a / 255.0 + bg_rgb[2] * (1f32 - a);

				let cell = buf.get_mut(area.left() + x, area.top() + (y / 2));

				match self.color_mode {
					ColorMode::Luma => {
						let luma = r * 0.3 + g * 0.59 + b * 0.11;
						let luma_u8 = (5.0 * luma) as u8;
						if luma_u8 == 0 {
							continue;
						}

						cell.set_char(match luma_u8 {
							1 => BLOCK_LIGHT,
							2 => BLOCK_MEDIUM,
							3 => BLOCK_DARK,
							_ => BLOCK_FULL,
						});
					}
					ColorMode::Rgb => {
						if y & 1 == 0 {
							cell.set_char(BLOCK_UPPER_HALF).set_fg(Color::Rgb(
								(255.0 * r) as u8,
								(255.0 * g) as u8,
								(255.0 * b) as u8,
							));
						} else {
							cell.set_bg(Color::Rgb(
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
}

impl<'a> Widget for Image<'a> {
	fn render(mut self, area: Rect, buf: &mut Buffer) {
		let area = match self.block.take() {
			Some(b) => {
				let inner_area = b.inner(area);
				b.render(area, buf);
				inner_area
			}
			None => area,
		};

		if area.width < 1 || area.height < 1 {
			return;
		}

		buf.set_style(area, self.style);

		if let Some(ref img) = self.img {
			if img.width() > area.width as u32 || img.height() > 2 * area.height as u32 {
				let scaled = resize(
					img,
					area.width as u32,
					2 * area.height as u32,
					FilterType::Nearest,
				);
				self.draw_img(area, buf, &scaled)
			} else {
				self.draw_img(area, buf, img)
			}
		} else if let Some(ref img_fn) = self.img_fn {
			if let Ok(img) = img_fn(area.width as usize, 2 * area.height as usize) {
				self.draw_img(area, buf, &img);
			}
		}
	}
}
