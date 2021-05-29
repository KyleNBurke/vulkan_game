use std::{time::Duration, vec, convert::TryInto};

use engine::{
	glfw,
	pool::{Pool, Handle},
	geometry3d::{Geometry3D, Topology},
	mesh::{Mesh, StaticMesh, Material},
	lights::PointLight,
	Scene,
	graph::{Node, Object},
	math::{Matrix4, Box3, Vector3}
};

use crate::{CameraController, State, StateAction, EngineResources};



pub struct GameplayState {
	camera_controller: CameraController,
	camera_controller_enabled: bool
}

impl GameplayState {
	pub fn new() -> Self {
		Self {
			camera_controller: CameraController::new(),
			camera_controller_enabled: false
		}
	}
}

impl State for GameplayState {
	fn enter(&mut self, resources: &mut EngineResources) {
		let scene = &mut resources.scene;

		// Static
		let mut geometries = Pool::<Geometry3D>::new();

		let box_geo = geometries.add(Geometry3D::create_box());
		let mut static_box = StaticMesh::new(box_geo, Material::Normal);
		static_box.transform.position.set(0.0, 3.0, 5.0);
		static_box.transform.update_matrix();

		resources.renderer.submit_static_meshes(&geometries, &vec![static_box]);

		// Dynamic
		let plane_geo = scene.geometries.add(Geometry3D::create_plane());
		let plane_mesh = Mesh::new(plane_geo, Material::Basic);
		let mut plane_node = Node::new(Object::Mesh(plane_mesh));
		plane_node.transform.scale.set_from_scalar(10.0);
		let plane_handle = scene.graph.add(plane_node);
		scene.graph.update_at(plane_handle);
	}

	fn handle_event(&mut self, event: &glfw::WindowEvent, resources: &mut EngineResources) {
		if let glfw::WindowEvent::Key(glfw::Key::Tab, _, glfw::Action::Press, _) = event {
			self.camera_controller_enabled = !self.camera_controller_enabled;

			if self.camera_controller_enabled {
				self.camera_controller.poll_mouse_pos(&resources.window);
				resources.window.set_cursor_mode(glfw::CursorMode::Disabled);
			}
			else {
				resources.window.set_cursor_mode(glfw::CursorMode::Normal);
			}
		}
	}

	fn update(&mut self, resources: &mut EngineResources, frame_time: &Duration) -> StateAction {
		if self.camera_controller_enabled {
			self.camera_controller.update(resources, frame_time);
		}

		StateAction::None
	}
}