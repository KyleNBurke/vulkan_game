use crate::{geometry::Geometry, Font};

pub struct Text {
	indices: Vec<u16>,
	attributes: Vec<f32>
}

impl Text {
	pub fn new(font: &Font, screen_width: f32, screen_height: f32, string: &str) -> Self {
		let mut indices: Vec<u16> = Vec::with_capacity(6 * string.len());
		let mut attributes: Vec<f32> = Vec::with_capacity(16 * string.len());
		let mut cursor_pos = 0.0;

		for (i, c) in string.chars().enumerate() {
			let glyph_index = font.glyphs.binary_search_by_key(&(c as u32), |g| g.char_code).unwrap();
			let glyph = &font.glyphs[glyph_index];

			let index_offset = i as u16 * 4;
			let glyph_indices = [
				index_offset, index_offset + 1, index_offset + 2,
				index_offset, index_offset + 2, index_offset + 3
			];

			let half_screen_width = screen_width / 2.0;
			let half_screen_height = screen_height / 2.0;
			let tex_pos_x = glyph.tex_pos.0 as f32;
			let tex_pos_y = glyph.tex_pos.1 as f32;
			let tex_width = glyph.tex_size.0 as f32;
			let tex_height = glyph.tex_size.1 as f32;
			let tex_width_ndc = glyph.tex_size.0 as f32 / half_screen_width;
			let tex_height_ndc = glyph.tex_size.1 as f32 / half_screen_height;
			let pen_offset_x_ndc = glyph.pen_offset.0 as f32 / half_screen_width;
			let pen_offset_y_ndc = glyph.pen_offset.1 as f32 / half_screen_height;
			let pen_advance_ndc = glyph.pen_advance as f32 / half_screen_width;

			let glyph_attributes = [
				cursor_pos + pen_offset_x_ndc, pen_offset_y_ndc, tex_pos_x, tex_pos_y,
				cursor_pos + pen_offset_x_ndc + tex_width_ndc, pen_offset_y_ndc, tex_pos_x + tex_width, tex_pos_y,
				cursor_pos + pen_offset_x_ndc + tex_width_ndc, pen_offset_y_ndc + tex_height_ndc, tex_pos_x + tex_width, tex_pos_y + tex_height,
				cursor_pos + pen_offset_x_ndc, pen_offset_y_ndc + tex_height_ndc, tex_pos_x, tex_pos_y + tex_height
			];

			indices.extend_from_slice(&glyph_indices);
			attributes.extend_from_slice(&glyph_attributes);
			cursor_pos += pen_advance_ndc;
		}

		Self {
			indices,
			attributes
		}
	}
}

impl Geometry for Text {
	fn get_vertex_indices(&self) -> &[u16] {
		&self.indices
	}

	fn get_vertex_attributes(&self) -> &[f32] {
		&self.attributes
	}
}