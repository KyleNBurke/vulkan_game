const IDENTITY: [[f32; 4]; 4] = [
	[1.0, 0.0, 0.0, 0.0],
	[0.0, 1.0, 0.0, 0.0],
	[0.0, 0.0, 1.0, 0.0],
	[0.0, 0.0, 0.0, 1.0]
];

#[derive(Copy, Clone)]
pub struct Matrix4 {
	pub elements: [[f32; 4]; 4]
}

impl Matrix4 {
	pub fn new() -> Self {
		Matrix4 {
			elements: IDENTITY
		}
	}

	pub fn set(&mut self, elements: [[f32; 4]; 4]) {
		self.elements = elements;
	}

	pub fn transpose(&mut self) {
		let e = &mut self.elements;
		let mut temp;
		
		temp = e[1][0]; e[1][0] = e[0][1]; e[0][1] = temp;
		temp = e[2][0]; e[2][0] = e[0][2]; e[0][2] = temp;
		temp = e[3][0]; e[3][0] = e[0][3]; e[0][3] = temp;
		
		temp = e[2][1]; e[2][1] = e[1][2]; e[1][2] = temp;
		temp = e[3][1]; e[3][1] = e[1][3]; e[1][3] = temp;

		temp = e[3][2]; e[3][2] = e[2][3]; e[2][3] = temp;
	}
}

impl From<[[f32; 4]; 4]> for Matrix4 {
    fn from(elements: [[f32; 4]; 4]) -> Self {
		Matrix4 {
			elements
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new() {
		assert_eq!(Matrix4::new().elements, IDENTITY);
	}

	#[test]
	fn set() {
		let elements = [
			[0.0, 0.1, 0.2, 0.3],
			[1.0, 1.1, 1.2, 1.3],
			[2.0, 2.1, 2.2, 2.3],
			[3.0, 3.1, 3.2, 3.3]
		];

		let mut m = Matrix4::new();
		m.set(elements);

		assert_eq!(m.elements, elements);
	}

	#[test]
	fn from() {
		let elements = [
			[0.0, 0.1, 0.2, 0.3],
			[1.0, 1.1, 1.2, 1.3],
			[2.0, 2.1, 2.2, 2.3],
			[3.0, 3.1, 3.2, 3.3]
		];
		
		assert_eq!(Matrix4::from(elements).elements, elements);
	}

	#[test]
	fn transpose() {
		let mut m = Matrix4::from([
			[0.0, 0.1, 0.2, 0.3],
			[1.0, 1.1, 1.2, 1.3],
			[2.0, 2.1, 2.2, 2.3],
			[3.0, 3.1, 3.2, 3.3]
		]);
		m.transpose();

		let expected = [
			[0.0, 1.0, 2.0, 3.0],
			[0.1, 1.1, 2.1, 3.1],
			[0.2, 1.2, 2.2, 3.2],
			[0.3, 1.3, 2.3, 3.3]
		];

		assert_eq!(m.elements, expected);
	}
}