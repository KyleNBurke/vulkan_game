use super::Geometry;

const VERTEX_INDICES: [u16; 6] = [
	0, 2, 3,
	0, 1, 2
];

const VERTEX_ATTRIBUTES: [f32; 12] = [
	-0.5, -0.5, 0.0,
	0.5, -0.5, 0.0,
	0.5, 0.5, 0.0,
	-0.5, 0.5, 0.0
];

pub struct Plane {}

impl Geometry for Plane {
	fn get_vertex_indices(&self) -> &[u16] {
		return &VERTEX_INDICES;
	}

	fn get_vertex_attributes(&self) -> &[f32] {
		return &VERTEX_ATTRIBUTES;
	}
}