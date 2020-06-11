use crate::geometry::Geometry;
use crate::math::{Vector3, Quaternion, Matrix4};
use crate::Object3D;

pub struct Mesh {
	pub position: Vector3,
	pub rotation: Quaternion,
	pub scale: Vector3,
	pub model_matrix: Matrix4,
	pub geometry: Box<dyn Geometry>
}

impl Mesh {
	pub fn new(geometry: Box<dyn Geometry>) -> Self {
		Mesh {
			position: Vector3::new(),
			rotation: Quaternion::new(),
			scale: Vector3::from_scalar(1.0),
			model_matrix: Matrix4::new(),
			geometry
		}
	}
}

impl Object3D for Mesh {
	fn position(&self) -> &Vector3 {
		&self.position
	}

	fn position_mut(&mut self) -> &mut Vector3 {
		&mut self.position
	}

	fn rotation(&self) -> &Quaternion {
		&self.rotation
	}

	fn rotation_mut(&mut self) -> &mut Quaternion {
		&mut self.rotation
	}

	fn scale(&self) -> &Vector3 {
		&self.scale
	}

	fn scale_mut(&mut self) -> &mut Vector3 {
		&mut self.scale
	}

	fn matrix(&self) -> &Matrix4 {
		&self.model_matrix
	}
	
	fn matrix_mut(&mut self) -> &mut Matrix4 {
		&mut self.model_matrix
	}
}