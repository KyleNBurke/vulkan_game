use crate::{
	Pool,
	Handle,
	Camera,
	lights::{AmbientLight, PointLight},
	Geometry3D,
	Mesh,
	Text,
	Transform3D
};

pub enum Entity {
	PointLight(Handle<PointLight>),
	Mesh(Handle<Mesh>),
	None
}

pub struct Node {
	pub parent_handle: Option<Handle<Node>>,
	pub child_handles: Vec<Handle<Node>>,
	pub entity: Entity,
	pub transform: Transform3D
}

pub struct Graph {
	nodes: Pool<Node>,
	root_handle: Handle<Node>
}

pub struct TraverseIter<'a> {
	nodes: &'a Pool<Node>,
	nodes_to_visit: Vec<&'a Node>
}

pub struct TraverseIterMut<'a> {
	nodes: &'a mut Pool<Node>,
	nodes_to_visit: Vec<*mut Node>
}

pub struct Scene {
	pub camera: Camera,
	pub ambient_light: AmbientLight,
	pub point_lights: Pool<PointLight>,
	pub geometries: Pool<Geometry3D>,
	pub meshes: Pool<Mesh>,
	pub text: Pool<Text>,
	pub graph: Graph
}

impl Scene {
	pub fn new(camera: Camera, ambient_light: AmbientLight) -> Self {
		Self {
			camera,
			ambient_light,
			point_lights: Pool::new(),
			geometries: Pool::new(),
			meshes: Pool::new(),
			text: Pool::new(),
			graph: Graph::new()
		}
	}
}

impl Graph {
	fn new() -> Self {
		let mut nodes = Pool::new();

		let root_handle = nodes.add(Node {
			parent_handle: None,
			child_handles: vec![],
			entity: Entity::None,
			transform: Transform3D::new()
		});

		Self {
			nodes,
			root_handle
		}
	}

	pub fn get_root_handle(&self) -> Handle<Node> {
		self.root_handle
	}

	pub fn add_node(&mut self, entity: Entity) -> Handle<Node> {
		self.add_child_node(self.root_handle, entity).unwrap()
	}

	pub fn add_child_node(&mut self, parent_handle: Handle<Node>, entity: Entity) -> Option<Handle<Node>> {
		if !self.nodes.valid(&parent_handle) {
			return None;
		}

		let child_handle = self.nodes.add(Node {
			parent_handle: Some(parent_handle),
			child_handles: vec![],
			entity,
			transform: Transform3D::new()
		});

		let parent = self.nodes.get_mut(&parent_handle).unwrap();
		parent.child_handles.push(child_handle);

		Some(child_handle)
	}

	pub fn get_node(&self, handle: &Handle<Node>) -> Option<&Node> {
		self.nodes.get(handle)
	}

	pub fn get_node_mut(&mut self, handle: &Handle<Node>) -> Option<&mut Node> {
		self.nodes.get_mut(handle)
	}

	pub fn traverse_iter(&self) -> TraverseIter {
		let root_node = self.nodes.get(&self.root_handle).unwrap();

		TraverseIter {
			nodes: &self.nodes,
			nodes_to_visit: vec![root_node]
		}
	}

	pub fn traverse_iter_mut(&mut self) -> TraverseIterMut {
		let root_node = self.nodes.get_mut(&self.root_handle).unwrap() as *mut Node;

		TraverseIterMut {
			nodes: &mut self.nodes,
			nodes_to_visit: vec![root_node]
		}
	}
}

impl<'a> Iterator for TraverseIter<'a> {
	type Item = &'a Node;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(node) = self.nodes_to_visit.pop() {
			let mut children: Vec<&Node> = node.child_handles.iter()
				.rev()
				.map(|handle| self.nodes.get(handle).unwrap())
				.collect();
			
			self.nodes_to_visit.append(&mut children);

			Some(node)
		}
		else {
			None
		}
	}
}

impl<'a> Iterator for TraverseIterMut<'a> {
	type Item = &'a mut Node;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(node) = self.nodes_to_visit.pop() {
			let node = unsafe { &mut *node };

			let mut children: Vec<*mut Node> = node.child_handles.iter()
				.rev()
				.map(|handle| self.nodes.get_mut(handle).unwrap() as *mut Node)
				.collect();
			
			self.nodes_to_visit.append(&mut children);
			
			Some(node)
		}
		else {
			None
		}
	}
}