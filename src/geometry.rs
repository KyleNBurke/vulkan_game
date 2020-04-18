const TRIANGLE_VERTEX_DATA: [f32; 9] = [
	0.0, -0.5, 0.0,
	0.5, 0.5, 0.0,
	-0.5, 0.5, 0.0
];

const PLANE_VERTEX_DATA: [f32; 18] = [
	-0.5, -0.5, 0.0,
	0.5, 0.5, 0.0,
	-0.5, 0.5, 0.0,
	-0.5, -0.5, 0.0,
	0.5, -0.5, 0.0,
	0.5, 0.5, 0.0
];

pub enum Geometry {
	Triange,
	Plane
}

impl Geometry {
	pub fn get_vertex_data(&self) -> &[f32] {
		match self {
			Self::Triange => {
				return &TRIANGLE_VERTEX_DATA;
			},
			Self::Plane => {
				return &PLANE_VERTEX_DATA;
			}
		}
	}
}