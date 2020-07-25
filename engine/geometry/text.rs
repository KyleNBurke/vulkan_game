use super::Geometry;

const VERTEX_INDICES: [u16; 6] = [
	0, 1, 2,
	0, 2, 3
];

const VERTEX_ATTRIBUTES: [f32; 16] = [
	-0.5, -0.5, 0.0, 0.0,
	0.5, -0.5, 1.0, 0.0,
	0.5, 0.5, 1.0, 1.0,
	-0.5, 0.5, 0.0, 1.0
];

pub struct Text {}

impl Geometry for Text {
	fn get_vertex_indices(&self) -> &[u16] {
		&VERTEX_INDICES
	}

	fn get_vertex_attributes(&self) -> &[f32] {
		&VERTEX_ATTRIBUTES
	}
}