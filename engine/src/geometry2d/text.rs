use crate::{geometry2d::Geometry2D, Font};

pub struct Text {
	indices: Vec<u16>,
	attributes: Vec<f32>
}

impl Text {
	pub fn new(font: &Font, string: &str) -> Self {
		let mut indices: Vec<u16> = Vec::with_capacity(6 * string.len());
		let mut attributes: Vec<f32> = Vec::with_capacity(16 * string.len());
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
			let glyph_indices = [
				index_offset, index_offset + 1, index_offset + 2,
				index_offset, index_offset + 2, index_offset + 3
			];
			
			let screen_pos_x = cursor_pos + glyph.bearing_x;

			let glyph_attributes = [
				screen_pos_x, glyph.bearing_y, glyph.position_x, glyph.position_y,
				screen_pos_x + glyph.width, glyph.bearing_y, glyph.position_x + glyph.width, glyph.position_y,
				screen_pos_x + glyph.width, glyph.bearing_y + glyph.height, glyph.position_x + glyph.width, glyph.position_y + glyph.height,
				screen_pos_x, glyph.bearing_y + glyph.height, glyph.position_x, glyph.position_y + glyph.height
			];

			indices.extend_from_slice(&glyph_indices);
			attributes.extend_from_slice(&glyph_attributes);
			char_count += 1;
			cursor_pos += glyph.advance;
		}

		Self {
			indices,
			attributes
		}
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