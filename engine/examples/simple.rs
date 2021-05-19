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
	let camera = Camera::new(extent.width as f32 / extent.height as f32, 75.0, 0.1, 50.0);
	let mut camera_node = Node::new(Object::Camera(camera));
	camera_node.transform.update_matrix();

	let mut scene = Scene::new();
	scene.camera_handle = scene.graph.add(camera_node);

	let geometry_handle = scene.geometries.add(Geometry3D::create_box());
	let mesh = Mesh::new(geometry_handle, Material::Normal);
	let mut mesh_node = Node::new(Object::Mesh(mesh));
	mesh_node.transform.translate_z(2.0);
	let mesh_handle = scene.graph.add(mesh_node);

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
			let camera_node = scene.graph.borrow_mut(scene.camera_handle);
			let camera_object = &mut camera_node.object;
			let camera = camera_object.as_camera_mut();
			camera.projection_matrix.make_perspective(extent.width as f32 / extent.height as f32, 75.0, 0.1, 50.0);
		}
		
		let mesh_node = scene.graph.borrow_mut(mesh_handle);
		mesh_node.transform.rotate_y(0.005);
		
		scene.graph.update();
		surface_changed = renderer.render(&mut scene);
	}
}