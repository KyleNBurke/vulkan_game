use crate::{geometry2d::Geometry2D, Font};

pub struct Text {
	indices: Vec<u16>,
	attributes: Vec<f32>
}

impl Text {
	pub fn new(font: &Font, screen_width: f32, screen_height: f32, string: &str) -> Self {
		let half_screen_width = screen_width / 2.0;
		let half_screen_height = screen_height / 2.0;
		let space_advance = font.space_advance as f32 / half_screen_width;

		let mut indices: Vec<u16> = Vec::with_capacity(6 * string.len());
		let mut attributes: Vec<f32> = Vec::with_capacity(16 * string.len());
		let mut printable_char_count = 0;
		let mut cursor_pos = 0.0;

		for c in string.chars() {
			if c == ' ' {
				cursor_pos += space_advance;
				continue;
			}

			let glyph_index = font.glyphs.binary_search_by_key(&(c as u32), |g| g.char_code).unwrap();
			let glyph = &font.glyphs[glyph_index];

			let index_offset = printable_char_count * 4;
			let glyph_indices = [
				index_offset, index_offset + 1, index_offset + 2,
				index_offset, index_offset + 2, index_offset + 3
			];
			
			let tex_pos_x = glyph.position.0 as f32;
			let tex_pos_y = glyph.position.1 as f32;
			let tex_width = glyph.size.0 as f32;
			let tex_height = glyph.size.1 as f32;
			let width = glyph.size.0 as f32 / half_screen_width;
			let height = glyph.size.1 as f32 / half_screen_height;
			let bearing_x = glyph.bearing.0 as f32 / half_screen_width;
			let bearing_y = glyph.bearing.1 as f32 / half_screen_height;
			let advance = glyph.advance as f32 / half_screen_width;

			let glyph_attributes = [
				cursor_pos + bearing_x, bearing_y, tex_pos_x, tex_pos_y,
				cursor_pos + bearing_x + width, bearing_y, tex_pos_x + tex_width, tex_pos_y,
				cursor_pos + bearing_x + width, bearing_y + height, tex_pos_x + tex_width, tex_pos_y + tex_height,
				cursor_pos + bearing_x, bearing_y + height, tex_pos_x, tex_pos_y + tex_height
			];

			indices.extend_from_slice(&glyph_indices);
			attributes.extend_from_slice(&glyph_attributes);
			printable_char_count += 1;
			cursor_pos += advance;
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