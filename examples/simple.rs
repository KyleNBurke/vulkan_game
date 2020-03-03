extern crate vulkan_engine;

use vulkan_engine::Renderer;

fn main() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
	glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
	let mut window = glfw.create_window(300, 300, "Vulkan", glfw::WindowMode::Windowed).unwrap().0;

	let mut renderer = Renderer::new(&window);

	while !window.should_close() {
		glfw.poll_events();

		if window.get_key(glfw::Key::Escape) == glfw::Action::Press {
			window.set_should_close(true);
		}

		renderer.render();
	}
}