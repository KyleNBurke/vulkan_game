use engine::{Camera, EntityManager, Geometry3D, component::{ComponentList, Light, Mesh, Renderable, Transform3D, mesh::Material}, pool::Pool, system::{RenderSystem, System}};

fn main() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
	glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
	let (mut window, events) = glfw.create_window(1280, 720, "Simple", glfw::WindowMode::Windowed).unwrap();
	window.set_framebuffer_size_polling(true);
	window.set_key_polling(true);

	let mut entity_manager = EntityManager::new();

	// create all our component lists
	let mut renderable_components = ComponentList::<Renderable>::new();
	let mut transform_components = ComponentList::<Transform3D>::new();
	let mut light_components = ComponentList::<Light>::new();
	let mut mesh_components = ComponentList::<Mesh>::new();

	// create all our systems
	let mut render_system = RenderSystem::new(&glfw, &window);

	let mut geometries = Pool::<Geometry3D>::new();
	let geo_handle = geometries.add(Geometry3D::create_box());

	let entity = entity_manager.create_entity();
	renderable_components.add_defer(entity, Renderable);
	transform_components.add_defer(entity, Transform3D::new());
	mesh_components.add_defer(entity, Mesh { geometry_handle: geo_handle, material: Material::Normal });

	let (extent_width, extent_height) = render_system.get_swapchain_extent();
	let mut camera = Camera::new(extent_width as f32 / extent_height as f32, 75.0, 0.1, 50.0);

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
			let (extent_width, extent_height) = render_system.resize(width, height);
			camera.projection_matrix.make_perspective(extent_width as f32 / extent_height as f32, 75.0, 0.1, 50.0);
		}
		
		renderable_components.maintain(&mut entity_manager);
		light_components.maintain(&mut entity_manager);
		mesh_components.maintain(&mut entity_manager);
		transform_components.maintain(&mut entity_manager);

		let mut systems: [Box<&mut dyn System>; 1] = [
			Box::new(&mut render_system)
		];

		entity_manager.maintain(&mut systems);
		
		surface_changed = render_system.render(&camera, &light_components, &geometries, &mesh_components, &transform_components);
	}
}