const IDENTITY: [[f32; 4]; 4] = [
	[1.0, 0.0, 0.0, 0.0],
	[0.0, 1.0, 0.0, 0.0],
	[0.0, 0.0, 1.0, 0.0],
	[0.0, 0.0, 0.0, 1.0]
];

pub struct Matrix4 {
	pub elements: [[f32; 4]; 4]
}

impl Matrix4 {
	pub fn new() -> Self {
		Matrix4 {
			elements: IDENTITY
		}
	}
}