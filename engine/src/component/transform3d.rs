use crate::math::{matrix4, Matrix4, Quaternion, Vector3, quaternion, vector3};

pub struct Transform3D {
	pub(crate) parent_entity: Option<usize>,
	pub(crate) child_entities: Vec<usize>,
	pub(crate) dirty: bool,
	pub position: Vector3,
	pub orientation: Quaternion,
	pub scale: Vector3,
	pub(crate) local_matrix: Matrix4,
	pub(crate) global_matrix: Matrix4
}

impl Transform3D {
	pub fn new() -> Self {
		Self {
			parent_entity: None,
			child_entities: Vec::new(),
			dirty: false,
			position: vector3::ZERO,
			orientation: quaternion::ZERO,
			scale: Vector3::from_scalar(1.0),
			local_matrix: matrix4::IDENTITY,
			global_matrix: matrix4::IDENTITY
		}
	}

	pub fn local_matrix(&self) -> &Matrix4 {
		&self.local_matrix
	}

	pub fn global_matrix(&self) -> &Matrix4 {
		&self.global_matrix
	}

	pub fn update_local_matrix(&mut self) {
		self.local_matrix.compose(&self.position, &self.orientation, &self.scale);
	}

	pub fn translate_on_axis(&mut self, mut axis: Vector3, distance: f32) {
		axis.apply_quaternion(&self.orientation);
		self.position += axis * distance;
	}

	pub fn translate_x(&mut self, distance: f32) {
		self.translate_on_axis(vector3::UNIT_X, distance);
	}

	pub fn translate_y(&mut self, distance: f32) {
		self.translate_on_axis(vector3::UNIT_Y, distance);
	}

	pub fn translate_z(&mut self, distance: f32) {
		self.translate_on_axis(vector3::UNIT_Z, distance);
	}

	pub fn rotate_on_axis(&mut self, axis: &Vector3, angle: f32) {
		let mut quat = quaternion::ZERO;
		quat.set_from_axis_angle(axis, angle);
		self.orientation *= quat;
	}

	pub fn rotate_x(&mut self, angle: f32) {
		self.rotate_on_axis(&vector3::UNIT_X, angle);
	}

	pub fn rotate_y(&mut self, angle: f32) {
		self.rotate_on_axis(&vector3::UNIT_Y, angle);
	}

	pub fn rotate_z(&mut self, angle: f32) {
		self.rotate_on_axis(&vector3::UNIT_Z, angle);
	}
}