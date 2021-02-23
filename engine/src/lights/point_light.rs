use crate::math::Vector3;

pub struct PointLight {
	pub color: Vector3,
	pub intensity: f32,
	pub position: Vector3
}

impl PointLight {
	pub fn new() -> Self {
		Self {
			color: Vector3::from_scalar(1.0),
			intensity: 0.3,
			position: Vector3::new()
		}
	}
}

impl Default for PointLight {
	fn default() -> Self {
		Self::new()
	}
}