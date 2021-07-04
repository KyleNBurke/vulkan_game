use engine::component::{ComponentList, Transform3DComponentList};

use crate::component::RigidBody;

pub struct PhysicsSystem {
	pub entities: Vec<usize>
}

impl PhysicsSystem {
	pub fn new() -> Self {
		Self {
			entities: Vec::new()
		}
	}

	pub fn update(&self, transform_components: &mut Transform3DComponentList, rigid_body_components: &mut ComponentList<RigidBody>) {
		for entity in &self.entities {
			let rigid_body = rigid_body_components.borrow_mut(*entity);
			rigid_body.velocity += rigid_body.acceleration;

			let transform = transform_components.borrow_mut(*entity);
			transform.position += rigid_body.velocity;
			transform.rotate_y(0.005);

			transform_components.update(*entity);
		}
	}
}