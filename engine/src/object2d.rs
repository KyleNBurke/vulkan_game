use crate::math::{Vector2, Matrix3};

pub trait Object2D {
	fn get_position(&self) -> &Vector2;
	fn get_position_mut(&mut self) -> &mut Vector2;

	fn get_rotation(&self) -> f32;
	fn get_rotation_mut(&mut self) -> &mut f32;

	fn get_scale(&self) -> &Vector2;
	fn get_scale_mut(&mut self) -> &mut Vector2;

	fn get_matrix(&self) -> &Matrix3;
	fn get_matrix_mut(&mut self) -> &mut Matrix3;

	fn update_matrix(&mut self) {
		let position = *self.get_position();
		let rotation = self.get_rotation();
		let scale = *self.get_scale();

		self.get_matrix_mut().compose(&position, rotation, &scale);
	}
}