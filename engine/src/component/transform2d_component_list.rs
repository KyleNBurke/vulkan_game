use crate::{EntityManager, Entity};
use super::{ComponentList, Transform2D};

pub struct Transform2DComponentList {
	component_list: ComponentList<Transform2D>,
	dirty_count: usize
}

impl Transform2DComponentList {
	pub fn new() -> Self {
		Self {
			component_list: ComponentList::<Transform2D>::new(),
			dirty_count: 0
		}
	}

	pub fn add(&mut self, entity_manager: &mut EntityManager, entity: Entity, mut transform: Transform2D) {
		transform.update_matrix();
		self.component_list.add(entity_manager, entity, transform);
	}

	pub fn remove(&mut self, entity_manager: &mut EntityManager, entity: &Entity) {
		let transform = self.component_list.borrow(entity);

		if transform.dirty {
			self.dirty_count -= 1;
		}

		self.component_list.remove(entity_manager, entity);
	}

	pub fn borrow(&self, entity: &Entity) -> &Transform2D {
		self.component_list.borrow(entity)
	}

	pub fn borrow_mut(&mut self, entity: &Entity) -> &mut Transform2D {
		let transform = self.component_list.borrow_mut(entity);
		transform.dirty = true;
		self.dirty_count += 1;
		transform
	}

	pub fn try_borrow(&self, entity: &Entity) -> Option<&Transform2D> {
		self.component_list.try_borrow(entity)
	}

	pub fn try_borrow_mut(&mut self, entity: &Entity) -> Option<&mut Transform2D> {
		let transform = self.component_list.try_borrow_mut(entity)?;
		transform.dirty = true;
		self.dirty_count += 1;
		Some(transform)
	}

	pub fn update(&mut self, entity: &Entity) {
		let transform = self.component_list.borrow_mut(entity);
		
		if transform.dirty {
			transform.update_matrix();
			transform.dirty = false;
			self.dirty_count -= 1;
		}
	}

	pub fn check_for_dirties(&self) {
		assert!(self.dirty_count == 0, "{} matrix/matrices have not been calculated", self.dirty_count);
	}
}