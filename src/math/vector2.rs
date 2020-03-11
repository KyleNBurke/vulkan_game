use std::ops::Add;

#[derive(Debug)]
pub struct Vector2 {
	pub x: f32,
	pub y: f32
}

impl Add for Vector2 {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		Self {
			x: self.x + other.x,
			y: self.y + other.y
		}
	}
}