use crate::{
	pool::Pool,
	Camera,
	lights::{AmbientLight, PointLight},
	Geometry3D,
	Mesh,
	Font,
	Text
};

pub struct Scene {
	pub camera: Camera,
	pub ambient_light: AmbientLight,
	pub point_lights: Pool<PointLight>,
	pub geometries: Pool<Geometry3D>,
	pub meshes: Pool<Mesh>,
	pub fonts: Pool<Font>,
	pub text: Pool<Text>
}

impl Scene {
	pub fn new(camera: Camera, ambient_light: AmbientLight) -> Self {
		Self {
			camera,
			ambient_light,
			point_lights: Pool::new(),
			geometries: Pool::new(),
			meshes: Pool::new(),
			fonts: Pool::new(),
			text: Pool::new()
		}
	}
}