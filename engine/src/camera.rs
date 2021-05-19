use crate::math::Matrix4;

pub struct Camera {
	pub projection_matrix: Matrix4
}

impl Camera {
	pub fn new(aspect: f32, fov: f32, near: f32, far: f32) -> Self {
		let mut projection_matrix = Matrix4::new();
		projection_matrix.make_perspective(aspect, fov, near, far);

		Self {
			projection_matrix
		}
	}
}