mod vulkan;
use vulkan::Context;

mod renderer;
use renderer::Renderer;

mod mesh;
use mesh::Mesh;

mod geometry;

#[allow(dead_code)]
mod math;

mod object3d;
use object3d::Object3D;

mod camera;
use camera::Camera;

fn main() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
	glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
	let (mut window, events) = glfw.create_window(400, 400, "Vulkan", glfw::WindowMode::Windowed).unwrap();
	window.set_key_polling(true);
	window.set_framebuffer_size_polling(true);

	let context = Context::new(&glfw, &window);
	let (width, height) = window.get_framebuffer_size();
	let mut renderer = Renderer::new(&context, width as u32, height as u32);

	let mut static_triangle = Mesh::new(Box::new(geometry::Triangle {}));
	static_triangle.position = math::Vector3::from(0.0, 1.0, 1.7);
	static_triangle.update_matrix();

	let mut static_plane = Mesh::new(Box::new(geometry::Plane {}));
	static_plane.position = math::Vector3::from(0.0, 1.0, 2.0);
	static_plane.update_matrix();

	let mut static_box = Mesh::new(Box::new(geometry::Box {}));
	static_box.position = math::Vector3::from(-2.0, 0.0, 0.0);
	static_box.update_matrix();

	let static_meshes = vec![static_triangle, static_plane, static_box];
	renderer.submit_static_meshes(&static_meshes);

	let mut dynamic_triangle = Mesh::new(Box::new(geometry::Triangle {}));
	dynamic_triangle.position = math::Vector3::from(-0.5, -0.6, 2.0);

	let mut dynamic_plane = Mesh::new(Box::new(geometry::Plane {}));
	dynamic_plane.position = math::Vector3::from(0.5, -0.6, 2.0);

	let mut dynamic_meshes = vec![dynamic_triangle, dynamic_plane];

	let (mouse_pos_x, mouse_pos_y) = window.get_cursor_pos();
	let mut camera = Camera::new(1.0, 75.0, 0.1, 10.0, mouse_pos_x as f32, mouse_pos_y as f32);

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
				glfw::WindowEvent::Key(glfw::Key::R, _, glfw::Action::Press, _) => {
					renderer.submit_static_meshes(&static_meshes);
					println!("Static meshes submitted");
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
			camera.projection_matrix.make_perspective(width as f32 / height as f32, 75.0, 0.1, 10.0);
		}
		
		dynamic_meshes[0].rotate_y(0.0005);
		dynamic_meshes[0].update_matrix();

		dynamic_meshes[1].rotate_y(0.0005);
		dynamic_meshes[1].update_matrix();

		camera.update(&window);

		renderer.render(&window, &camera, &dynamic_meshes);
	}
}