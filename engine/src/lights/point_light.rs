use crate::math::Vector3;

pub struct PointLight {
	pub position: Vector3,
	pub color: Vector3,
	pub intensity: f32
}

impl PointLight {
	pub fn new() -> Self {
		Self {
			position: Vector3::new(),
			color: Vector3::from_scalar(1.0),
			intensity: 1.0
		}
	}

	pub fn from(color: Vector3, intensity: f32) -> Self {
		Self {
			position: Vector3::new(),
			color,
			intensity
		}
	}
}

impl Default for PointLight {
	fn default() -> Self {
		Self::new()
	}
}