use super::Geometry3D;

const VERTEX_INDICES: [u16; 3] = [
	0, 1, 2
];

const VERTEX_ATTRIBUTES: [f32; 18] = [
	 0.0, -0.5, 0.0, 0.0, 0.0, -1.0,
	 0.5,  0.5, 0.0, 0.0, 0.0, -1.0,
	-0.5,  0.5, 0.0, 0.0, 0.0, -1.0,
];

pub struct Triangle {}

impl Geometry3D for Triangle {
	fn get_vertex_indices(&self) -> &[u16] {
		&VERTEX_INDICES
	}

	fn get_vertex_attributes(&self) -> &[f32] {
		&VERTEX_ATTRIBUTES
	}
}