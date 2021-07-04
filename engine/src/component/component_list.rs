use crate::entity_manager::MAX_ENTITY_COUNT;

pub struct ComponentList<T> {
	components: Vec<T>,
	entity_to_index_map: [Option<usize>; MAX_ENTITY_COUNT],
	index_to_entity_map: [usize; MAX_ENTITY_COUNT]
}

impl<T> ComponentList<T> {
	pub fn new() -> Self {
		Self {
			components: Vec::new(),
			entity_to_index_map: [None; MAX_ENTITY_COUNT],
			index_to_entity_map: [0; MAX_ENTITY_COUNT]
		}
	}

	pub fn add(&mut self, entity: usize, component: T) {
		assert!(self.entity_to_index_map[entity].is_none(), "Cannot add component to entity {} because it already has this component type", entity);
		self.components.push(component);
		let index = self.components.len() - 1;
		self.entity_to_index_map[entity] = Some(index);
		self.index_to_entity_map[index] = entity;
	}

	pub fn remove(&mut self, entity: usize) {
		assert!(self.entity_to_index_map[entity].is_some(), "Cannot remove component from entity {} because it does not have this component type", entity);
		let to_remove_index = self.entity_to_index_map[entity].unwrap();
		let last_index = self.components.len() - 1;
		let last_entity = self.index_to_entity_map[last_index];
		self.entity_to_index_map[last_entity] = Some(to_remove_index);
		self.index_to_entity_map[to_remove_index] = last_entity;
		self.entity_to_index_map[entity] = None;
		self.components.swap_remove(to_remove_index);
	}

	pub fn borrow(&self, entity: usize) -> &T {
		let index = self.entity_to_index_map[entity];
		assert!(index.is_some(), "Cannot borrow component from entity {} because it does not have this component type", entity);
		&self.components[index.unwrap()]
	}

	pub fn borrow_mut(&mut self, entity: usize) -> &mut T {
		let index = self.entity_to_index_map[entity];
		assert!(index.is_some(), "Cannot mutably borrow component from entity {} because it does not have this component type", entity);
		&mut self.components[index.unwrap()]
	}

	pub fn try_borrow(&self, entity: usize) -> Option<&T> {
		let index = self.entity_to_index_map[entity]?;
		Some(&self.components[index])
	}

	pub fn try_borrow_mut(&mut self, entity: usize) -> Option<&mut T> {
		let index = self.entity_to_index_map[entity]?;
		Some(&mut self.components[index])
	}
}