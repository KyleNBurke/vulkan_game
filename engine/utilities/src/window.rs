use std::sync::mpsc::Receiver;

pub struct Window {
	pub glfw: glfw::Glfw,
	pub glfw_window: glfw::Window,
	pub events: Receiver<(f64, glfw::WindowEvent)>
}

impl Window {
	pub fn new(title: &str) -> Self {
		let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
		glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
		let (mut glfw_window, events) = glfw.create_window(1280, 720, title, glfw::WindowMode::Windowed).unwrap();
		glfw_window.set_framebuffer_size_polling(true);
		glfw_window.set_key_polling(true);

		Self {
			glfw,
			glfw_window,
			events
		}
	}

	pub fn main_loop<F>(&mut self, mut render: F) where
		F: FnMut(bool, i32, i32)
	{
		let mut minimized = false;
		let mut resized;
		let mut width = 0;
		let mut height = 0;

		while !self.glfw_window.should_close() {
			resized = false;
			self.glfw.poll_events();

			for (_, event) in glfw::flush_messages(&self.events) {
				match event {
					glfw::WindowEvent::FramebufferSize(new_width, new_height) => {
						if new_width == 0 && new_height == 0 {
							minimized = true;
						}
						else {
							if !minimized {
								resized = true;
								width = new_width;
								height = new_height;
							}

							minimized = false;
						}
					},
					glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
						self.glfw_window.set_should_close(true);
					},
					_ => ()
				}
			}

			if minimized {
				self.glfw.wait_events();
				continue;
			}

			render(resized, width, height);
		}
	}
}