use crate::{EntityManager, Entity, entity_manager::MAX_ENTITY_COUNT};

pub struct ComponentList<T> {
	components: Vec<(Entity, T)>,
	entity_to_index_map: [Option<usize>; MAX_ENTITY_COUNT]
}

impl<T> ComponentList<T> {
	pub fn new() -> Self {
		Self {
			components: Vec::new(),
			entity_to_index_map: [None; MAX_ENTITY_COUNT]
		}
	}

	pub fn add(&mut self, entity_manager: &mut EntityManager, entity: Entity, component: T) {
		assert!(self.entity_to_index_map[entity.index].is_none(), "Cannot add component to entity {} because it already has this component type", entity);
		self.components.push((entity, component));
		let component_index = self.components.len() - 1;
		self.entity_to_index_map[entity.index] = Some(component_index);
		entity_manager.increment_component_count(entity.index)
	}

	pub fn remove(&mut self, entity_manager: &mut EntityManager, entity: &Entity) {
		let component_index_option = self.entity_to_index_map[entity.index];
		assert!(component_index_option.is_some(), "Cannot remove component from entity {} because it does not have this component type", entity);
		let component_index = component_index_option.unwrap();
		self.entity_to_index_map[entity.index] = None;
		self.components.swap_remove(component_index);
		let (swapped_entity, _) = self.components[component_index];
		self.entity_to_index_map[swapped_entity.index] = Some(component_index);
		entity_manager.decrement_component_count(entity.index);
	}

	pub fn borrow(&self, entity: &Entity) -> &T {
		let component_index_option = self.entity_to_index_map[entity.index];
		assert!(component_index_option.is_some(), "Cannot borrow component from entity {} because it does not have this component type", entity);
		let (saved_entity, component) = &self.components[component_index_option.unwrap()];
		assert_eq!(entity.generation, saved_entity.generation, "Cannot borrow component from entity {} because it's generation does not match", entity);
		component
	}

	pub fn borrow_mut(&mut self, entity: &Entity) -> &mut T {
		let component_index_option = self.entity_to_index_map[entity.index];
		assert!(component_index_option.is_some(), "Cannot mutably borrow component from entity {} because it does not have this component type", entity);
		let (saved_entity, component) = &mut self.components[component_index_option.unwrap()];
		assert_eq!(entity.generation, saved_entity.generation, "Cannot mutably borrow component from entity {} because it's generation does not match", entity);
		component
	}

	pub fn try_borrow(&self, entity: &Entity) -> Option<&T> {
		let index = self.entity_to_index_map[entity.index]?;
		let (saved_entity, component) = &self.components[index];
		if entity.generation == saved_entity.generation {
			Some(component)
		}
		else {
			None
		}
	}

	pub fn try_borrow_mut(&mut self, entity: &Entity) -> Option<&mut T> {
		let index = self.entity_to_index_map[entity.index]?;
		let (saved_entity, component) = &mut self.components[index];
		if entity.generation == saved_entity.generation {
			Some(component)
		}
		else {
			None
		}
	}

	pub fn iter(&self) -> impl Iterator<Item = &(Entity, T)> {
		self.components.iter()
	}
}