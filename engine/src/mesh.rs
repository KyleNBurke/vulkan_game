use crate::{Geometry3D, Transform3D, pool::Handle};

#[derive(Copy, Clone)]
pub enum Material {
	Basic,
	Lambert
}
pub struct StaticMesh {
	pub transform: Transform3D,
	pub geometry_handle: Handle<Geometry3D>,
	pub material: Material
}

impl StaticMesh {
	pub fn new(geometry_handle: Handle<Geometry3D>, material: Material) -> Self {
		Self {
			transform: Transform3D::new(),
			geometry_handle,
			material
		}
	}
}

pub struct StaticInstancedMesh {
	pub transforms: Vec<Transform3D>,
	pub geometry_handle: Handle<Geometry3D>,
	pub material: Material
}

impl StaticInstancedMesh {
	pub fn new(geometry_handle: Handle<Geometry3D>, material: Material) -> Self {
		Self {
			transforms: vec![],
			geometry_handle,
			material
		}
	}
}

pub struct Mesh {
	pub geometry_handle: Handle<Geometry3D>,
	pub material: Material,
	pub transform: Transform3D
}

impl Mesh {
	pub fn new(geometry_handle: Handle<Geometry3D>, material: Material) -> Self {
		Self {
			geometry_handle,
			material,
			transform: Transform3D::new()
		}
	}
}