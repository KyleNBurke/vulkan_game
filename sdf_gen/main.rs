use std::{env, ptr, ffi::CString, slice, fs, io::Write};
use freetype::freetype::*;

struct Glyph {
	char_code: u32,
	position: (u32, u32),
	size: (u32, u32),
	pitch: i32,
	buffer: Vec<u8>,
	bearing: (i32, i32),
	advance: i32
}

fn main() {
	let args: Vec<String> = env::args().collect();

	if args.len() < 4 {
		println!("Usage: sdf_gen input_font_file ouput_font_file font_size [-bmp ouput_bmp_file]");
		return;
	}

	let input_font_file_path = &args[1];
	let output_font_file_path = &args[2];
	let font_size = args[3].parse::<u32>().unwrap();
	let output_bmp_file_path_index = args.iter().position(|a| a.as_str() == "-bmp");
	let output_bmp_file_path = if let Some(i) = output_bmp_file_path_index { Some(&args[i + 1]) } else { None };

	let (mut glyphs, space_advance) = rasterize_font(input_font_file_path, font_size);
	let atlas = create_atlas(&mut glyphs);

	if let Some(file_path) = output_bmp_file_path {
		save_to_bitmap(file_path, &atlas);
	}

	save_to_font_file(output_font_file_path, &mut glyphs, space_advance, &atlas);
}

fn rasterize_font(file_path: &str, font_size: u32) -> (Vec<Glyph>, i32) {
	let mut library: FT_Library = ptr::null_mut();
	let error = unsafe { FT_Init_FreeType(&mut library) };
	assert!(error == 0, "Error code {} while initializing library", error);

	let mut face: FT_Face = ptr::null_mut();
	let file_path_cstring = CString::new(file_path.to_owned()).unwrap();
	let error = unsafe { FT_New_Face(library, file_path_cstring.as_ptr(), 0, &mut face) };
	assert!(error == 0, "Error code {} while loading a font face", error);

	let error = unsafe { FT_Set_Pixel_Sizes(face, 0, font_size) };
	assert!(error == 0, "Error code {} while setting the font size", error);
	
	let space_advance = unsafe {
		let glyph_index = FT_Get_Char_Index(face, 32);
		let error = FT_Load_Glyph(face, glyph_index, 0);
		assert!(error == 0, "Error code {} while loading the space glyph", error);

		(*(*face).glyph).advance.x / 64
	};

	let char_codes = 33u32..127;
	let mut glyphs: Vec<Glyph> = Vec::with_capacity(char_codes.len());

	for char_code in char_codes {
		let glyph = unsafe {
			let glyph_index = FT_Get_Char_Index(face, char_code);
			let error = FT_Load_Glyph(face, glyph_index, 0);
			assert!(error == 0, "Error code {} while loading a glyph", error);

			let error = FT_Render_Glyph((*face).glyph, FT_Render_Mode::FT_RENDER_MODE_MONO);
			assert!(error == 0, "Error code {} while rendering glyph", error);
		
			*(*face).glyph
		};

		let bitmap = glyph.bitmap;
		let buffer = unsafe { slice::from_raw_parts(bitmap.buffer, bitmap.rows as usize * bitmap.pitch.abs() as usize).to_vec() };

		glyphs.push(Glyph {
			char_code,
			position: (0, 0),
			size: (bitmap.width, bitmap.rows),
			pitch: bitmap.pitch,
			buffer,
			bearing: (glyph.bitmap_left, -glyph.bitmap_top),
			advance: glyph.advance.x / 64
		});
	}

	(glyphs, space_advance)
}

fn create_atlas(glyphs: &mut Vec<Glyph>) -> Vec<Vec<u8>> {
	glyphs.sort_unstable_by(|a, b| (b.size.0 * b.size.1).cmp(&(a.size.0 * a.size.1)));
	let mut atlas: Vec<Vec<u8>> = Vec::new();

	'glyph_loop: for glyph in glyphs {
		let atlas_height = atlas.len();
		let atlas_width = if atlas_height == 0 { 0 } else { atlas[0].len() };

		let glyph_width = glyph.size.0 as usize;
		let glyph_height = glyph.size.1 as usize;

		let atlas_col_bound = atlas_width.saturating_sub(glyph_width - 1);
		let atlas_row_bound = atlas_height.saturating_sub(glyph_height - 1);

		for atlas_row_index in 0..atlas_row_bound {
			'atlas_col_loop: for atlas_col_index in 0..atlas_col_bound {

				for glyph_row_index in 0..glyph_height {
					for glyph_col_index in 0..glyph_width {
						let texel = atlas[atlas_row_index + glyph_row_index][atlas_col_index + glyph_col_index];

						if texel != 127 {
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

		// Glyph cannot fit anywhere, expand atlas in a direction and place the glyph
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

	atlas
}

fn place_glyph(atlas: &mut Vec<Vec<u8>>, atlas_row: usize, atlas_col: usize, glyph: &mut Glyph) {
	let glyph_pitch_abs = glyph.pitch.abs() as usize;

	for glyph_row in 0..glyph.size.1 as usize {
		for glyph_col in 0..glyph.size.0 as usize {
			let glyph_byte = glyph.buffer[glyph_row * glyph_pitch_abs + glyph_col / 8];
			let mask = 0b1000_0000 >> glyph_col % 8;
			let byte_value = if glyph_byte & mask != 0 { 255 } else { 0 };
			atlas[atlas_row + glyph_row][atlas_col + glyph_col] = byte_value;
		}
	}

	glyph.position = (atlas_col as u32, atlas_row as u32);
}

fn expand_atlas(atlas: &mut Vec<Vec<u8>>, vertical_len: usize, horizontal_len: usize) {
	let atlas_width = if atlas.len() == 0 { 0 } else { atlas[0].len() };
	let additional_rows = vec![vec![127u8; atlas_width]; vertical_len];
	atlas.extend_from_slice(&additional_rows);

	let additional_cols = vec![127u8; horizontal_len];
	for row in atlas {
		row.extend_from_slice(&additional_cols);
	}
}

fn save_to_bitmap(file_path: &str, atlas: &Vec<Vec<u8>>) {
	let image_width = atlas[0].len();
	let image_height = atlas.len();
	let image_row_padding_len = (4 - image_width % 4) % 4;

	let mut buffer: Vec<u8> = Vec::with_capacity(1078 + (image_width + image_row_padding_len) * image_height);

	// Header
	buffer.push(66u8);
	buffer.push(77u8);

	let file_size = 0u32.to_ne_bytes();
	buffer.extend_from_slice(&file_size);

	let reserved = 0u16.to_ne_bytes();
	buffer.extend_from_slice(&reserved);
	buffer.extend_from_slice(&reserved);

	let pixel_data_offset = 0u32.to_ne_bytes();
	buffer.extend_from_slice(&pixel_data_offset);

	// Info header
	let header_size = 40u32.to_ne_bytes();
	buffer.extend_from_slice(&header_size);

	let image_width_i32 = (image_width as i32).to_ne_bytes();
	buffer.extend_from_slice(&image_width_i32);

	let image_height_i32 = (image_height as i32).to_ne_bytes();
	buffer.extend_from_slice(&image_height_i32);

	let planes = 1u16.to_ne_bytes();
	buffer.extend_from_slice(&planes);

	let bpp = 8u16.to_ne_bytes();
	buffer.extend_from_slice(&bpp);

	let compression_type = 0u32.to_ne_bytes();
	buffer.extend_from_slice(&compression_type);

	let compressed_image_size = 0u32.to_ne_bytes();
	buffer.extend_from_slice(&compressed_image_size);

	let x_pixels_per_meter = 0i32.to_ne_bytes();
	buffer.extend_from_slice(&x_pixels_per_meter);

	let y_pixels_per_meter = 0i32.to_ne_bytes();
	buffer.extend_from_slice(&y_pixels_per_meter);

	let total_colors = 0u32.to_ne_bytes();
	buffer.extend_from_slice(&total_colors);

	let important_colors = 0u32.to_ne_bytes();
	buffer.extend_from_slice(&important_colors);

	// Color table
	for i in 0..256 {
		let i_u8 = i as u8;
		buffer.push(i_u8);
		buffer.push(i_u8);
		buffer.push(i_u8);
		buffer.push(0u8);
	}

	// Pixel data offset in header
	let pixel_data_offset = (buffer.len() as u32).to_ne_bytes();
	for i in 0..4 { buffer[10 + i] = pixel_data_offset[i] };

	// Pixel data
	let padding = vec![0u8; image_row_padding_len];
	for row in atlas.iter().rev() {
		buffer.extend_from_slice(row);
		buffer.extend_from_slice(&padding);
	}

	// File size in header
	let file_size = (buffer.len() as u32).to_ne_bytes();
	for i in 0..4 { buffer[2 + i] = file_size[i] };

	let mut file = fs::File::create(file_path).unwrap();
	file.write_all(&buffer).unwrap();
}

fn save_to_font_file(file_path: &str, glyphs: &mut Vec<Glyph>, space_advance: i32, atlas: &Vec<Vec<u8>>) {
	glyphs.sort_unstable_by_key(|g| g.char_code);

	let atlas_width = atlas[0].len();
	let atlas_height = atlas.len();
	let glyph_count = glyphs.len();

	let mut buffer: Vec<u8> = Vec::with_capacity(12 + atlas_width * atlas_height + 32 * glyph_count);
	
	let atlas_width = (atlas_width as u32).to_ne_bytes();
	buffer.extend_from_slice(&atlas_width);

	let atlas_height = (atlas_height as u32).to_ne_bytes();
	buffer.extend_from_slice(&atlas_height);

	for row in atlas {
		buffer.extend_from_slice(row);
	}

	let space_advance = space_advance.to_ne_bytes();
	buffer.extend_from_slice(&space_advance);
	
	let glyph_count = (glyph_count as u32).to_ne_bytes();
	buffer.extend_from_slice(&glyph_count);

	for glyph in glyphs {
		let char_code = glyph.char_code.to_ne_bytes();
		buffer.extend_from_slice(&char_code);

		let position_x = glyph.position.0.to_ne_bytes();
		buffer.extend_from_slice(&position_x);

		let position_y = glyph.position.1.to_ne_bytes();
		buffer.extend_from_slice(&position_y);

		let width = glyph.size.0.to_ne_bytes();
		buffer.extend_from_slice(&width);

		let height = glyph.size.1.to_ne_bytes();
		buffer.extend_from_slice(&height);

		let bearing_x = glyph.bearing.0.to_ne_bytes();
		buffer.extend_from_slice(&bearing_x);

		let bearing_y = glyph.bearing.1.to_ne_bytes();
		buffer.extend_from_slice(&bearing_y);

		let advance = glyph.advance.to_ne_bytes();
		buffer.extend_from_slice(&advance);
	}

	let mut file = fs::File::create(file_path).unwrap();
	file.write_all(&buffer).unwrap();
}