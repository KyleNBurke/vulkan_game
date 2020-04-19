const TRIANGLE_VERTEX_INDICES: [u16; 3] = [
	0, 1, 2
];

const TRIANGLE_VERTEX_ATTRIBUTES: [f32; 9] = [
	0.0, -0.5, 0.0,
	0.5, 0.5, 0.0,
	-0.5, 0.5, 0.0
];

const PLANE_VERTEX_INDICES: [u16; 6] = [
	0, 2, 3,
	0, 1, 2
];

const PLANE_VERTEX_ATTRIBUTES: [f32; 12] = [
	-0.5, -0.5, 0.0,
	0.5, -0.5, 0.0,
	0.5, 0.5, 0.0,
	-0.5, 0.5, 0.0
];

pub enum Geometry {
	Triange,
	Plane
}

impl Geometry {
	pub fn get_vertex_data(&self) -> (&[u16], &[f32]) {
		match self {
			Self::Triange => {
				return (&TRIANGLE_VERTEX_INDICES, &TRIANGLE_VERTEX_ATTRIBUTES);
			},
			Self::Plane => {
				return (&PLANE_VERTEX_INDICES, &PLANE_VERTEX_ATTRIBUTES);
			}
		}
	}
}