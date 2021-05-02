use crate::math::{Vector2, Matrix3};

pub struct Transform2D {
	pub position: Vector2,
	pub rotation: f32,
	pub scale: Vector2,
	pub matrix: Matrix3
}

impl Transform2D {
	pub fn new() -> Self {
		Self {
			position: Vector2::new(),
			rotation: 0.0,
			scale: Vector2::from_scalar(1.0),
			matrix: Matrix3::new()
		}
	}

	pub fn update_matrix(&mut self) {
		self.matrix.compose(&self.position, self.rotation, &self.scale);
	}
}

impl Default for Transform2D {
	fn default() -> Self {
		Self::new()
	}
}