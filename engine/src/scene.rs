use crate::{
	Camera,
	lights::{AmbientLight, PointLight},
	Mesh,
	UIElement
};

pub struct Scene {
	pub camera: Camera,
	pub ambient_light: AmbientLight,
	pub point_lights: Vec<PointLight>,
	pub meshes: Vec<Mesh>,
	pub ui_elements: Vec<UIElement>
}