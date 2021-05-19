use crate::{Camera, Mesh, Transform3D, lights::{AmbientLight, PointLight}, math::{Matrix4, matrix4}, pool::{Pool, Handle, Iter, IterMut}};

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
	parent: Option<Handle<Self>>, // parent handle
	children: Vec<Handle<Self>>,
	pub transform: Transform3D,
	pub object: Object
}

impl Node {
	pub fn new(object: Object) -> Self {
		Self {
			parent: None,
			children: vec![],
			transform: Transform3D::new(),
			object
		}
	}

	pub fn parent(&self) -> Option<Handle<Self>> {
		self.parent
	}
}

pub struct Graph {
	nodes: Pool<Node>,
	root_handle: Handle<Node>
}

impl Graph {
	pub fn new() -> Self {
		let mut nodes = Pool::new();
		let root_handle = nodes.add(Node::new(Object::Empty));

		Self {
			nodes,
			root_handle
		}
	}

	pub fn root_handle(&self) -> Handle<Node> {
		self.root_handle
	}

	pub fn add(&mut self, node: Node) -> Handle<Node> {
		self.add_to(self.root_handle, node).unwrap()
	}

	pub fn add_to(&mut self, parent_handle: Handle<Node>, mut node: Node) -> Option<Handle<Node>> {
		if !self.nodes.handle_valid(&parent_handle) {
			return None;
		}

		node.parent = Some(parent_handle);
		let child_handle = self.nodes.add(node);
		let parent = self.nodes.get_mut(&parent_handle).unwrap();
		parent.children.push(child_handle);
		Some(child_handle)
	}

	pub fn remove(&mut self, handle: Handle<Node>) {
		todo!()
	}

	pub fn get(&self, handle: &Handle<Node>) -> Option<&Node> {
		self.nodes.get(handle)
	}

	pub fn get_mut(&mut self, handle: &Handle<Node>) -> Option<&mut Node> {
		self.nodes.get_mut(handle)
	}

	pub fn total_len(&self) -> usize {
		self.nodes.total_len()
	}

	pub fn available_len(&self) -> usize {
		self.nodes.available_len()
	}

	pub fn iter(&self) -> Iter<Node> {
		self.nodes.iter()
	}

	pub fn iter_mut(&mut self) -> IterMut<Node> {
		self.nodes.iter_mut()
	}

	pub fn update(&mut self) {
		self.update_at(self.root_handle);
	}

	pub fn update_at(&mut self, handle: Handle<Node>) {
		let mut updates = vec![handle];

		while let Some(child_handle) = updates.pop() {
			let child = self.nodes.get(&child_handle).unwrap();

			let parent_global_matrix = if let Some(parent_handle) = child.parent() {
				let parent = self.nodes.get(&parent_handle).unwrap();
				parent.transform.global_matrix
			}
			else {
				Matrix4::new()
			};

			let child = self.nodes.get_mut(&child_handle).unwrap();

			if child.transform.auto_update_matrix {
				child.transform.update_matrix();
			}

			child.transform.global_matrix = parent_global_matrix * &child.transform.matrix;
			updates.extend_from_slice(&child.children);
		}
	}
}