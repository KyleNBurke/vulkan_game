use crate::{
	pool::{Handle, Pool},
	Camera,
	lights::{AmbientLight, PointLight},
	Transform3D,
	Geometry3D,
	Mesh,
	Font,
	Text
};

pub enum Object {
	Empty,
	Camera(Camera),
	AmbientLight(AmbientLight),
	PointLight(PointLight),
	Mesh(Mesh)
}

impl Object {
	pub fn camera(&self) -> Option<&Camera> {
		match self {
			Object::Camera(camera) => Some(camera),
			_ => None
		}
	}

	pub fn camera_mut(&mut self) -> Option<&mut Camera> {
		match self {
			Object::Camera(camera) => Some(camera),
			_ => None
		}
	}

	pub fn point_light(&self) -> Option<&PointLight> {
		match self {
			Object::PointLight(point_light) => Some(point_light),
			_ => None
		}
	}

	pub fn mesh_mut(&mut self) -> Option<&mut Mesh> {
		match self {
			Object::Mesh(mesh) => Some(mesh),
			_ => None
		}
	}
}

pub struct Node {
	pub parent: Handle<Self>,
	pub transform: Transform3D,
	pub object: Object
}

impl Node {
	pub fn new(object: Object) -> Self {
		Self {
			parent: Handle::null(),
			transform: Transform3D::new(),
			object
		}
	}
}

pub struct Scene {
	pub geometries: Pool<Geometry3D>,
	pub nodes: Pool<Node>,
	pub camera_handle: Handle<Node>,
	pub fonts: Pool<Font>,
	pub text: Pool<Text>
}

impl Scene {
	pub fn new() -> Self {
		Self {
			geometries: Pool::new(),
			nodes: Pool::new(),
			camera_handle: Handle::null(),
			fonts: Pool::new(),
			text: Pool::new()
		}
	}
}