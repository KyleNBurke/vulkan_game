use crate::pool::Handle;

#[derive(Copy, Clone)]
pub enum Material {
	Line,
	Basic,
	Normal,
	Lambert
}

pub struct Mesh {
	pub geometry_handle: Handle,
	pub material: Material
}

impl Mesh {
	pub fn new(geometry_handle: Handle, material: Material) -> Self {
		Self {
			geometry_handle,
			material
		}
	}
}