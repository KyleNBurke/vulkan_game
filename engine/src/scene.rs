use crate::{pool::{Handle, Pool}, Geometry3D, graph::Graph, Font, Text};

pub struct Scene {
	pub geometries: Pool<Geometry3D>,
	pub graph: Graph,
	pub camera_handle: Handle,
	pub fonts: Pool<Font>,
	pub text: Pool<Text>
}

impl Scene {
	pub fn new() -> Self {
		Self {
			geometries: Pool::new(),
			graph: Graph::new(),
			camera_handle: Handle::null(),
			fonts: Pool::new(),
			text: Pool::new()
		}
	}
}