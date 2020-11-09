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
	let (width, height) = window.get_framebuffer_size();
	let mut renderer = Renderer::new(&context, width, height);

	let mut camera = Camera::new(width as f32 / height as f32, 75.0, 0.1, 10.0);
	camera.position.set(0.0, 0.0, -2.0);

	let geometry = geometry3d::Box::new();
	let mesh = Mesh::new(&geometry, Material::Basic);
	let mut meshes = [mesh];

	let ambient_light = AmbientLight::new();

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
				glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
					window.set_should_close(true);
				},
				_ => ()
			}
		}

		if minimized {
			glfw.wait_events();
			continue;
		}

		if resized || surface_changed {
			renderer.recreate_swapchain(width, height);
			camera.projection_matrix.make_perspective(width as f32 / height as f32, 75.0, 0.1, 10.0);
		}
		
		meshes[0].rotate_y(0.0001);

		surface_changed = renderer.render(&mut camera, &mut meshes, &ambient_light, &[], &mut []);
	}
}