use crate::{
	Camera,
	lights::{AmbientLight, PointLight},
	Mesh,
	UIElement,
	Pool
};

pub struct Scene {
	pub camera: Camera,
	pub ambient_light: AmbientLight,
	pub point_lights: Pool<PointLight>,
	pub meshes: Pool<Mesh>,
	pub ui_elements: Pool<UIElement>
}

impl Scene {
	pub fn new(camera: Camera, ambient_light: AmbientLight) -> Self {
		Self {
			camera,
			ambient_light,
			point_lights: Pool::new(),
			meshes: Pool::new(),
			ui_elements: Pool::new()
		}
	}
}