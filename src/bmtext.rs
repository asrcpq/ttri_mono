use anyhow::{Result, anyhow};
use ttri_model::cmodel::{Face, Model};

pub fn wide_test(ch: u32) -> bool {
	if let Some(ch) = char::from_u32(ch) {
		match unicode_width::UnicodeWidthChar::width(ch) {
			Some(x) => x >= 2,
			None => true,
		}
	} else {
		false
	}
}

pub struct FontConfig {
	scaler: i32,
	font_size: [u32; 2],
	screen_size: [u32; 2],
	texture_size: [u32; 2],
}

impl FontConfig {
	pub fn new(
		screen_size: [u32; 2],
		texture_size: [u32; 2],
		font_size: [u32; 2],
	) -> Self {
		Self {
			scaler: 1,
			font_size,
			texture_size,
			screen_size,
		}
	}

	pub fn with_scaler(mut self, scaler: i32) -> Self {
		self.scaler = scaler;
		self
	}

	pub fn resize_screen(&mut self, new_size: [u32; 2]) {
		self.screen_size = new_size;
		// eprintln!("FontConfig resized: {:?}", new_size);
	}

	// half wide
	pub fn get_scaled_font_size(&self) -> [u32; 2] {
		[
			self.font_size[0] * self.scaler as u32 / 2,
			self.font_size[1] * self.scaler as u32,
		]
	}

	pub fn get_font_size(&self) -> [u32; 2] {
		self.font_size
	}

	fn generate_vs(&self) -> [Vec<[f32; 4]>; 2] {
		let [xx, yy] = self.get_terminal_size_in_char();
		let [col, row] = self.get_scaled_font_size();
		let mut vs = Vec::new();
		for y in 0..=yy {
			for x in 0..=xx {
				vs.push([(x * col) as f32, (y * row) as f32, 0.0, 1.0]);
			}
		}
		let mut bvs = Vec::new();
		// background layer
		for y in 0..=yy {
			for x in 0..=xx {
				bvs.push([(x * col) as f32, (y * row) as f32, 0.5, 1.0]);
			}
		}
		[vs, bvs]
	}

	fn generate_uvs(&self) -> Vec<[f32; 2]> {
		let [xx, yy] = self.get_texture_size_in_char();
		let mut uvs = Vec::new();
		for y in 0..=yy {
			for x in 0..=xx {
				uvs.push([
					x as f32 / xx as f32,
					y as f32 / yy as f32,
				]);
				uvs.push([
					(x as f32 + 0.5) / xx as f32,
					y as f32 / yy as f32,
				]);
			}
		}
		uvs
	}

	pub fn generate_models(&self) -> [Model; 2] {
		let [vs, bvs] = self.generate_vs();
		let uvs = self.generate_uvs();
		[
			Model {
				vs,
				uvs,
				faces: Default::default(),
			},
			Model {
				vs: bvs,
				uvs: Default::default(),
				faces: Default::default(),
			},
		]
	}

	pub fn get_terminal_size_in_char(&self) -> [u32; 2] {
		[
			self.screen_size[0] / (self.font_size[0] / 2 * self.scaler as u32),
			self.screen_size[1] / (self.font_size[1] * self.scaler as u32),
		]
	}

	fn get_texture_size_in_char(&self) -> [u32; 2] {
		[
			self.texture_size[0] / self.font_size[0],
			self.texture_size[1] / self.font_size[1],
		]
	}

	pub fn char(
		&self,
		offset: [u32; 2],
		ch: u32,
		fg_color: [f32; 4],
		bg_color: [f32; 4],
		tex_layer: i32,
	) -> Result<[Face; 4]> {
		// x1 terminal size(in char), x2 texture size(in char)
		let [x1, y1] = self.get_terminal_size_in_char();
		let idx = offset[0] + offset[1] * x1;
		let [x2, _] = self.get_texture_size_in_char();
		let wide = wide_test(ch);
		// 10 chars has 11 vertices
		let pos_x = idx % x1;
		let pos_y = idx / x1;
		if pos_x >= x1 || pos_y >= y1 || pos_x == x1 - 1 && wide {
			return Err(anyhow!("text overflow"))
		}
		let screen_leftup = (pos_y * (x1 + 1) + pos_x) as usize;
		let screen_leftdown = ((pos_y + 1) * (x1 + 1) + pos_x) as usize;

		let pos_x = ch % x2;
		let pos_y = ch / x2;
		let texture_leftup = (pos_y * (x2 + 1) + pos_x) as usize * 2;
		let texture_leftdown = ((pos_y + 1) * (x2 + 1) + pos_x) as usize * 2;

		let n = if wide { 2 } else { 1 };
		let face0 = Face {
			vid: [screen_leftup, screen_leftup + n, screen_leftdown],
			color: fg_color,
			layer: tex_layer,
			uvid: [texture_leftup, texture_leftup + n, texture_leftdown],
		};
		let face1 = Face {
			vid: [screen_leftup + n, screen_leftdown, screen_leftdown + n],
			color: fg_color,
			layer: tex_layer,
			uvid: [texture_leftup + n, texture_leftdown, texture_leftdown + n],
		};
		let face2 = Face {
			vid: [screen_leftup, screen_leftup + n, screen_leftdown],
			color: bg_color,
			layer: -2,
			uvid: [0; 3],
		};
		let face3 = Face {
			vid: [screen_leftup + n, screen_leftdown, screen_leftdown + n],
			color: bg_color,
			layer: -2,
			uvid: [0; 3],
		};
		Ok([face0, face1, face2, face3])
	}
}
