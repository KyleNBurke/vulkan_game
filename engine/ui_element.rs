use crate::{
	math::{Vector2, Matrix3},
	geometry2d::Geometry2D,
	Object2D
};

pub struct UIElement<'a> {
	pub position: Vector2,
	pub rotation: f32,
	pub scale: Vector2,
	pub matrix: Matrix3,
	pub auto_update_matrix: bool,
	pub geometry: &'a dyn Geometry2D,
}

impl<'a> UIElement<'a> {
	pub fn new(geometry: &'a dyn Geometry2D) -> Self {
		Self {
			position: Vector2 { x: 0.0, y: 0.0 },
			rotation: 0.0,
			scale: Vector2::from_scalar(1.0),
			matrix: Matrix3::new(),
			auto_update_matrix: true,
			geometry
		}
	}
}

impl<'a> Object2D for UIElement<'a> {
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

	fn get_matrix(&self) -> &Matrix3 {
		&self.matrix
	}
	
	fn get_matrix_mut(&mut self) -> &mut Matrix3 {
		&mut self.matrix
	}
}