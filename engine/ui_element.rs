use crate::{
	math::{Vector2, Matrix3},
	geometry2d::Geometry2D,
	Object2D
};

pub struct UIElement {
	pub position: Vector2,
	pub rotation: f32,
	pub scale: Vector2,
	pub matrix: Matrix3,
	pub geometry: Box<dyn Geometry2D>,
}

impl UIElement {
	pub fn new(geometry: Box<dyn Geometry2D>) -> Self {
		Self {
			position: Vector2 { x: 0.0, y: 0.0 },
			rotation: 0.0,
			scale: Vector2::new(),
			matrix: Matrix3::new(),
			geometry
		}
	}
}

impl Object2D for UIElement {
	fn get_position(&self) -> &Vector2 {
		&self.position
	}

	fn get_position_mut(&mut self) -> &mut Vector2 {
		&mut self.position
	}

	fn get_rotation(&self) -> f32 {
		self.rotation
	}

	fn get_rotation_mut(&mut self) -> &mut f32 {
		&mut self.rotation
	}

	fn get_scale(&self) -> &Vector2 {
		&self.scale
	}

	fn get_scale_mut(&mut self) -> &mut Vector2 {
		&mut self.scale
	}
}