use engine::{
	Scene, 
	SceneGraph,
	SceneObject,
	state::{State, StateAction},
	geometry3d,
	Mesh,
	mesh::Material,
	lights::PointLight,
	math::Vector3,
	Object3D
};

use crate::{StateData, CameraController};

pub struct GameplayState {
	camera_controller: CameraController,
	camera_controller_enabled: bool
}

impl GameplayState {
	pub fn new() -> Self {
		Self {
			camera_controller: CameraController::new(),
			camera_controller_enabled: true
		}
	}

	pub fn create_static_meshes(&self) -> Vec<Mesh> {
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

		vec![static_triangle, static_plane, static_box, point_light_box1, point_light_box2]
	}
}

impl State<StateData> for GameplayState {
	fn enter(&mut self, window: &mut glfw::Window, scene_graph: &mut SceneGraph) {
		scene_graph.camera.position.set(0.0, 0.0, -2.0);

		window.set_cursor_mode(glfw::CursorMode::Disabled);
		self.camera_controller.poll_mouse_pos(window);

		let box_geo = Box::new(geometry3d::Box::new());
		let mut dynamic_box = Mesh::new(box_geo, Material::Lambert);
		dynamic_box.position.set(0.0, 0.0, 1.0);
		let handle = scene_graph.add(SceneObject::Mesh(dynamic_box));

		if let SceneObject::Mesh(mesh) = &mut scene_graph.get_node_mut(&handle).unwrap().object {
			mesh.rotate_y(std::f32::consts::FRAC_PI_4);
		}
	}

	fn leave(&mut self, _window: &mut glfw::Window, scene_graph: &mut SceneGraph) {}

	fn handle_event(&mut self, event: &glfw::WindowEvent, window: &mut glfw::Window, scene_graph: &mut SceneGraph) {
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

	fn update(&mut self, window: &mut glfw::Window, scene_graph: &mut SceneGraph, _data: &mut StateData) -> StateAction<StateData> {
		if self.camera_controller_enabled {
			self.camera_controller.update(window, &mut scene_graph.camera);
		}

		StateAction::None
	}
}