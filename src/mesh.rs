use crate::geometry::Geometry;
use crate::math::{Vector3, Quaternion, Matrix4};
use crate::Object3D;

#[derive(Copy, Clone)]
pub enum Material {
	Basic,
	Lambert
}

pub struct Mesh {
	pub position: Vector3,
	pub rotation: Quaternion,
	pub scale: Vector3,
	pub model_matrix: Matrix4,
	pub geometry: Box<dyn Geometry>,
	pub material: Material
}

impl Mesh {
	pub fn new(geometry: Box<dyn Geometry>, material: Material) -> Self {
		Mesh {
			position: Vector3::new(),
			rotation: Quaternion::new(),
			scale: Vector3::from_scalar(1.0),
			model_matrix: Matrix4::new(),
			geometry,
			material
		}
	}
}

impl Object3D for Mesh {
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
		&self.model_matrix
	}
	
	fn get_matrix_mut(&mut self) -> &mut Matrix4 {
		&mut self.model_matrix
	}
}