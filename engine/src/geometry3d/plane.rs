use super::Geometry3D;

const VERTEX_INDICES: [u16; 6] = [
	0, 2, 3,
	0, 1, 2
];

const VERTEX_ATTRIBUTES: [f32; 24] = [
	-0.5, -0.5, 0.0, 0.0, 0.0, -1.0,
	 0.5, -0.5, 0.0, 0.0, 0.0, -1.0,
	 0.5,  0.5, 0.0, 0.0, 0.0, -1.0,
	-0.5,  0.5, 0.0, 0.0, 0.0, -1.0
];

pub struct Plane {}

impl Plane {
	pub fn new() -> Self {
		Self {}
	}
}

impl Geometry3D for Plane {
	fn get_vertex_indices(&self) -> &[u16] {
		&VERTEX_INDICES
	}

	fn get_vertex_attributes(&self) -> &[f32] {
		&VERTEX_ATTRIBUTES
	}
}

impl Default for Plane {
	fn default() -> Self {
		Self::new()
	}
}