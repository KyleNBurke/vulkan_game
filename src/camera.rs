use crate::math::{Vector3, Quaternion, Matrix4};
use crate::Object3D;

const TRANSLATION_SPEED: f32 = 0.001;
const ROTATION_SPEED: f32 = 0.003;

pub struct Camera {
	pub position: Vector3,
	pub rotation: Quaternion,
	pub scale: Vector3,
	pub view_matrix: Matrix4,
	pub projection_matrix: Matrix4,
	prev_mouse_pos_x: f32,
	prev_mouse_pos_y: f32
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
			projection_matrix,
			prev_mouse_pos_x: mouse_pos_x,
			prev_mouse_pos_y: mouse_pos_y
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

		let (mouse_pos_x, mouse_pos_y) = window.get_cursor_pos();
		let mouse_pos_x = mouse_pos_x as f32;
		let mouse_pos_y = mouse_pos_y as f32;
		let mouse_pos_diff_x = mouse_pos_x - self.prev_mouse_pos_x;

		self.rotate_y(mouse_pos_diff_x * ROTATION_SPEED);

		self.prev_mouse_pos_x = mouse_pos_x;
		self.prev_mouse_pos_y = mouse_pos_y;
	}
}

impl Object3D for Camera {
	fn position(&self) -> &Vector3 {
		&self.position
	}

	fn position_mut(&mut self) -> &mut Vector3 {
		&mut self.position
	}

	fn rotation(&self) -> &Quaternion {
		&self.rotation
	}

	fn rotation_mut(&mut self) -> &mut Quaternion {
		&mut self.rotation
	}

	fn scale(&self) -> &Vector3 {
		&self.scale
	}

	fn scale_mut(&mut self) -> &mut Vector3 {
		&mut self.scale
	}

	fn matrix(&self) -> &Matrix4 {
		&self.view_matrix
	}

	fn matrix_mut(&mut self) -> &mut Matrix4 {
		&mut self.view_matrix
	}
}