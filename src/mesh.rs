use crate::geometry::Geometry;
use crate::math::Matrix4;

pub struct Mesh {
	pub geometry: Box<dyn Geometry>,
	pub model_matrix: Matrix4
}

impl Mesh {
	pub fn new(geometry: Box<dyn Geometry>) -> Self {
		Mesh {
			geometry,
			model_matrix: Matrix4::new()
		}
	}
}