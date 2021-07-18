use crate::{Entity, entity_manager::MAX_ENTITY_COUNT};

pub struct MultiComponentList<T> {
	components: Vec<(Vec<Entity>, T)>,
	entity_to_index_map: [Option<(usize, usize)>; MAX_ENTITY_COUNT]
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

	pub fn remove(&mut self, component_index: usize) {
		let (entities, _) = &self.components[component_index];
		for entity in entities {
			self.entity_to_index_map[entity.index] = None;
		}

		self.components.swap_remove(component_index);

		let (swapped_entities, _) = &self.components[component_index];
		for (iter_index, swapped_entity) in swapped_entities.iter().enumerate() {
			self.entity_to_index_map[swapped_entity.index] = Some((component_index, iter_index));
		}
	}

	pub fn assign(&mut self, entity: Entity, component_index: usize) {
		let (saved_entities, _) = &mut self.components[component_index];
		saved_entities.push(entity);
		let saved_entity_index = saved_entities.len() - 1;
		self.entity_to_index_map[entity.index] = Some((component_index, saved_entity_index));
	}

	pub fn unassign(&mut self, entity: &Entity) {
		let component_index_option = self.entity_to_index_map[entity.index];
		assert!(component_index_option.is_some(), "Cannot unassign component from entity {} because it does not have this component type", entity);
		let (component_index, saved_entity_index) = component_index_option.unwrap();
		let (saved_entities, _) = &mut self.components[component_index];
		assert_eq!(entity.generation, saved_entities[saved_entity_index].generation, "Cannot unassign component from entity {} because it's generation does not match", entity);
		saved_entities.swap_remove(saved_entity_index);
		let swapped_entity = saved_entities[saved_entity_index];
		self.entity_to_index_map[swapped_entity.index] = Some((component_index, saved_entity_index));
		self.entity_to_index_map[entity.index] = None;
	}

	pub fn borrow(&self, entity: &Entity) -> &T {
		let component_index_option = self.entity_to_index_map[entity.index];
		assert!(component_index_option.is_some(), "Cannot borrow component from entity {} because it does not have this component type", entity);
		let (component_index, saved_entity_index) = component_index_option.unwrap();
		let (saved_entities, component) = &self.components[component_index];
		assert_eq!(entity.generation, saved_entities[saved_entity_index].generation, "Cannot borrow component from entity {} because it's generation does not match", entity);
		component
	}

	pub fn borrow_mut(&mut self, entity: &Entity) -> &mut T {
		let component_index_option = self.entity_to_index_map[entity.index];
		assert!(component_index_option.is_some(), "Cannot mutably borrow component from entity {} because it does not have this component type", entity);
		let (component_index, saved_entity_index) = component_index_option.unwrap();
		let (saved_entities, component) = &mut self.components[component_index];
		assert_eq!(entity.generation, saved_entities[saved_entity_index].generation, "Cannot mutably borrow component from entity {} because it's generation does not match", entity);
		component
	}

	pub fn try_borrow(&self, entity: &Entity) -> Option<&T> {
		let (component_index, saved_entity_index) = self.entity_to_index_map[entity.index]?;
		let (saved_entities, component) = &self.components[component_index];
		if entity.generation == saved_entities[saved_entity_index].generation {
			Some(component)
		}
		else {
			None
		}
	}

	pub fn try_borrow_mut(&mut self, entity: &Entity) -> Option<&mut T> {
		let (component_index, saved_entity_index) = self.entity_to_index_map[entity.index]?;
		let (saved_entities, component) = &mut self.components[component_index];
		if entity.generation == saved_entities[saved_entity_index].generation {
			Some(component)
		}
		else {
			None
		}
	}

	pub fn iter(&self) -> impl Iterator<Item = &(Vec<Entity>, T)> {
		self.components.iter()
	}
}