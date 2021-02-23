use std::{time::Duration, vec};

use engine::{
	pool::{Pool, Handle},
	Geometry3D,
	Transform3D,
	mesh::{Material, StaticMesh, StaticInstancedMesh, Mesh},
	lights::PointLight,
	Text,
	vulkan::Font
};

use crate::{CameraController, State, StateAction, EngineResources};

pub struct GameplayState {
	camera_controller: CameraController,
	camera_controller_enabled: bool,
	box_handle: Handle<Mesh>,
	font_handle: Handle<Font>
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
		let scene = &mut resources.scene;

		// Static
		let mut geometries = Pool::<Geometry3D>::new();
		let triangle_geo = geometries.add(Geometry3D::create_triangle());
		let plane_geo = geometries.add(Geometry3D::create_plane());
		let box_geo = geometries.add(Geometry3D::create_box());

		let mut static_triangle = StaticMesh::new(triangle_geo, Material::Basic);
		static_triangle.transform.position.set(0.0, 1.0, 1.7);

		let mut static_plane = StaticMesh::new(plane_geo, Material::Basic);
		static_plane.transform.position.set(0.0, 1.0, 2.0);

		let mut static_box = StaticMesh::new(box_geo, Material::Lambert);
		static_box.transform.position.set(-2.0, 0.0, 0.0);

		let mut point_light_box1 = StaticMesh::new(box_geo, Material::Basic);
		point_light_box1.transform.position.set(0.0, -1.0, 0.0);
		point_light_box1.transform.scale.set_from_scalar(0.2);

		let mut point_light_box2 = StaticMesh::new(box_geo, Material::Basic);
		point_light_box2.transform.position.set(-1.0, -1.0, 0.0);
		point_light_box2.transform.scale.set_from_scalar(0.2);

		let mut instanced_box = StaticInstancedMesh::new(box_geo, Material::Basic);
		let mut transform = Transform3D::new();
		transform.position.set(0.0, 2.0, 0.0);
		instanced_box.transforms.push(transform);

		resources.renderer.submit_static_meshes(&geometries, &mut vec![static_triangle, static_plane, static_box, point_light_box1, point_light_box2], &mut vec![instanced_box]);

		// Dynamic
		let triangle_geo = scene.geometries.add(Geometry3D::create_triangle());
		let box_geo = scene.geometries.add(Geometry3D::create_box());

		let mut triangle_lambert_mesh = Mesh::new(triangle_geo, Material::Basic);
		triangle_lambert_mesh.transform.position.set(-0.5, -0.6, 2.0);
		triangle_lambert_mesh.transform.update_matrix();
		scene.meshes.add(triangle_lambert_mesh);

		let mut box_lambert_mesh = Mesh::new(box_geo, Material::Lambert);
		box_lambert_mesh.transform.position.set(2.0, 0.0, 0.0);
		box_lambert_mesh.transform.update_matrix();
		self.box_handle = scene.meshes.add(box_lambert_mesh);

		let mut point_light_1 = PointLight::new();
		point_light_1.position.set(0.0, -1.0, 0.0);
		resources.scene.point_lights.add(point_light_1);

		let mut point_light_2 = PointLight::new();
		point_light_2.position.set(-1.0, -1.0, 0.0);
		resources.scene.point_lights.add(point_light_2);

		resources.window.set_cursor_mode(glfw::CursorMode::Disabled);
		self.camera_controller.poll_mouse_pos(&resources.window);
	}

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

	fn update(&mut self, resources: &mut EngineResources, frame_time: &Duration) -> StateAction {
		if self.camera_controller_enabled {
			self.camera_controller.update(&resources.window, &mut resources.scene.camera, frame_time);
		}

		let mesh = resources.scene.meshes.get_mut(&self.box_handle).unwrap();
		mesh.transform.rotate_y(frame_time.as_secs_f32());
		mesh.transform.update_matrix();

		StateAction::None
	}
}