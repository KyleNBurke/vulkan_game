use std::{fs, io::{self, Read, Seek}};

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

pub struct Font {
	pub file_path: String,
	pub atlas_width: u32,
	pub atlas_height: u32,
	pub spread: f32,
	pub space_advance: f32,
	pub glyphs: Vec<Glyph>
}

impl Font {
	pub fn new(file_path: String) -> Self {
		let mut bytes = [0u8; 4];
		let mut file = fs::File::open(&file_path).unwrap();

		file.read_exact(&mut bytes).unwrap();
		let atlas_width = u32::from_ne_bytes(bytes);

		file.read_exact(&mut bytes).unwrap();
		let atlas_height = u32::from_ne_bytes(bytes);

		file.seek(io::SeekFrom::Current((atlas_width * atlas_height) as i64)).unwrap();

		file.read_exact(&mut bytes).unwrap();
		let spread = u32::from_ne_bytes(bytes) as f32;

		file.read_exact(&mut bytes).unwrap();
		let space_advance = f32::from_ne_bytes(bytes);

		file.read_exact(&mut bytes).unwrap();
		let glyph_count = u32::from_ne_bytes(bytes) as usize;

		let mut buffer: Vec<u8> = Vec::with_capacity(glyph_count * 32);
		file.read_to_end(&mut buffer).unwrap();

		let mut glyphs: Vec<Glyph> = Vec::with_capacity(glyph_count);

		for glyph_index in 0..glyph_count {
			let glyph_offset = glyph_index * 32;

			bytes.copy_from_slice(&buffer[glyph_offset..glyph_offset + 4]);
			let char_code = u32::from_ne_bytes(bytes);

			bytes.copy_from_slice(&buffer[glyph_offset + 4..glyph_offset + 8]);
			let position_x = u32::from_ne_bytes(bytes) as f32;

			bytes.copy_from_slice(&buffer[glyph_offset + 8..glyph_offset + 12]);
			let position_y = u32::from_ne_bytes(bytes) as f32;

			bytes.copy_from_slice(&buffer[glyph_offset + 12..glyph_offset + 16]);
			let width = u32::from_ne_bytes(bytes) as f32;

			bytes.copy_from_slice(&buffer[glyph_offset + 16..glyph_offset + 20]);
			let height = u32::from_ne_bytes(bytes) as f32;

			bytes.copy_from_slice(&buffer[glyph_offset + 20..glyph_offset + 24]);
			let bearing_x = i32::from_ne_bytes(bytes) as f32;

			bytes.copy_from_slice(&buffer[glyph_offset + 24..glyph_offset + 28]);
			let bearing_y = i32::from_ne_bytes(bytes) as f32;

			bytes.copy_from_slice(&buffer[glyph_offset + 28..glyph_offset + 32]);
			let advance = i32::from_ne_bytes(bytes) as f32;

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

		Self {
			file_path,
			atlas_width,
			atlas_height,
			spread,
			space_advance,
			glyphs
		}
	}
}