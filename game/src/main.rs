use std::time::{Instant, Duration};
use engine::glfw;

mod component;
mod system;

mod camera_controller;
pub use camera_controller::CameraController;

mod game;
use game::Game;

const MAX_FRAME_TIME: f64 = 1.0 / 10.0;
const MAX_UPDATES_PER_FRAME: u32 = 5;

fn main() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
	glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
	let (mut window, events) = glfw.create_window(1280, 720, "Vulkan", glfw::WindowMode::Windowed).unwrap();
	window.set_framebuffer_size_polling(true);
	window.set_key_polling(true);

	let mut game = Game::new(&glfw, &window);

	let duration_zero = Duration::new(0, 0);
	let max_duration = Duration::from_secs_f64(MAX_FRAME_TIME);
	let mut frame_start = Instant::now();

	let mut minimized = false;
	let mut resized;
	let mut width = 0;
	let mut height = 0;
	let mut surface_changed = false;

	while !window.should_close() {
		resized = false;
		glfw.poll_events();

		for (_, event) in glfw::flush_messages(&events) {
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
				glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => window.set_should_close(true),
				_ => ()
			}

			game.handle_event(&event, &mut window);
		}

		if minimized {
			glfw.wait_events();
			continue;
		}

		if resized || surface_changed {
			game.handle_resize(width, height);
		}

		let frame_end = Instant::now();
		let mut duration = frame_end.duration_since(frame_start);
		frame_start = frame_end;
		let mut updates = 0;

		while duration > duration_zero && updates <= MAX_UPDATES_PER_FRAME {
			let duration_capped = duration.min(max_duration);
			
			game.update(&window, &duration_capped);
			
			duration -= duration_capped;
			updates += 1;
		}

		surface_changed = game.render();
	}
}