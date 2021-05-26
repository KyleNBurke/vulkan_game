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
}