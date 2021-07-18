use crate::{EntityManager, Entity};
use super::{ComponentList, Transform3D};

pub struct Transform3DComponentList {
	component_list: ComponentList<Transform3D>,
	dirty_count: usize
}

impl Transform3DComponentList {
	pub fn new() -> Self {
		Self {
			component_list: ComponentList::<Transform3D>::new(),
			dirty_count: 0
		}
	}

	pub fn add(&mut self, entity_manager: &mut EntityManager, entity: Entity, mut transform: Transform3D) {
		transform.update_local_matrix();
		transform.global_matrix = transform.local_matrix;
		self.component_list.add(entity_manager, entity, transform);
	}

	pub fn add_child(&mut self, entity_manager: &mut EntityManager, parent_entity: Entity, child_entity: Entity, mut child_transform: Transform3D) {
		child_transform.update_local_matrix();
		let parent_transform = self.component_list.borrow_mut(&parent_entity);
		child_transform.global_matrix = parent_transform.global_matrix * child_transform.local_matrix;
		parent_transform.child_entities.push(child_entity);
		child_transform.parent_entity = Some(parent_entity);
		self.component_list.add(entity_manager, child_entity, child_transform);
	}

	pub fn remove(&mut self, entity_manager: &mut EntityManager, entity: Entity) {
		let transform = self.component_list.borrow(&entity);

		if let Some(parent_entity) = transform.parent_entity {
			let parent_transform = self.component_list.borrow_mut(&parent_entity);
			let child_entity_index = parent_transform.child_entities.iter().position(|e| *e == entity).unwrap();
			parent_transform.child_entities.swap_remove(child_entity_index);
		}

		let mut entities_to_visit = vec![entity];

		while let Some(entity) = entities_to_visit.pop() {
			let transform = self.component_list.borrow(&entity);
			entities_to_visit.extend_from_slice(&transform.child_entities);

			if transform.dirty {
				self.dirty_count -= 1;
			}

			self.component_list.remove(entity_manager, &entity);
		}
	}

	pub fn borrow(&self, entity: &Entity) -> &Transform3D {
		self.component_list.borrow(entity)
	}

	pub fn borrow_mut(&mut self, entity: &Entity) -> &mut Transform3D {
		let transform = self.component_list.borrow_mut(entity);
		transform.dirty = true;
		self.dirty_count += 1;
		transform
	}

	pub fn try_borrow(&self, entity: &Entity) -> Option<&Transform3D> {
		self.component_list.try_borrow(entity)
	}

	pub fn try_borrow_mut(&mut self, entity: &Entity) -> Option<&mut Transform3D> {
		let transform = self.component_list.try_borrow_mut(entity)?;
		transform.dirty = true;
		self.dirty_count += 1;
		Some(transform)
	}

	pub fn update(&mut self, entity: Entity) {
		let mut entities_to_visit = vec![entity];

		while let Some(entity) = entities_to_visit.pop() {
			let transform = self.component_list.borrow_mut(&entity);
			entities_to_visit.extend_from_slice(&transform.child_entities);

			if transform.dirty {
				transform.dirty = false;
				self.dirty_count -= 1;
			}

			transform.update_local_matrix();

			if let Some(parent_entity) = transform.parent_entity {
				let parent_global_matrix = self.component_list.borrow(&parent_entity).global_matrix;
				let child_transform = self.component_list.borrow_mut(&entity);
				child_transform.global_matrix = parent_global_matrix * child_transform.local_matrix;
			}
			else {
				transform.global_matrix = transform.local_matrix;
			}
		}
	}

	pub fn check_for_dirties(&self) {
		assert!(self.dirty_count == 0, "{} global matrix/matrices have not been calculated", self.dirty_count);
	}
}