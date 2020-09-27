use crate::Glyph;

pub fn generate_atlas(glyphs: &mut Vec<Glyph>) -> Vec<Vec<u8>> {
	println!("Generating atlas");

	// Heuristically start with glyphs that have a bigger area
	glyphs.sort_unstable_by(|a, b| (b.field.len() * b.field[0].len()).cmp(&(a.field.len() * a.field[0].len())));

	let mut atlas: Vec<Vec<Option<u8>>> = Vec::new();

	'glyph_loop: for glyph in glyphs {
		let atlas_height = atlas.len();
		let atlas_width = if atlas_height == 0 { 0 } else { atlas[0].len() };
		
		let glyph_height = glyph.field.len() as usize;
		let glyph_width = glyph.field[0].len() as usize;
		
		let atlas_row_bound = atlas_height.saturating_sub(glyph_height - 1);
		let atlas_col_bound = atlas_width.saturating_sub(glyph_width - 1);

		for atlas_row_index in 0..atlas_row_bound {
			'atlas_col_loop: for atlas_col_index in 0..atlas_col_bound {

				for glyph_row_index in 0..glyph_height {
					for glyph_col_index in 0..glyph_width {
						let texel = atlas[atlas_row_index + glyph_row_index][atlas_col_index + glyph_col_index];

						if texel.is_some() {
							// Glyph cannot fit here, move on to the next position
							continue 'atlas_col_loop;
						}
					}
				}

				// Glyph can fit here
				place_glyph(&mut atlas, atlas_row_index, atlas_col_index, glyph);
				continue 'glyph_loop;
			}
		}

		// Glyph cannot fit anywhere, expand atlas in shorter direction and place the glyph
		let vertical_expansion;
		let horizontal_expansion;
		let pos_row;
		let pos_col;

		if atlas_width > atlas_height {
			vertical_expansion = glyph_height;
			horizontal_expansion = glyph_width.saturating_sub(atlas_width);
			pos_row = atlas_height;
			pos_col = 0;
		}
		else {
			vertical_expansion = glyph_height.saturating_sub(atlas_height);
			horizontal_expansion = glyph_width;
			pos_row = 0;
			pos_col = atlas_width;
		}

		expand_atlas(&mut atlas, vertical_expansion, horizontal_expansion);
		place_glyph(&mut atlas, pos_row, pos_col, glyph);
	}

	// Zero out the unused regions
	let atlas_height = atlas.len();
	let atlas_width = atlas[0].len();
	let mut atlas_final = Vec::with_capacity(atlas_height);
	
	for row in atlas {
		let mut row_final = Vec::with_capacity(atlas_width);

		for texel in row {
			row_final.push(if let Some(dist) = texel { dist } else { 0 });
		}

		atlas_final.push(row_final);
	}

	atlas_final
}

fn place_glyph(atlas: &mut Vec<Vec<Option<u8>>>, atlas_row: usize, atlas_col: usize, glyph: &mut Glyph) {
	let glyph_height = glyph.field.len();
	let glyph_width = glyph.field[0].len();

	for glyph_row in 0..glyph_height {
		for glyph_col in 0..glyph_width {
			atlas[atlas_row + glyph_row][atlas_col + glyph_col] = Some(glyph.field[glyph_row][glyph_col]);
		}
	}

	glyph.position = (atlas_col, atlas_row);
}

fn expand_atlas(atlas: &mut Vec<Vec<Option<u8>>>, vertical_len: usize, horizontal_len: usize) {
	let atlas_width = if atlas.len() == 0 { 0 } else { atlas[0].len() };
	let additional_rows = vec![vec![None; atlas_width]; vertical_len];
	atlas.extend_from_slice(&additional_rows);

	let additional_cols = vec![None; horizontal_len];
	for row in atlas {
		row.extend_from_slice(&additional_cols);
	}
}