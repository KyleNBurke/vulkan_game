use crate::{Entity, Font, pool::Pool};
use super::{ComponentList, Text};

pub struct TextComponentList {
	component_list: ComponentList<Text>,
	dirty_list: Vec<Entity>
}

impl TextComponentList {
	pub fn new() -> Self {
		Self {
			component_list: ComponentList::<Text>::new(),
			dirty_list: Vec::new()
		}
	}

	pub fn add(&mut self, entity: Entity, text: Text) {
		self.dirty_list.push(entity);
		self.component_list.add(entity, text);
	}

	pub fn remove(&mut self, entity: &Entity) {
		self.component_list.remove(entity);
	}

	pub fn borrow(&self, entity: &Entity) -> &Text {
		self.component_list.borrow(entity)
	}

	pub fn borrow_mut(&mut self, entity: Entity) -> &mut Text {
		self.dirty_list.push(entity);
		self.component_list.borrow_mut(&entity)
	}

	pub fn try_borrow(&self, entity: &Entity) -> Option<&Text> {
		self.component_list.try_borrow(entity)
	}

	pub fn try_borrow_mut(&mut self, entity: Entity) -> Option<&mut Text> {
		self.dirty_list.push(entity);
		self.component_list.try_borrow_mut(&entity)
	}

	pub fn iter(&self) -> impl Iterator<Item = &(Entity, Text)> {
		self.component_list.iter()
	}

	pub fn generate_dirties(&mut self, fonts: &Pool<Font>) {
		while let Some(entity) = self.dirty_list.pop() {
			let text = self.component_list.borrow_mut(&entity);
			let font = fonts.borrow(text.font);

			text.indices.clear();
			text.attributes.clear();

			let mut char_count = 0;
			let mut cursor_pos = 0.0;

			for c in text.string.chars() {
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

				text.indices.append(&mut glyph_indices);
				text.attributes.append(&mut glyph_attributes);

				char_count += 1;
				cursor_pos += glyph.advance;
			}
		}
	}
}