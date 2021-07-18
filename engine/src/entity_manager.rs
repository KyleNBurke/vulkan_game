use crate::Entity;

pub const MAX_ENTITY_COUNT: usize = 500;

pub struct EntityManager {
	free_entities: Vec<Entity>,
	alive_entity_count: usize
}

impl EntityManager {
	pub fn new() -> Self {
		Self {
			free_entities: Vec::new(),
			alive_entity_count: 0
		}
	}

	pub fn create(&mut self) -> Entity {
		assert!(self.alive_entity_count != MAX_ENTITY_COUNT, "Cannot create entity because the limit of {} has been reached", MAX_ENTITY_COUNT);
		self.alive_entity_count += 1;
		
		if let Some(entity) = self.free_entities.pop() {
			entity
		}
		else {
			Entity::new(self.alive_entity_count - 1, 0)
		}
	}

	pub fn destroy(&mut self, mut entity: Entity) {
		entity.generation += 1;
		self.free_entities.push(entity);
		self.alive_entity_count -= 1;
	}
}