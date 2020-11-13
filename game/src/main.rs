use engine::{
	vulkan::{Context, Renderer},
	Mesh,
	mesh::Material,
	geometry3d,
	geometry2d,
	math::{Vector3},
	Object3D,
	Camera,
	lights::{AmbientLight, PointLight},
	Font,
	UIElement
};

use std::boxed::Box;

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

	let mut camera = Camera::new(width as f32 / height as f32, 75.0, 0.1, 10.0);
	let mut camera_controller = CameraController::new();
	let mut camera_controller_enabled = false;

	let triangle_geo = Box::new(geometry3d::Triangle::new());
	let mut static_triangle = Mesh::new(triangle_geo, Material::Basic);
	static_triangle.position.set(0.0, 1.0, 1.7);

	let plane_geo = Box::new(geometry3d::Plane::new());
	let mut static_plane = Mesh::new(plane_geo, Material::Basic);
	static_plane.position.set(0.0, 1.0, 2.0);

	let box_geo = Box::new(geometry3d::Box::new());
	let mut static_box = Mesh::new(box_geo, Material::Lambert);
	static_box.position.set(-2.0, 0.0, 0.0);

	let box_geo = Box::new(geometry3d::Box::new());
	let mut point_light_box1 = Mesh::new(box_geo, Material::Basic);
	point_light_box1.position.set(0.0, -1.0, 0.0);
	point_light_box1.scale.set_from_scalar(0.2);

	let box_geo = Box::new(geometry3d::Box::new());
	let mut point_light_box2 = Mesh::new(box_geo, Material::Basic);
	point_light_box2.position.set(-1.0, -1.0, 0.0);
	point_light_box2.scale.set_from_scalar(0.2);

	let mut static_meshes = vec![static_triangle, static_plane, static_box, point_light_box1, point_light_box2];
	renderer.submit_static_meshes(&mut static_meshes);

	let triangle_geo = Box::new(geometry3d::Triangle::new());
	let mut dynamic_triangle = Mesh::new(triangle_geo, Material::Lambert);
	dynamic_triangle.position.set(-0.5, -0.6, 2.0);

	let plane_geo = Box::new(geometry3d::Plane::new());
	let mut dynamic_plane = Mesh::new(plane_geo, Material::Lambert);
	dynamic_plane.position.set(0.5, -0.6, 2.0);

	let box_geo = Box::new(geometry3d::Box::new());
	let mut dynamic_box = Mesh::new(box_geo, Material::Lambert);
	dynamic_box.position.set(2.0, 0.0, 0.0);

	let mut dynamic_meshes = [dynamic_triangle, dynamic_plane, dynamic_box];

	let ambient_light = AmbientLight::from(Vector3::from_scalar(1.0), 0.01);

	let mut point_light1 = PointLight::from(Vector3::from_scalar(1.0), 0.3);
	point_light1.position.set(0.0, -1.0, 0.0);

	let mut point_light2 = PointLight::from(Vector3::from_scalar(1.0), 0.3);
	point_light2.position.set(-1.0, -1.0, 0.0);

	let point_lights = [point_light1, point_light2];

	let font = Font::new("game/res/roboto.ttf", 32);
	renderer.submit_font(&font);

	let text_geo = Box::new(geometry2d::Text::new(&font, "Text rendering example"));
	let mut text = UIElement::new(text_geo);
	text.position.set(10.0, 40.0);
	let mut ui_elements = [text];

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
				glfw::WindowEvent::Key(glfw::Key::Tab, _, glfw::Action::Press, _) => {
					camera_controller_enabled = !camera_controller_enabled;

					if camera_controller_enabled {
						camera_controller.poll_mouse_pos(&window);
						window.set_cursor_mode(glfw::CursorMode::Disabled);
					}
					else {
						window.set_cursor_mode(glfw::CursorMode::Normal);
					}
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
		
		if camera_controller_enabled {
			camera_controller.update(&window, &mut camera);
		}
		
		dynamic_meshes[0].rotate_y(0.0005);
		dynamic_meshes[1].rotate_y(0.0005);
		dynamic_meshes[2].rotate_y(-0.0001);

		surface_changed = renderer.render(&mut camera, &mut dynamic_meshes, &ambient_light, &point_lights, &mut ui_elements);
	}
}