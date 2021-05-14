use std::time::{Instant, Duration};
use engine::{glfw, Renderer, Camera, lights::AmbientLight, scene::{Scene, Node, Object}, math::Vector3, Font, Text};

mod state_manager;
use state_manager::{GameResources, EngineResources, StateManager, State, StateAction};

mod states;
use states::{FrameMetricsState, GameplayState};

mod camera_controller;
use camera_controller::CameraController;

const MAX_FRAME_TIME: f64 = 1.0 / 10.0;
const MAX_UPDATES_PER_FRAME: u32 = 5;

fn main() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
	glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
	let (mut window, events) = glfw.create_window(1280, 720, "Vulkan", glfw::WindowMode::Windowed).unwrap();
	window.set_framebuffer_size_polling(true);
	window.set_key_polling(true);

	let mut renderer = Renderer::new(&glfw, &window);

	let mut scene = Scene::new();

	let extent = renderer.get_swapchain_extent();
	let camera = Camera::new(extent.width as f32 / extent.height as f32, 75.0, 0.1, 50.0);
	scene.camera_handle = scene.nodes.add(Node::new(Object::Camera(camera)));

	let font = Font::new("game/res/roboto.ttf", 14);
	let roboto_14 = scene.fonts.add(font);
	renderer.submit_fonts(&mut scene.fonts);

	let mut resources = EngineResources {
		window,
		renderer,
		game_resources: GameResources {
			roboto_14
		},
		scene
	};
	
	let mut state_manager = StateManager::new();
	let frame_metrics_state = Box::new(FrameMetricsState::new(&mut resources));
	let gameplay_state = Box::new(GameplayState::new());
	state_manager.push(&mut resources, frame_metrics_state);
	state_manager.push(&mut resources, gameplay_state);

	let duration_zero = Duration::new(0, 0);
	let max_duration = Duration::from_secs_f64(MAX_FRAME_TIME);
	let mut frame_start = Instant::now();

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
			resources.renderer.resize(width, height);
			let extent = resources.renderer.get_swapchain_extent();
			let camera_node = resources.scene.nodes.get_mut(&resources.scene.camera_handle).unwrap();
			let camera_object = &mut camera_node.object;
			let camera = camera_object.camera_mut().unwrap();
			camera.projection_matrix.make_perspective(extent.width as f32 / extent.height as f32, 75.0, 0.1, 50.0);
		}

		let frame_end = Instant::now();
		let mut duration = frame_end.duration_since(frame_start);
		frame_start = frame_end;
		let mut updates = 0;

		while duration > duration_zero && updates <= MAX_UPDATES_PER_FRAME {
			let duration_capped = duration.min(max_duration);
			
			state_manager.update(&mut resources, &duration_capped);
			
			duration -= duration_capped;
			updates += 1;
		}

		surface_changed = resources.renderer.render(&mut resources.scene);
	}
}