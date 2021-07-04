use crate::{component::Transform3D, math::{matrix4, Matrix4}};

pub struct Camera {
	pub projection_matrix: Matrix4,
	pub transform: Transform3D
}

impl Camera {
	pub fn new(aspect: f32, fov: f32, near: f32, far: f32) -> Self {
		let mut projection_matrix = matrix4::IDENTITY;
		projection_matrix.make_perspective(aspect, fov, near, far);

		Self {
			projection_matrix,
			transform: Transform3D::new()
		}
	}

	pub fn update(&mut self) {
		self.transform.update_local_matrix();
		self.transform.global_matrix = self.transform.local_matrix;
	}
}