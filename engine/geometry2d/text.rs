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
			
			let left_edge = cursor_pos + glyph.bearing.0;

			let glyph_attributes = [
				left_edge, glyph.bearing.1, glyph.position.0, glyph.position.1,
				left_edge + glyph.size.0, glyph.bearing.1, glyph.position.0 + glyph.size.0, glyph.position.1,
				left_edge + glyph.size.0, glyph.bearing.1 + glyph.size.1, glyph.position.0 + glyph.size.0, glyph.position.1 + glyph.size.1,
				left_edge, glyph.bearing.1 + glyph.size.1, glyph.position.0, glyph.position.1 + glyph.size.1
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