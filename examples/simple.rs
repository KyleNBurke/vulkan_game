use engine::{
	vulkan::{Context, Renderer},
	mesh::{Mesh, Material},
	geometry3d,
	Object3D,
	Camera,
	lights::AmbientLight
};
fn main() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
	glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
	let (mut window, events) = glfw.create_window(1280, 720, "Simple", glfw::WindowMode::Windowed).unwrap();
	window.set_key_polling(true);
	window.set_framebuffer_size_polling(true);

	let context = Context::new(&glfw, &window);
	let (framebuffer_width, framebuffer_height) = window.get_framebuffer_size();
	let mut renderer = Renderer::new(&context, framebuffer_width as u32, framebuffer_height as u32);

	let mut meshes = [Mesh::new(Box::new(geometry3d::Box {}), Material::Basic)];

	let mut camera = Camera::new(framebuffer_width as f32 / framebuffer_height as f32, 75.0, 0.1, 10.0);
	camera.translate_z(-2.0);

	let ambient_light = AmbientLight::new();

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
				_ => {}
			}
		}

		let (framebuffer_width, framebuffer_height) = window.get_framebuffer_size();
		if framebuffer_width == 0 || framebuffer_height == 0 {
			glfw.wait_events();
			continue;
		}

		if framebuffer_resized {
			renderer.recreate_swapchain(framebuffer_width as u32, framebuffer_height as u32);
			camera.projection_matrix.make_perspective(framebuffer_width as f32 / framebuffer_height as f32, 75.0, 0.1, 10.0);
		}
		
		meshes[0].rotate_y(0.0001);

		renderer.render(&window, &mut camera, &mut meshes, &ambient_light, &[], &[]);
	}
}