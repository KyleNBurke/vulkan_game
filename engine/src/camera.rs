use crate::{Transform3D, math::Matrix4};

pub struct Camera {
	pub transform: Transform3D,
	pub auto_update_view_matrix: bool,
	pub projection_matrix: Matrix4
}

impl Camera {
	pub fn new(aspect: f32, fov: f32, near: f32, far: f32) -> Self {
		let mut projection_matrix = Matrix4::new();
		projection_matrix.make_perspective(aspect, fov, near, far);

		Self {
			transform: Transform3D::new(),
			auto_update_view_matrix: true,
			projection_matrix
		}
	}
}