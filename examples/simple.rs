extern crate vulkan_engine;

use vulkan_engine::Renderer;

fn main() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
	glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
	let (mut window, events) = glfw.create_window(300, 300, "Vulkan", glfw::WindowMode::Windowed).unwrap();
	window.set_key_polling(true);
	window.set_framebuffer_size_polling(true);

	let mut renderer = Renderer::new(&glfw, &window);

	while !window.should_close() {
		glfw.poll_events();

		for (_, event) in glfw::flush_messages(&events) {
			match event {
				glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
					window.set_should_close(true);
				}
				glfw::WindowEvent::FramebufferSize(_, _) => {
					renderer.framebuffer_resized = true
				},
				_ => {}
			}
		}

		renderer.render(&mut glfw, &window);
	}
}