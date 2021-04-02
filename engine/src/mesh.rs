use crate::{Geometry3D, Transform3D, pool::Handle};

#[derive(Copy, Clone)]
pub enum Material {
	Basic,
	Normal,
	Lambert
}

pub struct Mesh {
	pub transform: Transform3D,
	pub geometry_handle: Handle<Geometry3D>,
	pub material: Material
}

impl Mesh {
	pub fn new(geometry_handle: Handle<Geometry3D>, material: Material) -> Self {
		Self {
			transform: Transform3D::new(),
			geometry_handle,
			material
		}
	}
}