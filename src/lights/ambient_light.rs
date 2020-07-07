use crate::math::Vector3;

pub struct AmbientLight {
	pub color: Vector3,
	pub intensity: f32
}

impl AmbientLight {
	pub fn new() -> Self {
		Self {
			color: Vector3::from_scalar(1.0),
			intensity: 1.0
		}
	}
}