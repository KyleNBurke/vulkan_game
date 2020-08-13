use crate::math::Vector2;

pub trait Object2D {
	fn get_position(&self) -> &Vector2;
	fn get_position_mut(&mut self) -> &mut Vector2;

	fn get_rotation(&self) -> f32;
	fn get_rotation_mut(&mut self) -> &mut f32;

	fn get_scale(&self) -> &Vector2;
	fn get_scale_mut(&mut self) -> &mut Vector2;
}