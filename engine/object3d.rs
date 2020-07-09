use crate::math::{vector3, Vector3, Quaternion, Matrix4};

pub trait Object3D {
	fn get_position(&self) -> &Vector3;
	fn get_position_mut(&mut self) -> &mut Vector3;

	fn get_rotation(&self) -> &Quaternion;
	fn get_rotation_mut(&mut self) -> &mut Quaternion;

	fn get_scale(&self) -> &Vector3;
	fn get_scale_mut(&mut self) -> &mut Vector3;

	fn get_matrix(&self) -> &Matrix4;
	fn get_matrix_mut(&mut self) -> &mut Matrix4;

	fn update_matrix(&mut self) {
		let position = *self.get_position();
		let rotation = *self.get_rotation();
		let scale = *self.get_scale();

		self.get_matrix_mut().compose(&position, &rotation, &scale);
	}

	fn translate_on_axis(&mut self, axis: &Vector3, distance: f32) {
		let mut object_space_axis = *axis;
		object_space_axis.apply_quaternion(self.get_rotation());
		*self.get_position_mut() += object_space_axis * distance;
	}

	fn translate_x(&mut self, distance: f32) {
		self.translate_on_axis(&vector3::UNIT_X, distance);
	}

	fn translate_y(&mut self, distance: f32) {
		self.translate_on_axis(&vector3::UNIT_Y, distance);
	}

	fn translate_z(&mut self, distance: f32) {
		self.translate_on_axis(&vector3::UNIT_Z, distance);
	}

	fn rotate_on_axis(&mut self, axis: &Vector3, angle: f32) {
		let mut q = Quaternion::new();
		q.set_from_axis_angle(axis, angle);
		*self.get_rotation_mut() *= q;
	}

	fn rotate_x(&mut self, angle: f32) {
		self.rotate_on_axis(&vector3::UNIT_X, angle);
	}

	fn rotate_y(&mut self, angle: f32) {
		self.rotate_on_axis(&vector3::UNIT_Y, angle);
	}

	fn rotate_z(&mut self, angle: f32) {
		self.rotate_on_axis(&vector3::UNIT_Z, angle);
	}
}