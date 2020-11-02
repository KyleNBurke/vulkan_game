use crate::{
	Object3D,
	math::{Vector3, Quaternion, Matrix4}
};

pub struct Camera {
	pub position: Vector3,
	pub rotation: Quaternion,
	pub scale: Vector3,
	pub view_matrix: Matrix4,
	pub auto_update_view_matrix: bool,
	pub projection_matrix: Matrix4
}

impl Camera {
	pub fn new(aspect: f32, fov: f32, near: f32, far: f32) -> Self {
		let mut projection_matrix = Matrix4::new();
		projection_matrix.make_perspective(aspect, fov, near, far);

		Self {
			position: Vector3::new(),
			rotation: Quaternion::new(),
			scale: Vector3::from_scalar(1.0),
			view_matrix: Matrix4::new(),
			auto_update_view_matrix: true,
			projection_matrix
		}
	}
}

impl Object3D for Camera {
	fn get_position(&self) -> &Vector3 {
		&self.position
	}

	fn get_position_mut(&mut self) -> &mut Vector3 {
		&mut self.position
	}

	fn get_rotation(&self) -> &Quaternion {
		&self.rotation
	}

	fn get_rotation_mut(&mut self) -> &mut Quaternion {
		&mut self.rotation
	}

	fn get_scale(&self) -> &Vector3 {
		&self.scale
	}

	fn get_scale_mut(&mut self) -> &mut Vector3 {
		&mut self.scale
	}

	fn get_matrix(&self) -> &Matrix4 {
		&self.view_matrix
	}

	fn get_matrix_mut(&mut self) -> &mut Matrix4 {
		&mut self.view_matrix
	}
}