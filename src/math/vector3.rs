use std::ops::Add;

#[derive(Debug)]
pub struct Vector3 {
	pub x: f32,
	pub y: f32,
	pub z: f32
}

impl Add for Vector3 {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		Self {
			x: self.x + other.x,
			y: self.y + other.y,
			z: self.z + other.z
		}
	}
}