use crate::math::{vector2, Vector2, matrix3, Matrix3};

pub struct Transform2D {
	pub position: Vector2,
	pub orientation: f32,
	pub scale: Vector2,
	pub matrix: Matrix3
}

impl Transform2D {
	pub fn new() -> Self {
		Self {
			position: vector2::ZERO,
			orientation: 0.0,
			scale: Vector2::from_scalar(1.0),
			matrix: matrix3::IDENTITY
		}
	}

	pub fn update_matrix(&mut self) {
		self.matrix.compose(&self.position, self.orientation, &self.scale);
	}
}