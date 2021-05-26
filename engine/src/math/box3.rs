use super::Vector3;

pub const ONE_BY_ONE: Box3 = Box3 {
	min: Vector3 { x: -0.5, y: -0.5, z: -0.5 },
	max: Vector3 { x: 0.5, y: 0.5, z: 0.5 }
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
		let min = Vector3::from(1.0, 2.0, 3.0);
		let max = Vector3::from(4.0, 5.0, 6.0);

		assert_eq!(Box3::new(min, max), Box3 { min, max });
	}
}