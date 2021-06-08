use std::fmt;
use crate::{Camera, mesh::Mesh, Transform3D, lights::{AmbientLight, PointLight}, pool::{Pool, Handle, Iter}};

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

	pub fn as_mesh(&self) -> &Mesh {
		match self {
			Object::Mesh(mesh) => mesh,
			_ => panic!("Cannot convert object {} to mesh", self)
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
	pub transform: Transform3D, // you can never get a node mutably so this can just be public? Right now, you can create a mutable node and mutate this though
	dirty: bool,
	pub object: Object // you can never get a node mutably so this can just be public?
}

impl Node {
	pub fn new(object: Object) -> Self {
		Self {
			parent_handle: None,
			child_handles: vec![],
			transform: Transform3D::new(),
			dirty: false,
			object
		}
	}
}

pub struct Graph {
	nodes: Pool<Node>,
	dirty_handles: Vec<Handle>
}

impl Graph {
	pub fn new() -> Self {
		Self {
			nodes: Pool::new(),
			dirty_handles: vec![]
		}
	}

	pub fn add(&mut self, mut node: Node) -> Handle {
		let transform = &mut node.transform;
		transform.update_local_matrix();
		transform.global_matrix = *transform.local_matrix();

		self.nodes.add(node)
	}

	pub fn add_child(&mut self, parent_handle: Handle, mut child_node: Node) -> Handle {
		let parent_node = self.nodes.borrow_mut(parent_handle);
		let parent_global_matrix = &parent_node.transform.global_matrix;
		let child_transform = &mut child_node.transform;
		child_transform.update_local_matrix();
		child_transform.global_matrix = parent_global_matrix * child_transform.local_matrix();

		child_node.parent_handle = Some(parent_handle);
		let child_handle = self.nodes.add(child_node);
		let parent_node = self.nodes.borrow_mut(parent_handle);
		parent_node.child_handles.push(child_handle);

		child_handle
	}

	pub fn borrow_node(&self, handle: Handle) -> &Node {
		self.nodes.borrow(handle)
	}

	pub fn borrow_transform(&self, handle: Handle) -> &Transform3D {
		&self.nodes.borrow(handle).transform
	}

	pub fn borrow_transform_mut(&mut self, handle: Handle) -> &mut Transform3D {
		let mut handles_to_visit = vec![handle];

		while let Some(handle) = handles_to_visit.pop() {
			let node = self.nodes.borrow_mut(handle);
			if node.dirty { break };
			node.dirty = true;
			self.dirty_handles.push(handle);
			handles_to_visit.extend_from_slice(&node.child_handles);
		}

		&mut self.nodes.borrow_mut(handle).transform
	}

	pub fn borrow_object(&self, handle: Handle) -> &Object {
		&self.nodes.borrow(handle).object
	}

	pub fn borrow_object_mut(&mut self, handle: Handle) -> &mut Object {
		&mut self.nodes.borrow_mut(handle).object
	}

	pub fn update(&mut self) {
		while let Some(dirty_handle) = self.dirty_handles.pop() {
			let mut nodes_to_visit = vec![dirty_handle];

			while let Some(child_handle) = nodes_to_visit.pop() {
				let child_node = self.nodes.borrow_mut(child_handle);
				if !child_node.dirty { break };
				child_node.transform.update_local_matrix();
				child_node.dirty = false;
				nodes_to_visit.extend_from_slice(&child_node.child_handles);
				
				if let Some(parent_handle) = child_node.parent_handle {
					let parent_global_matrix = self.nodes.borrow(parent_handle).transform.global_matrix;
					let child_transform = &mut self.nodes.borrow_mut(dirty_handle).transform;
					child_transform.global_matrix = parent_global_matrix * child_transform.local_matrix();
				}
				else {
					let child_transform = &mut child_node.transform;
					child_transform.global_matrix = *child_transform.local_matrix();
				}
			}
		}
	}

	pub fn iter(&self) -> Iter<Node> {
		self.nodes.iter()
	}
}