use crate::entity_manager::MAX_ENTITY_COUNT;

pub struct MultiComponentList<T> {
	components: Vec<(Vec<usize>, T)>,
	entity_to_index_map: [Option<usize>; MAX_ENTITY_COUNT]
}

impl<T> MultiComponentList<T> {
	pub fn new() -> Self {
		Self {
			components: Vec::new(),
			entity_to_index_map: [None; MAX_ENTITY_COUNT]
		}
	}

	pub fn add(&mut self, component: T) -> usize {
		self.components.push((Vec::new(), component));
		self.components.len() - 1
	}

	pub fn remove(&mut self, index: usize) {
		let entities = &self.components[index].0;
		for entity in entities {
			self.entity_to_index_map[*entity] = None;
		}

		self.components.swap_remove(index);

		let (swapped_entities, _) = &self.components[index];
		for entity in swapped_entities {
			self.entity_to_index_map[*entity] = Some(index);
		}
	}

	pub fn assign(&mut self, entity: usize, index: usize) {
		self.entity_to_index_map[entity] = Some(index);
		self.components[index].0.push(entity);
	}

	pub fn unassign(&mut self, entity: usize) {
		let index = self.entity_to_index_map[entity];
		assert!(index.is_some(), "Cannot unassign component from entity {} because it does not have this component type", entity);
		let (entities, _) = &mut self.components[index.unwrap()];
		let entity_index = entities.iter().position(|e| *e == entity).unwrap();
		entities.swap_remove(entity_index);
		self.entity_to_index_map[entity] = None;
	}

	pub fn borrow(&self, entity: usize) -> &T {
		let index = self.entity_to_index_map[entity];
		assert!(index.is_some(), "Cannot borrow component from entity {} because it does not have this component type", entity);
		&self.components[index.unwrap()].1
	}

	pub fn borrow_mut(&mut self, entity: usize) -> &mut T {
		let index = self.entity_to_index_map[entity];
		assert!(index.is_some(), "Cannot mutably borrow component from entity {} because it does not have this component type", entity);
		&mut self.components[index.unwrap()].1
	}

	pub fn try_borrow(&self, entity: usize) -> Option<&T> {
		let index = self.entity_to_index_map[entity]?;
		Some(&self.components[index].1)
	}

	pub fn try_borrow_mut(&mut self, entity: usize) -> Option<&mut T> {
		let index = self.entity_to_index_map[entity]?;
		Some(&mut self.components[index].1)
	}

	pub fn iter(&self) -> impl Iterator<Item = &(Vec<usize>, T)> {
		self.components.iter()
	}
}