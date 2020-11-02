pub const ZERO: Vector2 = Vector2 { x: 0.0, y: 0.0 };

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Vector2 {
	pub x: f32,
	pub y: f32
}

impl Vector2 {
	pub fn new() -> Self {
		ZERO
	}

	pub fn from(x: f32, y: f32) -> Self {
		Self { x, y }
	}

	pub fn from_scalar(scalar: f32) -> Self {
		Self { x: scalar, y: scalar }
	}

	pub fn set(&mut self, x: f32, y: f32) {
		self.x = x;
		self.y = y;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new() {
		assert_eq!(Vector2::new(), Vector2 { x: 0.0, y: 0.0 });
	}

	#[test]
	fn from() {
		assert_eq!(Vector2::from(1.0, 2.0), Vector2 { x: 1.0, y: 2.0 });
	}

	#[test]
	fn from_scalar() {
		assert_eq!(Vector2::from_scalar(1.0), Vector2 { x: 1.0, y: 1.0 });
	}

	#[test]
	fn set() {
		let mut v = Vector2::new();
		v.set(1.0, 2.0);
		assert_eq!(v, Vector2 { x: 1.0, y: 2.0 });
	}
}