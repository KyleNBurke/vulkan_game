use crate::{geometry2d::Geometry2D, Font};

pub struct Text {
	indices: Vec<u16>,
	attributes: Vec<f32>
}

impl Text {
	pub fn new(font: &Font, string: &str) -> Self {
		let (indices, attributes) = Self::generate(font, string);

		Self {
			indices,
			attributes
		}
	}

	fn generate(font: &Font, string: &str) -> (Vec<u16>, Vec<f32>) {
		let mut indices = Vec::with_capacity(6 * string.len());
		let mut attributes = Vec::with_capacity(16 * string.len());
		let mut char_count = 0;
		let mut cursor_pos = 0.0;

		for c in string.chars() {
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

			indices.append(&mut glyph_indices);
			attributes.append(&mut glyph_attributes);
			char_count += 1;
			cursor_pos += glyph.advance;
		}
		
		(indices, attributes)
	}

	pub fn update(&mut self, font: &Font, string: &str) {
		let (indices, attributes) = Self::generate(font, string);

		self.indices = indices;
		self.attributes = attributes;
	}
}

impl Geometry2D for Text {
	fn get_vertex_indices(&self) -> &[u16] {
		&self.indices
	}

	fn get_vertex_attributes(&self) -> &[f32] {
		&self.attributes
	}
}