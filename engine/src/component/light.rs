use crate::math::Vector3;

pub enum Light {
	PointLight(PointLight),
	AmbientLight(AmbientLight)
}

pub struct PointLight {
	pub color: Vector3,
	pub intensity: f32
}

pub struct AmbientLight {
	pub color: Vector3,
	pub intensity: f32
}

impl Light {
	pub fn as_point_light(&self) -> &PointLight {
		match self {
			Light::PointLight(point_light) => point_light,
			_ => panic!("Cannot cast Light to PointLight varient because it's not a PointLight")
		}
	}
}