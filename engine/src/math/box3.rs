use super::Vector3;

pub const DEFAULT_SQUARE: Box3 = Box3 {
	min: Vector3 { x: -1.0, y: -1.0, z: -1.0 },
	max: Vector3 { x:  1.0, y:  1.0, z:  1.0 }
};

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Box3 {
	pub min: Vector3,
	pub max: Vector3
}

impl Box3 {
	pub fn new(min: Vector3, max: Vector3) -> Self {
		Self { min, max }
	}

	pub fn set(&mut self, min: Vector3, max: Vector3) {
		self.min = min;
		self.max = max;
	}

	pub fn as_vertices(&self) -> [Vector3; 8] {
		let min = &self.min;
		let max = &self.max;
		
		[
			Vector3::new(max.x, max.y, max.z),
			Vector3::new(min.x, max.y, max.z),
			Vector3::new(min.x, max.y, min.z),
			Vector3::new(max.x, max.y, min.z),
			Vector3::new(max.x, min.y, max.z),
			Vector3::new(min.x, min.y, max.z),
			Vector3::new(min.x, min.y, min.z),
			Vector3::new(max.x, min.y, min.z)
		]
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new() {
		let min = Vector3::new(1.0, 2.0, 3.0);
		let max = Vector3::new(4.0, 5.0, 6.0);

		assert_eq!(Box3::new(min, max), Box3 { min, max });
	}

	#[test]
	fn set() {
		let min = Vector3::new(1.0, 2.0, 3.0);
		let max = Vector3::new(4.0, 5.0, 6.0);

		let mut b = DEFAULT_SQUARE;
		b.set(min, max);

		assert_eq!(b, Box3 { min, max });
	}

	#[test]
	fn as_vertices() {
		let expected = [
			Vector3::new( 1.0,  1.0,  1.0),
			Vector3::new(-1.0,  1.0,  1.0),
			Vector3::new(-1.0,  1.0, -1.0),
			Vector3::new( 1.0,  1.0, -1.0),
			Vector3::new( 1.0, -1.0,  1.0),
			Vector3::new(-1.0, -1.0,  1.0),
			Vector3::new(-1.0, -1.0, -1.0),
			Vector3::new( 1.0, -1.0, -1.0)
		];

		assert_eq!(DEFAULT_SQUARE.as_vertices(), expected);
	}
}