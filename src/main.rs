mod vulkan;

mod renderer;
use renderer::Renderer;

mod mesh;
use mesh::Mesh;

mod geometry;
mod math;

use std::time;

fn main() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
	glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
	let (mut window, events) = glfw.create_window(400, 400, "Vulkan", glfw::WindowMode::Windowed).unwrap();
	window.set_key_polling(true);
	window.set_framebuffer_size_polling(true);

	let context = vulkan::Context::new(&glfw, &window);

	let mut renderer = Renderer::new(&glfw, &window);
	
	let mut static_triangle = Mesh::new(Box::new(geometry::Triangle {}));
	static_triangle.model_matrix.set([
		[1.0, 0.0, 0.0, -0.5],
		[0.0, 1.0, 0.0, 1.1],
		[0.0, 0.0, 1.0, 0.0],
		[0.0, 0.0, 0.0, 1.0]
	]);

	let mut static_plane = Mesh::new(Box::new(geometry::Plane {}));
	static_plane.model_matrix.set([
		[1.0, 0.0, 0.0, 0.5],
		[0.0, 1.0, 0.0, 1.1],
		[0.0, 0.0, 1.0, 0.0],
		[0.0, 0.0, 0.0, 1.0]
	]);

	let static_meshes = vec![static_triangle, static_plane];
	renderer.submit_static_meshes(&static_meshes);

	let mut dynamic_triangle = Mesh::new(Box::new(geometry::Triangle {}));
	dynamic_triangle.model_matrix.set([
		[1.0, 0.0, 0.0, -0.5],
		[0.0, 1.0, 0.0, 0.0],
		[0.0, 0.0, 1.0, 0.0],
		[0.0, 0.0, 0.0, 1.0]
	]);

	let mut dynamic_plane = Mesh::new(Box::new(geometry::Plane {}));
	dynamic_plane.model_matrix.set([
		[1.0, 0.0, 0.0, 0.5],
		[0.0, 1.0, 0.0, 0.0],
		[0.0, 0.0, 1.0, 0.0],
		[0.0, 0.0, 0.0, 1.0]
	]);

	let mut dynamic_meshes = vec![dynamic_triangle, dynamic_plane];

	let projection_matrix = math::Matrix4::from([
		[0.8, 0.0, 0.0, 0.0],
		[0.0, 0.8, 0.0, 0.0],
		[0.0, 0.0, 0.8, 0.0],
		[0.0, 0.0, 0.0, 1.0]
	]);

	let view_matrix = math::Matrix4::from([
		[1.0, 0.0, 0.0, 0.0],
		[0.0, 1.0, 0.0, -0.5],
		[0.0, 0.0, 1.0, 0.0],
		[0.0, 0.0, 0.0, 1.0]
	]);

	let timer = time::Instant::now();

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
					renderer.submit_static_meshes(&static_meshes);
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
		}

		let elapsed = timer.elapsed().as_secs_f32();
		
		dynamic_meshes[0].model_matrix.set([
			[elapsed.cos(), -elapsed.sin(), 0.0, -0.5],
			[elapsed.sin(), elapsed.cos(), 0.0, 0.0],
			[0.0, 0.0, 1.0, 0.0],
			[0.0, 0.0, 0.0, 1.0]
		]);

		dynamic_meshes[1].model_matrix.set([
			[elapsed.cos(), -elapsed.sin(), 0.0, 0.5],
			[elapsed.sin(), elapsed.cos(), 0.0, 0.0],
			[0.0, 0.0, 1.0, 0.0],
			[0.0, 0.0, 0.0, 1.0]
		]);

		renderer.render(&window, &projection_matrix, &view_matrix, &dynamic_meshes);
	}
}