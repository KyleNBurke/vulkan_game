use crate::{
	Pool,
	Camera,
	lights::{AmbientLight, PointLight},
	Mesh,
	Text
};

pub struct Scene {
	pub camera: Camera,
	pub ambient_light: AmbientLight,
	pub point_lights: Pool<PointLight>,
	pub meshes: Pool<Mesh>,
	pub text: Pool<Text>
}

impl Scene {
	pub fn new(camera: Camera, ambient_light: AmbientLight) -> Self {
		Self {
			camera,
			ambient_light,
			point_lights: Pool::new(),
			meshes: Pool::new(),
			text: Pool::new()
		}
	}
}