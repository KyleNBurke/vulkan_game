use crate::{Transform3D, Geometry3D};
use std::boxed::Box;

#[derive(Copy, Clone)]
pub enum Material {
	Basic,
	Lambert
}

pub struct Mesh {
	pub transform: Transform3D,
	pub auto_update_matrix: bool,
	pub geometry: Box<dyn Geometry3D>,
	pub material: Material,
	pub(crate) index_offset: usize,
	pub(crate) attribute_offset: usize,
	pub(crate) uniform_offset: usize
}

impl Mesh {
	pub fn new(geometry: Box<dyn Geometry3D>, material: Material) -> Self {
		Self {
			transform: Transform3D::new(),
			auto_update_matrix: true,
			geometry,
			material,
			index_offset: 0,
			attribute_offset: 0,
			uniform_offset: 0
		}
	}
}