use std::{ptr, ffi::CString, slice};
use freetype::freetype::*;

fn main() {
	let mut library: FT_Library = ptr::null_mut();

	let error = unsafe { FT_Init_FreeType(&mut library) };
	assert!(error == 0, "Error code {} while initializing library", error);

	let mut face: FT_Face = ptr::null_mut();
	let file_path = CString::new("../consolas.ttf").unwrap();
	let error = unsafe { FT_New_Face(library, file_path.as_ptr(), 0, &mut face) };
	assert!(error == 0, "Error code {} while loading a font face", error);

	let error = unsafe { FT_Set_Pixel_Sizes(face, 0, 64) };
	assert!(error == 0, "Error code {} while setting the font size", error);

	let glyph_index = unsafe { FT_Get_Char_Index(face, 65) };
	let error = unsafe { FT_Load_Glyph(face, glyph_index, FT_LOAD_DEFAULT as i32) };
	assert!(error == 0, "Error code {} while loading a glyph", error);

	let error = unsafe { FT_Render_Glyph((*face).glyph, FT_Render_Mode::FT_RENDER_MODE_MONO) };
	assert!(error == 0, "Error code {} while rendering glyph", error);

	let bitmap = unsafe { (*(*face).glyph).bitmap };
	let pitch_pos = bitmap.pitch.abs() as u32;
	let buffer = unsafe { slice::from_raw_parts(bitmap.buffer, (bitmap.rows * pitch_pos) as usize) };

	println!("rows: {} width: {} pitch: {}", bitmap.rows, bitmap.width, bitmap.pitch);
	
	for r in 0..bitmap.rows {
		for b in 0..pitch_pos {
			let byte = buffer[(r * pitch_pos + b) as usize];
			
			print!("{:08b}", byte);
		}
		
		println!();
	}
}