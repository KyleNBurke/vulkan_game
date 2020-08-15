pub const ZERO: Vector2 = Vector2 { x: 0.0, y: 0.0 };

pub struct Vector2 {
	pub x: f32,
	pub y: f32
}

impl Vector2 {
	pub fn new() -> Self {
		ZERO
	}
}