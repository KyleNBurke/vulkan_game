use engine::{
	state::{State, StateAction},
	Scene,
	geometry3d,
	Mesh,
	mesh::Material,
	lights::PointLight,
	math::Vector3,
	pool::Handle,
	geometry2d::Text,
	UIElement
};

use crate::{StateData, CameraController};

pub struct GameplayState {
	camera_controller: CameraController,
	camera_controller_enabled: bool,
	box_handle: Handle
}

impl GameplayState {
	pub fn new() -> Self {
		Self {
			camera_controller: CameraController::new(),
			camera_controller_enabled: true,
			box_handle: Handle::null()
		}
	}

	pub fn create_static_meshes(&self) -> Vec<Mesh> {
		let triangle_geo = Box::new(geometry3d::Triangle::new());
		let mut static_triangle = Mesh::new(triangle_geo, Material::Basic);
		static_triangle.transform.position.set(0.0, 1.0, 1.7);

		let plane_geo = Box::new(geometry3d::Plane::new());
		let mut static_plane = Mesh::new(plane_geo, Material::Basic);
		static_plane.transform.position.set(0.0, 1.0, 2.0);

		let box_geo = Box::new(geometry3d::Box::new());
		let mut static_box = Mesh::new(box_geo, Material::Lambert);
		static_box.transform.position.set(-2.0, 0.0, 0.0);

		let box_geo = Box::new(geometry3d::Box::new());
		let mut point_light_box1 = Mesh::new(box_geo, Material::Basic);
		point_light_box1.transform.position.set(0.0, -1.0, 0.0);
		point_light_box1.transform.scale.set_from_scalar(0.2);

		let box_geo = Box::new(geometry3d::Box::new());
		let mut point_light_box2 = Mesh::new(box_geo, Material::Basic);
		point_light_box2.transform.position.set(-1.0, -1.0, 0.0);
		point_light_box2.transform.scale.set_from_scalar(0.2);

		vec![static_triangle, static_plane, static_box, point_light_box1, point_light_box2]
	}
}

impl State<StateData> for GameplayState {
	fn enter(&mut self, window: &mut glfw::Window, data: &mut StateData, scene: &mut Scene) {
		let text_geo = Box::new(Text::new(&data.font, "Hello"));
		let mut text_ui_element = UIElement::new(text_geo);
		text_ui_element.transform.position.set(10.0, 40.0);
		scene.ui_elements.add(text_ui_element);

		let triangle_geo = Box::new(geometry3d::Triangle::new());
		let mut dynamic_triangle = Mesh::new(triangle_geo, Material::Lambert);
		dynamic_triangle.transform.position.set(-0.5, -0.6, 2.0);
		scene.meshes.add(dynamic_triangle);

		let plane_geo = Box::new(geometry3d::Plane::new());
		let mut dynamic_plane = Mesh::new(plane_geo, Material::Lambert);
		dynamic_plane.transform.position.set(0.5, -0.6, 2.0);
		scene.meshes.add(dynamic_plane);

		let box_geo = Box::new(geometry3d::Box::new());
		let mut dynamic_box = Mesh::new(box_geo, Material::Lambert);
		dynamic_box.transform.position.set(2.0, 0.0, 0.0);
		self.box_handle = scene.meshes.add(dynamic_box);

		let mut point_light1 = PointLight::from(Vector3::from_scalar(1.0), 0.3);
		point_light1.position.set(0.0, -1.0, 0.0);
		scene.point_lights.add(point_light1);

		let mut point_light2 = PointLight::from(Vector3::from_scalar(1.0), 0.3);
		point_light2.position.set(-1.0, -1.0, 0.0);
		scene.point_lights.add(point_light2);
		scene.camera.transform.position.set(0.0, 0.0, -2.0);

		window.set_cursor_mode(glfw::CursorMode::Disabled);
		self.camera_controller.poll_mouse_pos(window);
	}

	fn leave(&mut self, _window: &mut glfw::Window, _data: &mut StateData, _scene: &mut Scene) {}

	fn handle_event(&mut self, event: &glfw::WindowEvent, window: &mut glfw::Window, _scene: &mut Scene) {
		match event {
			glfw::WindowEvent::Key(glfw::Key::Tab, _, glfw::Action::Press, _) => {
				self.camera_controller_enabled = !self.camera_controller_enabled;

				if self.camera_controller_enabled {
					self.camera_controller.poll_mouse_pos(&window);
					window.set_cursor_mode(glfw::CursorMode::Disabled);
				}
				else {
					window.set_cursor_mode(glfw::CursorMode::Normal);
				}
			},
			_ => ()
		}
	}

	fn update(&mut self, window: &mut glfw::Window, _data: &mut StateData, scene: &mut Scene) -> StateAction<StateData> {
		if self.camera_controller_enabled {
			self.camera_controller.update(window, &mut scene.camera);
		}

		scene.meshes.get_mut(&self.box_handle).unwrap().transform.rotate_y(0.0001);

		StateAction::None
	}
}