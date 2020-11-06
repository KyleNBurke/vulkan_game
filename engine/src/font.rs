use std::{path, fs, io, ptr, ffi::CString, slice, io::{Read, Write, Seek}, convert::TryInto};
use freetype::freetype::*;

pub struct Glyph {
	pub char_code: u32,
	pub position_x: f32,
	pub position_y: f32,
	pub width: f32,
	pub height: f32,
	pub bearing_x: f32,
	pub bearing_y: f32,
	pub advance: f32
}

struct UnplacedGlyph {
	char_code: u32,
	bitmap: Vec<Vec<u8>>,
	width: f32,
	height: f32,
	bearing_x: f32,
	bearing_y: f32,
	advance: f32
}

pub struct Font {
	pub fnt_path: String,
	pub atlas_width: usize,
	pub atlas_height: usize,
	pub space_advance: f32,
	pub glyphs: Vec<Glyph>
}

impl Font {
	pub fn new(file_path: &str, size: u32) -> Self {
		let file_path_buf = path::PathBuf::from(file_path);
		let file_stem = file_path_buf.file_stem().unwrap().to_str().unwrap();

		let fnt_path = format!("target/fonts/{}{}.fnt", file_stem, size);

		let (atlas_width, atlas_height, space_advance, glyphs) = match fs::File::open(fnt_path.to_owned()) {
			Ok(file) => {
				println!("Loading font {} at size {}", file_stem, size);

				Self::load_fnt(file)
			},
			Err(e) => {
				if e.kind() == io::ErrorKind::NotFound {
					println!("Generating font {} at size {}", file_stem, size);

					let ttf_path = CString::new(file_path).unwrap();
					let (space_advance, unplaced_glyphs) = Self::load_ttf(ttf_path, size);
					let (atlas, placed_glyphs) = Self::create_atlas(unplaced_glyphs);
					Self::save_fnt(&fnt_path, &atlas, space_advance, &placed_glyphs);

					(atlas[0].len(), atlas.len(), space_advance, placed_glyphs)
				}
				else {
					panic!("{}", e);
				}
			}
		};

		Self {
			fnt_path,
			atlas_width,
			atlas_height,
			space_advance,
			glyphs
		}
	}

	fn load_ttf(ttf_path: CString, size: u32) -> (f32, Vec<UnplacedGlyph>) {
		let mut library: FT_Library = ptr::null_mut();
		let error = unsafe { FT_Init_FreeType(&mut library) };
		assert_eq!(error, 0, "Error code {} while initializing library", error);

		let mut face: FT_Face = ptr::null_mut();
		let error = unsafe { FT_New_Face(library, ttf_path.as_ptr(), 0, &mut face) };
		assert_eq!(error, 0, "Error code {} while loading a font face", error);

		let error = unsafe { FT_Set_Pixel_Sizes(face, 0, size) };
		assert_eq!(error, 0, "Error code {} while setting the font size", error);

		let space_glyph_index = unsafe { FT_Get_Char_Index(face, 32) };
		let error = unsafe { FT_Load_Glyph(face, space_glyph_index, 0) };
		assert_eq!(error, 0, "Error code {} while loading the space glyph", error);
		let space_advance = unsafe { (*(*face).glyph).advance.x / 64 } as f32;

		let char_codes = 33..127;
		let mut unplaced_glyphs: Vec<UnplacedGlyph> = Vec::with_capacity(char_codes.len());

		for char_code in char_codes {
			let glyph_index = unsafe { FT_Get_Char_Index(face, char_code) };
			let error = unsafe { FT_Load_Glyph(face, glyph_index, 0) };
			assert_eq!(error, 0, "Error code {} while loading a glyph", error);

			let error = unsafe { FT_Render_Glyph((*face).glyph, FT_Render_Mode::FT_RENDER_MODE_NORMAL) };
			assert_eq!(error, 0, "Error code {} while rendering glyph", error);

			let ft_glyph = unsafe { *(*face).glyph };
			let ft_bitmap = ft_glyph.bitmap;
			let rows = ft_bitmap.rows as usize;
			let width = ft_bitmap.width as usize;
			let pitch_abs = ft_bitmap.pitch.abs() as usize;

			let mut bitmap: Vec<Vec<u8>> = Vec::with_capacity(rows);

			for row_index in 0..rows {
				bitmap.push(unsafe { slice::from_raw_parts(ft_bitmap.buffer.add(row_index * pitch_abs), width).to_vec() });
			}

			unplaced_glyphs.push(UnplacedGlyph {
				char_code,
				bitmap,
				width: ft_bitmap.width as f32,
				height: ft_bitmap.rows as f32,
				bearing_x: ft_glyph.bitmap_left as f32,
				bearing_y: -ft_glyph.bitmap_top as f32,
				advance: (ft_glyph.advance.x / 64) as f32
			});
		}

		(space_advance, unplaced_glyphs)
	}

	fn create_atlas(unplaced_glyphs: Vec<UnplacedGlyph>) -> (Vec<Vec<u8>>, Vec<Glyph>) {
		let mut unplaced_glyphs_sorted: Vec<&UnplacedGlyph> = unplaced_glyphs.iter().collect();
		unplaced_glyphs_sorted.sort_unstable_by_key(|g| -g.width as isize * g.height as isize);

		let mut placed_glyphs: Vec<Glyph> = Vec::with_capacity(unplaced_glyphs.len());
		let mut atlas: Vec<Vec<Option<u8>>> = Vec::new();

		'glyph_loop: for unplaced_glyph in unplaced_glyphs_sorted {
			let atlas_height = atlas.len();
			let atlas_width = if atlas_height == 0 { 0 } else { atlas[0].len() };
			
			let glyph_height = unplaced_glyph.height as usize;
			let glyph_width = unplaced_glyph.width as usize;
			
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
					placed_glyphs.push(Self::place_glyph(&mut atlas, atlas_row_index, atlas_col_index, unplaced_glyph));
					continue 'glyph_loop;
				}
			}

			// Glyph cannot fit anywhere, expand atlas in shorter direction and place the glyph
			let vertical_expansion;
			let horizontal_expansion;
			let pos_row;
			let pos_col;

			if atlas_width + glyph_width > atlas_height + glyph_height {
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

			Self::expand_atlas(&mut atlas, vertical_expansion, horizontal_expansion);
			placed_glyphs.push(Self::place_glyph(&mut atlas, pos_row, pos_col, unplaced_glyph));
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

		placed_glyphs.sort_unstable_by_key(|g| g.char_code);

		(atlas_final, placed_glyphs)
	}

	fn place_glyph(atlas: &mut Vec<Vec<Option<u8>>>, atlas_row: usize, atlas_col: usize, unplaced_glyph: &UnplacedGlyph) -> Glyph {
		let glyph_height = unplaced_glyph.height as usize;
		let glyph_width = unplaced_glyph.width as usize;
	
		for glyph_row in 0..glyph_height {
			for glyph_col in 0..glyph_width {
				atlas[atlas_row + glyph_row][atlas_col + glyph_col] = Some(unplaced_glyph.bitmap[glyph_row][glyph_col]);
			}
		}

		Glyph {
			char_code: unplaced_glyph.char_code,
			position_x: atlas_col as f32,
			position_y: atlas_row as f32,
			width: unplaced_glyph.width,
			height: unplaced_glyph.height,
			bearing_x: unplaced_glyph.bearing_x,
			bearing_y: unplaced_glyph.bearing_y,
			advance: unplaced_glyph.advance
		}
	}
	
	fn expand_atlas(atlas: &mut Vec<Vec<Option<u8>>>, vertical_len: usize, horizontal_len: usize) {
		let atlas_width = if atlas.is_empty() { 0 } else { atlas[0].len() };
		let additional_rows = vec![vec![None; atlas_width]; vertical_len];
		atlas.extend_from_slice(&additional_rows);
	
		let additional_cols = vec![None; horizontal_len];
		for row in atlas {
			row.extend_from_slice(&additional_cols);
		}
	}

	fn save_fnt(path: &str, atlas: &[Vec<u8>], space_advance: f32, glyphs: &[Glyph]) {
		let atlas_width = atlas[0].len();
		let atlas_height = atlas.len();
		let atlas_padding_size = (4 - (atlas_width * atlas_height) % 4) % 4;
		let glyph_count = glyphs.len();

		let mut buffer: Vec<u8> = Vec::with_capacity(16 + atlas_width * atlas_height + atlas_padding_size + 32 * glyph_count);

		buffer.extend_from_slice(&(atlas_width as u32).to_le_bytes());
		buffer.extend_from_slice(&(atlas_height as u32).to_le_bytes());

		for row in atlas {
			buffer.extend_from_slice(&row);
		}

		buffer.extend_from_slice(&vec![0u8; atlas_padding_size]);
		buffer.extend_from_slice(&space_advance.to_le_bytes());
		buffer.extend_from_slice(&(glyphs.len() as u32).to_le_bytes());

		for glyph in glyphs {
			buffer.extend_from_slice(&glyph.char_code.to_le_bytes());
			buffer.extend_from_slice(&glyph.position_x.to_le_bytes());
			buffer.extend_from_slice(&glyph.position_y.to_le_bytes());
			buffer.extend_from_slice(&glyph.width.to_le_bytes());
			buffer.extend_from_slice(&glyph.height.to_le_bytes());
			buffer.extend_from_slice(&glyph.bearing_x.to_le_bytes());
			buffer.extend_from_slice(&glyph.bearing_y.to_le_bytes());
			buffer.extend_from_slice(&glyph.advance.to_le_bytes());
		}

		fs::create_dir_all("target/fonts").unwrap();
		let mut file = fs::File::create(path).unwrap();
		file.write_all(&buffer).unwrap();
	}

	fn load_fnt(mut file: fs::File) -> (usize, usize, f32, Vec<Glyph>) {
		let mut bytes = [0u8; 4];

		file.read_exact(&mut bytes).unwrap();
		let atlas_width = u32::from_le_bytes(bytes) as usize;

		file.read_exact(&mut bytes).unwrap();
		let atlas_height = u32::from_le_bytes(bytes) as usize;

		let atlas_padding_size = (4 - (atlas_width * atlas_height) % 4) % 4;

		file.seek(io::SeekFrom::Current((atlas_width * atlas_height + atlas_padding_size) as i64)).unwrap();

		file.read_exact(&mut bytes).unwrap();
		let space_advance = f32::from_le_bytes(bytes);

		file.read_exact(&mut bytes).unwrap();
		let glyph_count = u32::from_le_bytes(bytes) as usize;

		let mut buffer: Vec<u8> = Vec::with_capacity(glyph_count * 32);
		file.read_to_end(&mut buffer).unwrap();

		let mut glyphs: Vec<Glyph> = Vec::with_capacity(glyph_count);
		let get_bytes_at = |offset| buffer[offset..offset + 4].try_into().unwrap();

		for glyph_index in 0..glyph_count {
			let offset = glyph_index * 32;

			let char_code = u32::from_le_bytes(get_bytes_at(offset));
			let position_x = f32::from_le_bytes(get_bytes_at(offset + 4));
			let position_y = f32::from_le_bytes(get_bytes_at(offset + 8));
			let width = f32::from_le_bytes(get_bytes_at(offset + 12));
			let height = f32::from_le_bytes(get_bytes_at(offset + 16));
			let bearing_x = f32::from_le_bytes(get_bytes_at(offset + 20));
			let bearing_y = f32::from_le_bytes(get_bytes_at(offset + 24));
			let advance = f32::from_le_bytes(get_bytes_at(offset + 28));

			glyphs.push(Glyph {
				char_code,
				position_x,
				position_y,
				width,
				height,
				bearing_x,
				bearing_y,
				advance
			});
		}

		(atlas_width, atlas_height, space_advance, glyphs)
	}
}