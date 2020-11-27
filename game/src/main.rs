use engine::{
	vulkan::{Context, Renderer},
	Camera,
	lights::AmbientLight,
	Font,
	Scene,
	state::StateManager,
	math::Vector3
};

mod states;
use states::gameplay::GameplayState;

pub struct StateData;

fn main() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
	glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
	let (mut window, events) = glfw.create_window(1280, 720, "Vulkan", glfw::WindowMode::Windowed).unwrap();
	let (width, height) = window.get_framebuffer_size();
	window.set_framebuffer_size_polling(true);
	window.set_key_polling(true);

	let context = Context::new(&glfw, &window);
	let mut renderer = Renderer::new(&context, width, height);
	
	let font = Font::new("game/res/roboto.ttf", 32);
	renderer.submit_font(&font);

	let mut scene = Scene {
		camera: Camera::new(width as f32 / height as f32, 75.0, 0.1, 10.0),
		ambient_light: AmbientLight::from(Vector3::from_scalar(1.0), 0.01),
		point_lights: vec![],
		meshes: vec![],
		ui_elements: vec![]
	};

	let gameplay_state = Box::new(GameplayState);
	let mut static_meshes = gameplay_state.create_static_meshes();
	renderer.submit_static_meshes(&mut static_meshes);

	let mut state_manager = StateManager::new(&mut scene, gameplay_state);
	let mut state_data = StateData;

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
				glfw::WindowEvent::Key(glfw::Key::R, _, glfw::Action::Press, _) => {
					renderer.submit_static_meshes(&mut static_meshes);
					renderer.submit_font(&font);
					println!("Static meshes and font submitted");
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
			scene.camera.projection_matrix.make_perspective(width as f32 / height as f32, 75.0, 0.1, 10.0);
		}

		state_manager.update(&mut scene, &mut state_data);

		surface_changed = renderer.render(&mut scene.camera, &mut scene.meshes, &scene.ambient_light, &scene.point_lights, &mut scene.ui_elements);
	}
}