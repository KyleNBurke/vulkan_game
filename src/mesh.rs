use crate::geometry::Geometry;

pub struct Mesh {
	pub geometry: Box<dyn Geometry>
}