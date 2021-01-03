use crate::{Transform2D, vulkan::Font, pool::Handle};

pub struct Text {
	pub transform: Transform2D,
	pub auto_update_matrix: bool,
	pub font: Handle,
	string: String,
	indices: Vec<u16>,
	attributes: Vec<f32>,
	pub(crate) generate: bool,
	pub(crate) index_offset: usize,
	pub(crate) attribute_offset: usize,
	pub(crate) matrix_uniform_offset: usize,
	pub(crate) atlas_index_uniform_offset: usize,
	pub(crate) atlas_index: usize
}

impl Text {
	pub fn new(font: Handle, string: String) -> Self {
		Self {
			transform: Transform2D::new(),
			auto_update_matrix: true,
			font,
			string,
			indices: vec![],
			attributes: vec![],
			generate: true,
			index_offset: 0,
			attribute_offset: 0,
			matrix_uniform_offset: 0,
			atlas_index_uniform_offset: 0,
			atlas_index: 0
		}
	}

	pub fn get_string(&self) -> &str {
		&self.string
	}

	pub fn set_string(&mut self, string: String) {
		self.string = string;
		self.generate = true;
	}

	pub(crate) fn generate(&mut self, font: &Font) {
		self.indices.clear();
		self.attributes.clear();

		let mut char_count = 0;
		let mut cursor_pos = 0.0;

		for c in self.string.chars() {
			if c == ' ' {
				cursor_pos += font.space_advance;
				continue;
			}

			let glyph_index = font.glyphs.binary_search_by_key(&(c as u32), |g| g.char_code).unwrap();
			let glyph = &font.glyphs[glyph_index];

			let index_offset = char_count * 4;
			let mut glyph_indices = vec![
				index_offset, index_offset + 1, index_offset + 2,
				index_offset, index_offset + 2, index_offset + 3
			];
			
			let screen_pos_x = cursor_pos + glyph.bearing_x;

			let mut glyph_attributes = vec![
				screen_pos_x, glyph.bearing_y, glyph.position_x, glyph.position_y,
				screen_pos_x + glyph.width, glyph.bearing_y, glyph.position_x + glyph.width, glyph.position_y,
				screen_pos_x + glyph.width, glyph.bearing_y + glyph.height, glyph.position_x + glyph.width, glyph.position_y + glyph.height,
				screen_pos_x, glyph.bearing_y + glyph.height, glyph.position_x, glyph.position_y + glyph.height
			];

			self.indices.append(&mut glyph_indices);
			self.attributes.append(&mut glyph_attributes);

			char_count += 1;
			cursor_pos += glyph.advance;
		}

		self.generate = false;
	}

	pub fn get_vertex_indices(&self) -> &[u16] {
		&self.indices
	}

	pub fn get_vertex_attributes(&self) -> &[f32] {
		&self.attributes
	}
}