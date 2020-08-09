use engine::{
	vulkan::{Context, Renderer},
	mesh::{self, Mesh},
	geometry,
	Object3D,
	Camera,
	lights::AmbientLight,
	Font,
	Text
};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

fn main() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
	glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
	let (mut window, events) = glfw.create_window(WINDOW_WIDTH, WINDOW_HEIGHT, "Simple", glfw::WindowMode::Windowed).unwrap();
	window.set_key_polling(true);
	window.set_framebuffer_size_polling(true);

	let context = Context::new(&glfw, &window);
	let (width, height) = window.get_framebuffer_size();
	let mut renderer = Renderer::new(&context, width as u32, height as u32);

	let dynamic_box = Mesh::new(Box::new(geometry::Box {}), mesh::Material::Basic);
	let mut dynamic_meshes = vec![dynamic_box];

	let mut camera = Camera::new(WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32, 75.0, 0.1, 10.0);
	camera.translate_z(-2.0);

	let ambient_light = AmbientLight::new();

	let font = Font::new(String::from(""));
	let text = Text::new(&font, 0.0, 0.0, "");

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

		let (width, height) = window.get_framebuffer_size();
		if width == 0 || height == 0 {
			glfw.wait_events();
			continue;
		}

		if framebuffer_resized {
			renderer.recreate_swapchain(width as u32, height as u32);
			camera.projection_matrix.make_perspective(width as f32 / height as f32, 75.0, 0.1, 10.0);
		}
		
		dynamic_meshes[0].rotate_y(0.0001);

		renderer.render(&window, &mut camera, &mut dynamic_meshes, &ambient_light, &[], &text);
	}
}