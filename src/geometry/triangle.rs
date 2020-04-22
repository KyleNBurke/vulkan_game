use super::Geometry;

const VERTEX_INDICES: [u16; 3] = [
	0, 1, 2
];

const VERTEX_ATTRIBUTES: [f32; 9] = [
	0.0, -0.5, 0.0,
	0.5, 0.5, 0.0,
	-0.5, 0.5, 0.0
];

pub struct Triangle {}

impl Geometry for Triangle {
	fn get_vertex_indices(&self) -> &[u16] {
		return &VERTEX_INDICES;
	}

	fn get_vertex_attributes(&self) -> &[f32] {
		return &VERTEX_ATTRIBUTES;
	}
}