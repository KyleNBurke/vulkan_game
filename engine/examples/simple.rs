use engine::{
	Renderer,
	Scene,
	graph::{Node, Object},
	Camera,
	Geometry3D,
	mesh::{Mesh, Material}
};

fn main() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
	glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
	let (mut window, events) = glfw.create_window(1280, 720, "Simple", glfw::WindowMode::Windowed).unwrap();
	window.set_framebuffer_size_polling(true);
	window.set_key_polling(true);

	let mut renderer = Renderer::new(&glfw, &window);

	let extent = renderer.get_swapchain_extent();
	let mut camera = Camera::new(extent.width as f32 / extent.height as f32, 75.0, 0.1, 50.0);
	camera.transform.position.z = -2.0;
	camera.transform.update_matrix();

	let mut scene = Scene::new();
	scene.camera_handle = scene.graph.add(Node::new(Object::Camera(camera)));

	let geometry_handle = scene.geometries.add(Geometry3D::create_box());
	let mesh = Mesh::new(geometry_handle, Material::Normal);
	let mesh_handle = scene.graph.add(Node::new(Object::Mesh(mesh)));

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
			renderer.resize(width, height);
			let extent = renderer.get_swapchain_extent();
			let camera_node = scene.graph.get_mut(&scene.camera_handle).unwrap();
			let camera_object = &mut camera_node.object;
			let camera = camera_object.camera_mut().unwrap();
			camera.projection_matrix.make_perspective(extent.width as f32 / extent.height as f32, 75.0, 0.1, 50.0);
		}
		
		let mesh_node = scene.graph.get_mut(&mesh_handle).unwrap();
		mesh_node.transform.rotate_y(0.005);
		mesh_node.transform.update_matrix();

		surface_changed = renderer.render(&mut scene);
	}
}