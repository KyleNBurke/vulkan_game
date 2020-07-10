use engine::{
	vulkan::Context,
	Renderer,
	mesh::{self, Mesh},
	geometry,
	math,
	Object3D,
	Camera,
	lights
};

fn main() {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
	glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
	let (mut window, events) = glfw.create_window(1280, 720, "Vulkan", glfw::WindowMode::Windowed).unwrap();
	window.set_key_polling(true);
	window.set_framebuffer_size_polling(true);

	let context = Context::new(&glfw, &window);
	let (width, height) = window.get_framebuffer_size();
	let mut renderer = Renderer::new(&context, width as u32, height as u32);

	let mut static_triangle = Mesh::new(Box::new(geometry::Triangle {}), mesh::Material::Basic);
	static_triangle.position = math::Vector3::from(0.0, 1.0, 1.7);

	let mut static_plane = Mesh::new(Box::new(geometry::Plane {}), mesh::Material::Basic);
	static_plane.position = math::Vector3::from(0.0, 1.0, 2.0);

	let mut static_box = Mesh::new(Box::new(geometry::Box {}), mesh::Material::Lambert);
	static_box.position = math::Vector3::from(-2.0, 0.0, 0.0);

	let mut point_light_box1 = Mesh::new(Box::new(geometry::Box {}), mesh::Material::Basic);
	point_light_box1.translate_y(-1.0);
	*point_light_box1.get_scale_mut() = math::Vector3::from_scalar(0.2);

	let mut point_light_box2 = Mesh::new(Box::new(geometry::Box {}), mesh::Material::Basic);
	point_light_box2.translate_x(-1.0);
	point_light_box2.translate_y(-1.0);
	*point_light_box2.get_scale_mut() = math::Vector3::from_scalar(0.2);

	let mut static_meshes = vec![static_triangle, static_plane, static_box, point_light_box1, point_light_box2];
	renderer.submit_static_meshes(&mut static_meshes);

	let mut dynamic_triangle = Mesh::new(Box::new(geometry::Triangle {}), mesh::Material::Lambert);
	dynamic_triangle.position = math::Vector3::from(-0.5, -0.6, 2.0);

	let mut dynamic_plane = Mesh::new(Box::new(geometry::Plane {}), mesh::Material::Lambert);
	dynamic_plane.position = math::Vector3::from(0.5, -0.6, 2.0);

	let mut dynamic_box = Mesh::new(Box::new(geometry::Box {}), mesh::Material::Lambert);
	dynamic_box.position = math::Vector3::from(2.0, 0.0, 0.0);

	let mut dynamic_meshes = vec![dynamic_triangle, dynamic_plane, dynamic_box];

	let (mouse_pos_x, mouse_pos_y) = window.get_cursor_pos();
	let mut camera = Camera::new(1280.0 / 720.0, 75.0, 0.1, 10.0, mouse_pos_x as f32, mouse_pos_y as f32);

	let ambient_light = lights::AmbientLight::from(math::Vector3::from_scalar(1.0), 0.01);

	let mut point_light1 = lights::PointLight::from(math::Vector3::from_scalar(1.0), 0.3);
	let mut point_light2 = lights::PointLight::from(math::Vector3::from_scalar(1.0), 0.3);
	point_light1.translate_y(-1.0);
	point_light2.translate_x(-1.0);
	point_light2.translate_y(-1.0);
	let point_lights = [point_light1, point_light2];

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
				glfw::WindowEvent::Key(glfw::Key::R, _, glfw::Action::Press, _) => {
					renderer.submit_static_meshes(&mut static_meshes);
					println!("Static meshes submitted");
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
		
		dynamic_meshes[0].rotate_y(0.0005);
		dynamic_meshes[1].rotate_y(0.0005);
		dynamic_meshes[2].rotate_y(-0.0001);

		camera.update(&window);

		renderer.render(&window, &mut camera, &mut dynamic_meshes, &ambient_light, &point_lights);
	}
}