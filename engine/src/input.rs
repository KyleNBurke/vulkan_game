pub struct Input {
	pressed_keys: Vec<glfw::Key>,
	released_keys: Vec<glfw::Key>
}

impl Input {
	pub fn new() -> Self {
		Self {
			pressed_keys: vec![],
			released_keys: vec![]
		}
	}

	pub fn set_key_action(&mut self, key: glfw::Key, action: glfw::Action) {
		match action {
			glfw::Action::Press => self.pressed_keys.push(key),
			glfw::Action::Release => self.released_keys.push(key),
			glfw::Action::Repeat => ()
		}
	}

	pub fn key_pressed(&self, key: glfw::Key) -> bool {
		self.pressed_keys.iter().find(|&&k| k == key).is_some()
	}

	pub fn key_released(&self, key: glfw::Key) -> bool {
		self.released_keys.iter().find(|&&k| k == key).is_some()
	}

	pub fn clear(&mut self) {
		self.pressed_keys.clear();
		self.released_keys.clear();
	}
}