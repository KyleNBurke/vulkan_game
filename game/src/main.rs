use engine::{
	vulkan::{Context, Renderer},
	Camera,
	lights::AmbientLight,
	Scene,
	StateManager,
	EngineResources,
	math::Vector3
};

mod states;
use states::{StateData, gameplay::GameplayState};

mod camera_controller;
use camera_controller::CameraController;

fn main() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
	glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
	let (mut window, events) = glfw.create_window(1280, 720, "Vulkan", glfw::WindowMode::Windowed).unwrap();
	let (width, height) = window.get_framebuffer_size();
	window.set_framebuffer_size_polling(true);
	window.set_key_polling(true);

	let context = Context::new(&glfw, &window);
	let mut renderer = Renderer::new(&context, width, height);

	let roboto_32 = renderer.add_font("game/res/roboto.ttf", 32);
	let roboto_14 = renderer.add_font("game/res/roboto.ttf", 14);
	
	let state_data = StateData { roboto_32, roboto_14 };

	let camera = Camera::new(width as f32 / height as f32, 75.0, 0.1, 10.0);
	let ambient_light = AmbientLight::from(Vector3::from_scalar(1.0), 0.01);
	let scene = Scene::new(camera, ambient_light);

	let mut resources = EngineResources {
		window,
		renderer,
		game_resources: state_data,
		scene
	};
	
	let gameplay_state = Box::new(GameplayState::new());

	let mut state_manager = StateManager::new(&mut resources, gameplay_state);

	let mut minimized = false;
	let mut resized;
	let mut width = 0;
	let mut height = 0;
	let mut surface_changed = false;

	while !resources.window.should_close() {
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
				glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => resources.window.set_should_close(true),
				_ => ()
			}

			state_manager.handle_event(&event, &mut resources);
		}

		if minimized {
			glfw.wait_events();
			continue;
		}

		if resized || surface_changed {
			resources.renderer.handle_resize(width, height);
			resources.scene.camera.projection_matrix.make_perspective(width as f32 / height as f32, 75.0, 0.1, 10.0);
		}

		state_manager.update(&mut resources);

		surface_changed = resources.renderer.render(&mut resources.scene);
	}
}