use super::Geometry;

const VERTEX_INDICES: [u16; 36] = [
	0, 1, 2, // top
	0, 2, 3,
	4, 6, 5, // bottom
	4, 7, 6,
	2, 1, 5, // right
	2, 5, 6,
	0, 3, 4, // left
	3, 7, 4,
	0, 5, 1, // front
	0, 4, 5,
	3, 2, 6, // back
	3, 6, 7
];

const VERTEX_ATTRIBUTES: [f32; 24] = [
	-0.5, -0.5, 0.5,
	0.5, -0.5, 0.5,
	0.5, -0.5, -0.5,
	-0.5, -0.5, -0.5,
	-0.5, 0.5, 0.5,
	0.5, 0.5, 0.5,
	0.5, 0.5, -0.5,
	-0.5, 0.5, -0.5
];

pub struct Box {}

impl Geometry for Box {
	fn get_vertex_indices(&self) -> &[u16] {
		&VERTEX_INDICES
	}

	fn get_vertex_attributes(&self) -> &[f32] {
		&VERTEX_ATTRIBUTES
	}
}