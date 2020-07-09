use crate::{
	object3d::Object3D,
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

	pub fn from(color: Vector3, intensity: f32) -> Self {
		Self {
			position: Vector3::new(),
			matrix: Matrix4::new(),
			color,
			intensity
		}
	}
}

impl Object3D for PointLight {
	fn get_position(&self) -> &Vector3 {
		&self.position
	}

	fn get_position_mut(&mut self) -> &mut Vector3 {
		&mut self.position
	}

	fn get_rotation(&self) -> &Quaternion {
		&quaternion::ZERO
	}

	fn get_rotation_mut(&mut self) -> &mut Quaternion {
		panic!("Point lights cannot be rotated");
	}

	fn get_scale(&self) -> &Vector3 {
		&vector3::ONE
	}

	fn get_scale_mut(&mut self) -> &mut Vector3 {
		panic!("Point lights cannot be scaled");
	}

	fn get_matrix(&self) -> &Matrix4 {
		&self.matrix
	}

	fn get_matrix_mut(&mut self) -> &mut Matrix4 {
		&mut self.matrix
	}
}