use crate::{Transform3D, Handle};

#[derive(Copy, Clone)]
pub enum Material {
	Basic,
	Lambert
}

pub struct Mesh {
	pub transform: Transform3D,
	pub auto_update_matrix: bool,
	pub geometry: Handle,
	pub material: Material
}

impl Mesh {
	pub fn new(geometry: Handle, material: Material) -> Self {
		Self {
			transform: Transform3D::new(),
			auto_update_matrix: true,
			geometry,
			material
		}
	}
}