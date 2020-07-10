use crate::math::{Vector3, Quaternion, Matrix4, euler, Euler};
use crate::object3d::Object3D;

const TRANSLATION_SPEED: f32 = 0.001;
const ROTATION_SPEED: f32 = 0.003;
const MAX_VERTICAL_ROTATION_ANGLE: f32 = 1.57;

pub struct Camera {
	pub position: Vector3,
	pub rotation: Quaternion,
	pub scale: Vector3,
	pub view_matrix: Matrix4,
	pub auto_update_view_matrix: bool,
	pub projection_matrix: Matrix4,
	prev_mouse_pos_x: f32,
	prev_mouse_pos_y: f32,
	euler: Euler
}

impl Camera {
	pub fn new(aspect: f32, fov: f32, near: f32, far: f32, mouse_pos_x: f32, mouse_pos_y: f32) -> Self {
		let mut projection_matrix = Matrix4::new();
		projection_matrix.make_perspective(aspect, fov, near, far);

		Self {
			position: Vector3::new(),
			rotation: Quaternion::new(),
			scale: Vector3::from_scalar(1.0),
			view_matrix: Matrix4::new(),
			auto_update_view_matrix: true,
			projection_matrix,
			prev_mouse_pos_x: mouse_pos_x,
			prev_mouse_pos_y: mouse_pos_y,
			euler: Euler::from(0.0, 0.0, 0.0, euler::Order::YXZ)
		}
	}

	pub fn update(&mut self, window: &glfw::Window) {
		if window.get_key(glfw::Key::W) == glfw::Action::Press {
			self.translate_z(TRANSLATION_SPEED);
		}

		if window.get_key(glfw::Key::S) == glfw::Action::Press {
			self.translate_z(-TRANSLATION_SPEED);
		}

		if window.get_key(glfw::Key::A) == glfw::Action::Press {
			self.translate_x(-TRANSLATION_SPEED);
		}

		if window.get_key(glfw::Key::D) == glfw::Action::Press {
			self.translate_x(TRANSLATION_SPEED);
		}

		if window.get_key(glfw::Key::E) == glfw::Action::Press {
			self.translate_y(-TRANSLATION_SPEED);
		}

		if window.get_key(glfw::Key::Q) == glfw::Action::Press {
			self.translate_y(TRANSLATION_SPEED);
		}

		let (mouse_pos_x, mouse_pos_y) = window.get_cursor_pos();
		let mouse_pos_x = mouse_pos_x as f32;
		let mouse_pos_y = mouse_pos_y as f32;
		let mouse_pos_diff_x = mouse_pos_x - self.prev_mouse_pos_x;
		let mouse_pos_diff_y = mouse_pos_y - self.prev_mouse_pos_y;

		self.euler.set_from_quaternion(&self.rotation);
		self.euler.y += mouse_pos_diff_x * ROTATION_SPEED;
		self.euler.x -= mouse_pos_diff_y * ROTATION_SPEED;
		self.euler.x = self.euler.x.max(-MAX_VERTICAL_ROTATION_ANGLE).min(MAX_VERTICAL_ROTATION_ANGLE);
		self.rotation.set_from_euler(&self.euler);

		self.prev_mouse_pos_x = mouse_pos_x;
		self.prev_mouse_pos_y = mouse_pos_y;
	}
}

impl Object3D for Camera {
	fn get_position(&self) -> &Vector3 {
		&self.position
	}

	fn get_position_mut(&mut self) -> &mut Vector3 {
		&mut self.position
	}

	fn get_rotation(&self) -> &Quaternion {
		&self.rotation
	}

	fn get_rotation_mut(&mut self) -> &mut Quaternion {
		&mut self.rotation
	}

	fn get_scale(&self) -> &Vector3 {
		&self.scale
	}

	fn get_scale_mut(&mut self) -> &mut Vector3 {
		&mut self.scale
	}

	fn get_matrix(&self) -> &Matrix4 {
		&self.view_matrix
	}

	fn get_matrix_mut(&mut self) -> &mut Matrix4 {
		&mut self.view_matrix
	}
}