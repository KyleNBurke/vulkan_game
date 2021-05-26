use crate::math::{Box3, Vector3};

pub enum Topology {
	Triangle,
	Line
}

pub struct Geometry3D {
	indices: Vec<u16>,
	attributes: Vec<f32>,
	topology: Topology,
	bounding_box: Box3
}

impl Geometry3D {
	pub fn new(indices: Vec<u16>, attributes: Vec<f32>, topology: Topology) -> Self {
		let mut min = Vector3::from_scalar(f32::INFINITY);
		let mut max = Vector3::from_scalar(f32::NEG_INFINITY);

		let stride = match topology {
			Topology::Triangle => 6,
			Topology::Line => 3
		};

		for i in 0..(attributes.len() / stride) {
			let x = attributes[i * stride];
			let y = attributes[i * stride + 1];
			let z = attributes[i * stride + 2];

			min.x = min.x.min(x);
			min.y = min.y.min(y);
			min.z = min.z.min(z);

			max.x = max.x.max(x);
			max.y = max.y.max(y);
			max.z = max.z.max(z);
		}

		Self {
			indices,
			attributes,
			topology,
			bounding_box: Box3::new(min, max)
		}
	}

	pub fn indices(&self) -> &[u16] {
		&self.indices
	}

	pub fn attributes(&self) -> &[f32] {
		&self.attributes
	}

	pub fn topology(&self) -> &Topology {
		&self.topology
	}

	pub fn bounding_box(&self) -> &Box3 {
		&self.bounding_box
	}

	pub fn create_plane() -> Self {
		let indices = vec![
			0, 2, 1,
			0, 3, 2
		];

		let attributes = vec![
			 1.0, 0.0,  1.0, 0.0, 1.0, 0.0,
			-1.0, 0.0,  1.0, 0.0, 1.0, 0.0,
			-1.0, 0.0, -1.0, 0.0, 1.0, 0.0,
			 1.0, 0.0, -1.0, 0.0, 1.0, 0.0
		];

		Self::new(indices, attributes, Topology::Triangle)
	}

	pub fn create_box() -> Self {
		let indices = vec![
			0,  3,  2,  // top
			0,  2,  1,
			4,  6,  7,  // bottom
			4,  5,  6,
			8,  9,  10, // right
			8,  10, 11,
			12, 15, 13, // left
			13, 15, 14,
			16, 17, 18, // front
			16, 18, 19,
			20, 23, 22, // back
			20, 22, 21
		];

		let attributes = vec![
			 1.0,  1.0,  1.0,  0.0,  1.0,  0.0, // top
			-1.0,  1.0,  1.0,  0.0,  1.0,  0.0,
			-1.0,  1.0, -1.0,  0.0,  1.0,  0.0,
			 1.0,  1.0, -1.0,  0.0,  1.0,  0.0,
			 1.0, -1.0,  1.0,  0.0, -1.0,  0.0, // bottom
			-1.0, -1.0,  1.0,  0.0, -1.0,  0.0,
			-1.0, -1.0, -1.0,  0.0, -1.0,  0.0,
			 1.0, -1.0, -1.0,  0.0, -1.0,  0.0,
			-1.0,  1.0,  1.0, -1.0,  0.0,  0.0, // right
			-1.0,  1.0, -1.0, -1.0,  0.0,  0.0,
			-1.0, -1.0, -1.0, -1.0,  0.0,  0.0,
			-1.0, -1.0,  1.0, -1.0,  0.0,  0.0,
			 1.0,  1.0,  1.0,  1.0,  0.0,  0.0, // left
			 1.0,  1.0, -1.0,  1.0,  0.0,  0.0,
			 1.0, -1.0, -1.0,  1.0,  0.0,  0.0,
			 1.0, -1.0,  1.0,  1.0,  0.0,  0.0,
			 1.0,  1.0,  1.0,  0.0,  0.0,  1.0, // front
			-1.0,  1.0,  1.0,  0.0,  0.0,  1.0,
			-1.0, -1.0,  1.0,  0.0,  0.0,  1.0,
			 1.0, -1.0,  1.0,  0.0,  0.0,  1.0,
			 1.0,  1.0, -1.0,  0.0,  0.0, -1.0, // back
			-1.0,  1.0, -1.0,  0.0,  0.0, -1.0,
			-1.0, -1.0, -1.0,  0.0,  0.0, -1.0,
			 1.0, -1.0, -1.0,  0.0,  0.0, -1.0
		];

		Self::new(indices, attributes, Topology::Triangle)
	}

	pub fn create_axis_helper() -> Self {
		let indices = vec![0, 1, 0, 2, 0, 3];
	
		let attributes = vec![
			0.0, 0.0, 0.0,
			1.0, 0.0, 0.0,
			0.0, 1.0, 0.0,
			0.0, 0.0, 1.0
		];
	
		Self::new(indices, attributes, Topology::Line)
	}

	pub fn create_box_helper(box3: &Box3) -> Self {
		let min = &box3.min;
		let max = &box3.max;

		let indices = vec![0, 1, 1, 2, 2, 3, 3, 0, 0, 4, 1, 5, 2, 6, 3, 7, 4, 5, 5, 6, 6, 7, 7, 4];
	
		let attributes = vec![
			max.x, max.y, max.z,
			min.x, max.y, max.z,
			min.x, max.y, min.z,
			max.x, max.y, min.z,
			max.x, min.y, max.z,
			min.x, min.y, max.z,
			min.x, min.y, min.z,
			max.x, min.y, min.z,
		];
	
		Self::new(indices, attributes, Topology::Line)
	}
}