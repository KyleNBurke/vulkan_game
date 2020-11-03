use engine::{
	Camera,
	math::{Euler, Order},
	Object3D
};

const TRANSLATION_SPEED: f32 = 0.001;
const ROTATION_SPEED: f32 = 0.003;
const MAX_VERTICAL_ROTATION_ANGLE: f32 = 1.57;

pub struct CameraController {
	prev_mouse_pos_x: f32,
	prev_mouse_pos_y: f32,
	euler: Euler
}

impl CameraController {
	pub fn new(window: &glfw::Window) -> Self {
		let (mouse_pos_x, mouse_pos_y) = window.get_cursor_pos();

		Self {
			prev_mouse_pos_x: mouse_pos_x as f32,
			prev_mouse_pos_y: mouse_pos_y as f32,
			euler: Euler::from(0.0, 0.0, 0.0, Order::YXZ)
		}
	}

	pub fn set_mouse_pos(&mut self, window: &glfw::Window) {
		let (mouse_pos_x, mouse_pos_y) = window.get_cursor_pos();

		self.prev_mouse_pos_x = mouse_pos_x as f32;
		self.prev_mouse_pos_y = mouse_pos_y as f32;
	}

	pub fn update(&mut self, window: &glfw::Window, camera: &mut Camera) {
		if window.get_key(glfw::Key::W) == glfw::Action::Press {
			camera.translate_z(TRANSLATION_SPEED);
		}

		if window.get_key(glfw::Key::S) == glfw::Action::Press {
			camera.translate_z(-TRANSLATION_SPEED);
		}

		if window.get_key(glfw::Key::A) == glfw::Action::Press {
			camera.translate_x(-TRANSLATION_SPEED);
		}

		if window.get_key(glfw::Key::D) == glfw::Action::Press {
			camera.translate_x(TRANSLATION_SPEED);
		}

		if window.get_key(glfw::Key::E) == glfw::Action::Press {
			camera.translate_y(-TRANSLATION_SPEED);
		}

		if window.get_key(glfw::Key::Q) == glfw::Action::Press {
			camera.translate_y(TRANSLATION_SPEED);
		}

		let (mouse_pos_x, mouse_pos_y) = window.get_cursor_pos();
		let mouse_pos_x = mouse_pos_x as f32;
		let mouse_pos_y = mouse_pos_y as f32;
		let mouse_pos_diff_x = mouse_pos_x - self.prev_mouse_pos_x;
		let mouse_pos_diff_y = mouse_pos_y - self.prev_mouse_pos_y;

		self.euler.set_from_quaternion(&camera.rotation);
		self.euler.y += mouse_pos_diff_x * ROTATION_SPEED;
		self.euler.x -= mouse_pos_diff_y * ROTATION_SPEED;
		self.euler.x = self.euler.x.max(-MAX_VERTICAL_ROTATION_ANGLE).min(MAX_VERTICAL_ROTATION_ANGLE);
		camera.rotation.set_from_euler(&self.euler);

		self.prev_mouse_pos_x = mouse_pos_x;
		self.prev_mouse_pos_y = mouse_pos_y;
	}
}