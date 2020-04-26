mod renderer;
use renderer::Renderer;

mod mesh;
use mesh::Mesh;

mod geometry;
mod math;

fn main() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
	glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
	let (mut window, events) = glfw.create_window(400, 400, "Vulkan", glfw::WindowMode::Windowed).unwrap();
	window.set_key_polling(true);
	window.set_framebuffer_size_polling(true);

	let mut renderer = Renderer::new(&glfw, &window);
	
	let mut triangle = Mesh::new(Box::new(geometry::Triangle {}));
	triangle.model_matrix.set([
		[0.8, 0.0, 0.0, -0.5],
		[0.0, 0.8, 0.0, 0.0],
		[0.0, 0.0, 0.8, 0.0],
		[0.0, 0.0, 0.0, 1.0]
	]);

	let mut plane = Mesh::new(Box::new(geometry::Plane {}));
	plane.model_matrix.set([
		[0.8, 0.0, 0.0, 0.5],
		[0.0, 0.8, 0.0, 0.0],
		[0.0, 0.0, 0.8, 0.0],
		[0.0, 0.0, 0.0, 1.0]
	]);

	let meshes = vec![triangle, plane];
	
	renderer.submit_static_meshes(&meshes);

	while !window.should_close() {
		let mut framebuffer_resized = false;
		glfw.poll_events();

		for (_, event) in glfw::flush_messages(&events) {
			match event {
				glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
					window.set_should_close(true);
				},
				glfw::WindowEvent::FramebufferSize(_, _) => {
					framebuffer_resized = true;
				},
				glfw::WindowEvent::Key(glfw::Key::Q, _, glfw::Action::Press, _) => {
					renderer.submit_static_meshes(&meshes);
				},
				_ => {}
			}
		}

		let (width, height) = window.get_framebuffer_size();
		if width == 0 || height == 0 {
			glfw.wait_events();
			continue;
		}

		if framebuffer_resized {
			renderer.recreate_swapchain(width as u32, height as u32);
			renderer.submit_static_meshes(&meshes);
		}

		renderer.render(&window);
	}
}