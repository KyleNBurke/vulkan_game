use engine::{
	geometry3d,
	Mesh,
	Material,
	lights::PointLight,
	math::Vector3,
	Handle,
	Text,
};

use crate::{CameraController, State, StateAction, EngineResources};

pub struct GameplayState {
	camera_controller: CameraController,
	camera_controller_enabled: bool,
	box_handle: Handle,
	font_handle: Handle
}

impl GameplayState {
	pub fn new() -> Self {
		Self {
			camera_controller: CameraController::new(),
			camera_controller_enabled: true,
			box_handle: Handle::null(),
			font_handle: Handle::null()
		}
	}
}

impl State for GameplayState {
	fn enter(&mut self, resources: &mut EngineResources) {
		// Static
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

		resources.renderer.submit_static_meshes(&mut vec![static_triangle, static_plane, static_box, point_light_box1, point_light_box2]);

		// Dynamic
		let mut text = Text::new(resources.game_resources.roboto_32, String::from("Hello from a 32 pixel size font"));
		text.transform.position.set(10.0, 30.0);
		resources.scene.text.add(text);

		let mut text = Text::new(resources.game_resources.roboto_14, String::from("Hello from a 14 pixel size font"));
		text.transform.position.set(10.0, 60.0);
		resources.scene.text.add(text);

		let triangle_geo = Box::new(geometry3d::Triangle::new());
		let mut dynamic_triangle = Mesh::new(triangle_geo, Material::Lambert);
		dynamic_triangle.transform.position.set(-0.5, -0.6, 2.0);
		resources.scene.meshes.add(dynamic_triangle);

		let plane_geo = Box::new(geometry3d::Plane::new());
		let mut dynamic_plane = Mesh::new(plane_geo, Material::Basic);
		dynamic_plane.transform.position.set(0.5, -0.6, 2.0);
		resources.scene.meshes.add(dynamic_plane);

		let box_geo = Box::new(geometry3d::Box::new());
		let mut dynamic_box = Mesh::new(box_geo, Material::Lambert);
		dynamic_box.transform.position.set(2.0, 0.0, 0.0);
		self.box_handle = resources.scene.meshes.add(dynamic_box);

		let mut point_light1 = PointLight::from(Vector3::from_scalar(1.0), 0.3);
		point_light1.position.set(0.0, -1.0, 0.0);
		resources.scene.point_lights.add(point_light1);

		let mut point_light2 = PointLight::from(Vector3::from_scalar(1.0), 0.3);
		point_light2.position.set(-1.0, -1.0, 0.0);
		resources.scene.point_lights.add(point_light2);
		resources.scene.camera.transform.position.set(0.0, 0.0, -2.0);

		resources.window.set_cursor_mode(glfw::CursorMode::Disabled);
		self.camera_controller.poll_mouse_pos(&resources.window);
	}

	fn leave(&mut self, _resources: &mut EngineResources) {}

	fn handle_event(&mut self, event: &glfw::WindowEvent, resources: &mut EngineResources) {
		match event {
			glfw::WindowEvent::Key(glfw::Key::Tab, _, glfw::Action::Press, _) => {
				self.camera_controller_enabled = !self.camera_controller_enabled;

				if self.camera_controller_enabled {
					self.camera_controller.poll_mouse_pos(&resources.window);
					resources.window.set_cursor_mode(glfw::CursorMode::Disabled);
				}
				else {
					resources.window.set_cursor_mode(glfw::CursorMode::Normal);
				}
			},
			glfw::WindowEvent::Key(glfw::Key::G, _, glfw::Action::Press, _) => {
				self.font_handle = resources.renderer.add_font("game/res/roboto.ttf", 64);
				let mut text = Text::new(self.font_handle, String::from("Hello from a 64 pixel size font"));
				text.transform.position.set(10.0, 150.0);
				resources.scene.text.add(text);
			},
			glfw::WindowEvent::Key(glfw::Key::H, _, glfw::Action::Press, _) => {
				resources.renderer.remove_font(&self.font_handle);
			},
			_ => ()
		}
	}

	fn update(&mut self, resources: &mut EngineResources) -> StateAction {
		if self.camera_controller_enabled {
			self.camera_controller.update(&resources.window, &mut resources.scene.camera);
		}

		resources.scene.meshes.get_mut(&self.box_handle).unwrap().transform.rotate_y(0.0001);

		StateAction::None
	}
}