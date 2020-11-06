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
	window.set_framebuffer_size_polling(true);
	window.set_key_polling(true);

	let context = Context::new(&glfw, &window);
	let (framebuffer_width, framebuffer_height) = window.get_framebuffer_size();
	let mut renderer = Renderer::new(&context, framebuffer_width, framebuffer_height);

	let mut camera = Camera::new(framebuffer_width as f32 / framebuffer_height as f32, 75.0, 0.1, 10.0);
	camera.position.set(0.0, 0.0, -2.0);

	let geometry = geometry3d::Box::new();
	let mut meshes = [Mesh::new(&geometry, Material::Basic)];

	let ambient_light = AmbientLight::new();

	let mut window_minimized = false;
	let mut framebuffer_resized;
	let mut framebuffer_width = 0;
	let mut framebuffer_height = 0;
	let mut surface_changed = false;

	while !window.should_close() {
		framebuffer_resized = false;
		glfw.poll_events();

		for (_, event) in glfw::flush_messages(&events) {
			match event {
				glfw::WindowEvent::FramebufferSize(width, height) => {
					if width == 0 && height == 0 {
						window_minimized = true;
					}
					else {
						if !window_minimized {
							framebuffer_resized = true;
							framebuffer_width = width;
							framebuffer_height = height;
						}

						window_minimized = false;
					}
				},
				glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
					window.set_should_close(true);
				},
				_ => ()
			}
		}

		if window_minimized {
			glfw.wait_events();
			continue;
		}

		if framebuffer_resized || surface_changed {
			renderer.recreate_swapchain(framebuffer_width, framebuffer_height);
			camera.projection_matrix.make_perspective(framebuffer_width as f32 / framebuffer_height as f32, 75.0, 0.1, 10.0);
		}
		
		meshes[0].rotate_y(0.0001);

		surface_changed = renderer.render(&mut camera, &mut meshes, &ambient_light, &[], &mut []);
	}
}