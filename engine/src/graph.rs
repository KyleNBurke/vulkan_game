use std::fmt;
use crate::{Camera, mesh::Mesh, Transform3D, lights::{AmbientLight, PointLight}, math::matrix4, pool::{Pool, Handle, Iter, IterMut}};

pub enum Object {
	Empty,
	Camera(Camera),
	AmbientLight(AmbientLight),
	PointLight(PointLight),
	Mesh(Mesh)
}

impl Object {
	pub fn as_camera(&self) -> &Camera {
		match self {
			Object::Camera(camera) => camera,
			_ => panic!("Cannot convert object {} to camera", self)
		}
	}

	pub fn as_camera_mut(&mut self) -> &mut Camera {
		match self {
			Object::Camera(camera) => camera,
			_ => panic!("Cannot convert object {} to camera", self)
		}
	}

	pub fn as_point_light(&self) -> &PointLight {
		match self {
			Object::PointLight(point_light) => point_light,
			_ => panic!("Cannot convert object {} to point light", self)
		}
	}

	pub fn as_mesh_mut(&mut self) -> &mut Mesh {
		match self {
			Object::Mesh(mesh) => mesh,
			_ => panic!("Cannot convert object {} to mesh", self)
		}
	}
}

impl fmt::Display for Object {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Object::Empty => write!(f, "empty"),
			Object::Camera(_) => write!(f, "camera"),
			Object::AmbientLight(_) => write!(f, "ambient light"),
			Object::PointLight(_) => write!(f, "point light"),
			Object::Mesh(_) => write!(f, "mesh"),
		}
	}
}

pub struct Node {
	parent_handle: Option<Handle>,
	child_handles: Vec<Handle>,
	pub transform: Transform3D,
	pub object: Object
}

impl Node {
	pub fn new(object: Object) -> Self {
		Self {
			parent_handle: None,
			child_handles: vec![],
			transform: Transform3D::new(),
			object
		}
	}

	pub fn parent_handle(&self) -> Option<Handle> {
		self.parent_handle
	}

	pub fn child_handles(&self) -> &[Handle] {
		&self.child_handles
	}
}

pub struct Graph {
	nodes: Pool<Node>,
	root_handle: Handle
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

	pub fn root_handle(&self) -> Handle {
		self.root_handle
	}

	pub fn add(&mut self, node: Node) -> Handle {
		self.add_to(self.root_handle, node)
	}

	pub fn add_to(&mut self, parent_handle: Handle, mut child_node: Node) -> Handle {
		child_node.parent_handle = Some(parent_handle);
		let child_handle = self.nodes.add(child_node);
		let parent = self.nodes.borrow_mut(parent_handle);
		parent.child_handles.push(child_handle);
		child_handle
	}

	pub fn remove_at(&mut self, handle: Handle) {
		if handle == self.root_handle {
			panic!("Cannot remove node from graph, handle points to root node");
		}

		let node = self.nodes.borrow(handle);
		if let Some(parent_handle) = node.parent_handle() {
			let parent_node = self.nodes.borrow_mut(parent_handle);
			let child_handle_index = parent_node.child_handles.iter().position(|h| *h == handle).unwrap();
			parent_node.child_handles.remove(child_handle_index);
		}

		let mut removes = vec![handle];

		while let Some(handle) = removes.pop() {
			let node = self.nodes.borrow(handle);
			removes.extend_from_slice(&node.child_handles);
			self.nodes.remove(handle);
		}
	}

	pub fn borrow(&self, handle: Handle) -> &Node {
		self.nodes.borrow(handle)
	}

	pub fn borrow_mut(&mut self, handle: Handle) -> &mut Node {
		self.nodes.borrow_mut(handle)
	}

	pub fn try_borrow(&self, handle: Handle) -> Option<&Node> {
		self.nodes.try_borrow(handle)
	}

	pub fn try_borrow_mut(&mut self, handle: Handle) -> Option<&mut Node> {
		self.nodes.try_borrow_mut(handle)
	}

	pub fn capacity(&self) -> usize {
		self.nodes.capacity()
	}

	pub fn node_count(&self) -> usize {
		self.nodes.occupied_record_count()
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

	pub fn update_at(&mut self, handle: Handle) {
		let mut updates = vec![handle];

		while let Some(child_handle) = updates.pop() {
			let child_node = self.nodes.borrow(child_handle);

			let parent_global_matrix = if let Some(parent_handle) = child_node.parent_handle {
				let parent_node = self.nodes.borrow(parent_handle);
				parent_node.transform.global_matrix
			}
			else {
				matrix4::IDENTITY
			};

			let child_node = self.nodes.borrow_mut(child_handle);

			if child_node.transform.auto_update_matrix {
				child_node.transform.update_matrix();
			}

			child_node.transform.global_matrix = parent_global_matrix * child_node.transform.matrix;
			updates.extend_from_slice(&child_node.child_handles);
		}
	}
}

impl Default for Graph {
	fn default() -> Self {
		Self::new()
	}
}