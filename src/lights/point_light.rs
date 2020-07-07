use crate::{
	Object3D,
	math::{vector3, Vector3, quaternion, Quaternion, Matrix4}
};

pub struct PointLight {
	pub position: Vector3,
	pub matrix: Matrix4,
	pub color: Vector3,
	pub intensity: f32
}

impl PointLight {
	pub fn new() -> Self {
		Self {
			position: Vector3::new(),
			matrix: Matrix4::new(),
			color: Vector3::from_scalar(1.0),
			intensity: 1.0
		}
	}
}

impl Object3D for PointLight {
	fn position(&self) -> &Vector3 {
		&self.position
	}

	fn position_mut(&mut self) -> &mut Vector3 {
		&mut self.position
	}

	fn rotation(&self) -> &Quaternion {
		&quaternion::ZERO
	}

	fn rotation_mut(&mut self) -> &mut Quaternion {
		panic!("Point lights cannot be rotated");
	}

	fn scale(&self) -> &Vector3 {
		&vector3::ONE
	}

	fn scale_mut(&mut self) -> &mut Vector3 {
		panic!("Point lights cannot be scaled");
	}

	fn matrix(&self) -> &Matrix4 {
		&self.matrix
	}

	fn matrix_mut(&mut self) -> &mut Matrix4 {
		&mut self.matrix
	}
}