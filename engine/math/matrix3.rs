const IDENTITY: [[f32; 3]; 3] = [
	[1.0, 0.0, 0.0],
	[0.0, 1.0, 0.0],
	[0.0, 0.0, 1.0]
];

pub struct Matrix3 {
	pub elements: [[f32; 3]; 3]
}

impl Matrix3 {
	pub fn new() -> Self {
		Self { elements: IDENTITY }
	}

	pub fn to_padded_array(&self) -> [[f32; 4]; 3] {
		let e = &self.elements;

		[
			[e[0][0], e[0][1], e[0][2], 0.0],
			[e[1][0], e[1][1], e[1][2], 0.0],
			[e[2][0], e[2][1], e[2][2], 0.0],
		]
	}
}