use crate::{
	Camera,
	lights::{AmbientLight, PointLight},
	Mesh,
	UIElement,
	Empty
};

pub struct Scene {
	pub camera: Camera,
	pub ambient_light: AmbientLight,
	pub point_lights: Vec<PointLight>,
	pub meshes: Vec<Mesh>,
	pub ui_elements: Vec<UIElement>
}

pub enum SceneObject {
	Mesh(Mesh),
	PointLight(PointLight),
	Empty(Empty)
}

#[derive(Copy, Clone)]
pub struct Handle {
	index: usize,
	generation: u32
}

pub struct Node {
	parent: Option<Handle>,
	pub object: SceneObject,
	children: Vec<Handle>
}

impl Node {
	pub fn get_parent(&self) -> Option<&Handle> {
		self.parent.as_ref()
	}

	pub fn get_children(&self) -> &Vec<Handle> {
		&self.children
	}
}

struct Record {
	generation: u32,
	node: Option<Node>
}

pub struct SceneGraph {
	records: Vec<Record>,
	vacant_records: Vec<usize>,
	pub camera: Camera,
	pub ambient_light: AmbientLight
}

pub struct DepthFirstIter<'a> {
	records: &'a Vec<Record>,
	records_to_visit: Vec<usize>
}

impl SceneGraph {
	pub fn new(camera: Camera, ambient_light: AmbientLight) -> Self {
		let root = Record {
			generation: 0,
			node: Some(Node {
				parent: None,
				object: SceneObject::Empty(Empty::new()),
				children: vec![]
			})
		};

		Self {
			records: vec![root],
			vacant_records: vec![],
			camera,
			ambient_light
		}
	}

	fn valid_handle(&self, handle: &Handle) -> bool {
		handle.index < self.records.len() && handle.generation == self.records[handle.index].generation && self.records[handle.index].node.is_some()
	}

	pub fn get_root_handle(&self) -> Handle {
		Handle {
			generation: 0,
			index: 0
		}
	}

	/*pub fn get_object(&self, handle: &Handle) -> Option<&SceneObject> {
		if handle.index >= self.records.len() || handle.generation != self.records[handle.index].generation {
			return None;
		}

		if let Some(node) = self.records[handle.index].node.as_ref() {
			Some(&node.object)
		}
		else {
			None
		}
	}

	pub fn get_object_mut(&mut self, handle: &Handle) -> Option<&mut SceneObject> {
		if handle.index >= self.records.len() || handle.generation != self.records[handle.index].generation {
			return None;
		}

		if let Some(node) = self.records[handle.index].node.as_mut() {
			Some(&mut node.object)
		}
		else {
			None
		}
	}*/

	pub fn get_node(&self, handle: &Handle) -> Option<&Node> {
		if handle.index < self.records.len() && handle.generation == self.records[handle.index].generation {
			self.records[handle.index].node.as_ref()
		}
		else {
			None
		}
	}

	pub fn get_node_mut(&mut self, handle: &Handle) -> Option<&mut Node> {
		if handle.index < self.records.len() && handle.generation == self.records[handle.index].generation {
			self.records[handle.index].node.as_mut()
		}
		else {
			None
		}
	}

	pub fn add_child(&mut self, parent: Handle, object: SceneObject) -> Option<Handle> {
		if !self.valid_handle(&parent) {
			return None;
		}

		let handle = if let Some(index) = self.vacant_records.pop() {
			let record = &mut self.records[index];
			let new_generation = record.generation + 1;

			record.generation = new_generation;
			record.node = Some(Node {
				parent: Some(parent),
				object,
				children: vec![]
			});

			Handle {
				generation: new_generation,
				index
			}
		}
		else {
			let generation = 0;

			let record = Record {
				generation,
				node: Some(Node {
					parent: Some(parent),
					object,
					children: vec![]
				})
			};

			self.records.push(record);

			Handle {
				generation,
				index: self.records.len() - 1
			}
		};

		let parent_node = self.records[parent.index].node.as_mut().unwrap();
		parent_node.children.push(handle);

		Some(handle)
	}

	pub fn add(&mut self, object: SceneObject) -> Handle {
		self.add_child(self.get_root_handle(), object).unwrap() // break up add_child because this doesn't need to validate against the root nor use a Option<Handle>
	}

	pub fn depth_first_iter_mut(&self) -> DepthFirstIter{
		DepthFirstIter {
			records: &self.records,
			records_to_visit: vec![0]
		}
	}
}

impl<'a> Iterator for DepthFirstIter<'a> {
	type Item = Handle;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(index) = self.records_to_visit.pop() {
			let record = &self.records[index];
			let node = record.node.as_ref().unwrap();

			let mut children: Vec<usize> = node.children.iter()
				.rev()
				.map(|handle| handle.index)
				.collect();

			self.records_to_visit.append(&mut children);

			Some(Handle {
				index,
				generation: record.generation
			})
		}
		else {
			None
		}
	}
}