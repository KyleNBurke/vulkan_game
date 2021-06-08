use std::{time::Duration, vec};

use engine::{geometry3d::{Geometry3D}, glfw, graph::{Node, Object}, math::{Box3, Vector3, box3, vector3}, mesh::{Mesh, Material}, pool::Handle};

use crate::{CameraController, State, StateAction, EngineResources};

struct Cube {
	handle: Handle,
	bounds_handle: Handle,
	velocity: Vector3,
	acceleration: Vector3
}

pub struct GameplayState {
	camera_controller: CameraController,
	camera_controller_enabled: bool,
	cubes: Vec<Cube>,
	plane_bounds: Box3
}

impl GameplayState {
	pub fn new() -> Self {
		Self {
			camera_controller: CameraController::new(),
			camera_controller_enabled: false,
			cubes: vec![],
			plane_bounds: Box3::new(Vector3::new(-5.0, 0.0, -5.0),  Vector3::new(5.0, 0.0, 5.0))
		}
	}
}

impl State for GameplayState {
	fn enter(&mut self, resources: &mut EngineResources) {
		let scene = &mut resources.scene;

		let plane_geo = scene.geometries.add(Geometry3D::create_plane());
		let plane_mesh = Mesh::new(plane_geo, Material::Basic);
		let mut plane_node = Node::new(Object::Mesh(plane_mesh));
		plane_node.transform.scale.set_from_scalar(10.0);
		scene.graph.add(plane_node);

		let box_geo = scene.geometries.add(Geometry3D::create_box());
		let box_mesh = Mesh::new(box_geo, Material::Normal);
		let mut box_node = Node::new(Object::Mesh(box_mesh));
		box_node.transform.position.set(0.0, 10.0, 5.0);
		box_node.transform.rotate_x(0.5);
		box_node.transform.rotate_z(0.3);
		box_node.transform.scale.set_from_scalar(0.5);
		let handle_1 = scene.graph.add(box_node);

		let bounds_geo_handle = scene.geometries.add(Geometry3D::create_box_helper(&box3::DEFAULT_SQUARE));
		let bounds_mesh = Mesh::new(bounds_geo_handle, Material::Line);
		let bounds_node = Node::new(Object::Mesh(bounds_mesh));
		let bounds_handle_1 = scene.graph.add(bounds_node);

		let box_geo = scene.geometries.add(Geometry3D::create_box());
		let box_mesh = Mesh::new(box_geo, Material::Normal);
		let mut box_node = Node::new(Object::Mesh(box_mesh));
		box_node.transform.position.set(4.0, 3.0, 5.5);
		let handle_2 = scene.graph.add(box_node);

		let bounds_geo_handle = scene.geometries.add(Geometry3D::create_box_helper(&box3::DEFAULT_SQUARE));
		let bounds_mesh = Mesh::new(bounds_geo_handle, Material::Line);
		let bounds_node = Node::new(Object::Mesh(bounds_mesh));
		let bounds_handle_2 = scene.graph.add(bounds_node);

		self.cubes = vec![
			Cube {
				handle: handle_1,
				bounds_handle: bounds_handle_1,
				velocity: vector3::ZERO,
				acceleration: Vector3::new(0.0, -0.00001, 0.0),
			},
			Cube {
				handle: handle_2,
				bounds_handle: bounds_handle_2,
				velocity: vector3::ZERO,
				acceleration: Vector3::new(0.0, -0.00001, 0.0),
			}
		];
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

		for cube in &mut self.cubes {
			let cube_transform = resources.scene.graph.borrow_transform_mut(cube.handle);
			cube.velocity += cube.acceleration;
			cube_transform.position += cube.velocity;
			cube_transform.rotate_y(0.005);
		}

		resources.scene.graph.update();

		for cube in &self.cubes {
			let cube_transform = resources.scene.graph.borrow_transform(cube.handle);
			let bounds_transform = cube_transform.global_matrix();
			let cube_mesh = resources.scene.graph.borrow_object(cube.handle).as_mesh();
			let cube_geometry = resources.scene.geometries.borrow(cube_mesh.geometry_handle);
			let cube_bounding_box_vertices = cube_geometry.bounding_box().as_vertices();

			let mut min = Vector3::from_scalar(f32::INFINITY);
			let mut max = Vector3::from_scalar(f32::NEG_INFINITY);
			
			for vertex in &cube_bounding_box_vertices {
				let transformed_vertex = bounds_transform * vertex.expand(1.0);

				min.x = min.x.min(transformed_vertex.x);
				min.y = min.y.min(transformed_vertex.y);
				min.z = min.z.min(transformed_vertex.z);

				max.x = max.x.max(transformed_vertex.x);
				max.y = max.y.max(transformed_vertex.y);
				max.z = max.z.max(transformed_vertex.z);
			}

			let bounds_mesh = resources.scene.graph.borrow_object(cube.bounds_handle).as_mesh();
			let bounds_geo = resources.scene.geometries.borrow_mut(bounds_mesh.geometry_handle);
			bounds_geo.make_box_helper(&Box3::new(min, max));

			let a = Box3::new(min, max);
			let b = &self.plane_bounds;
			let colliding = a.min.x <= b.max.x && a.max.x >= b.min.x &&
				a.min.y <= b.max.y && a.max.y >= b.min.y &&
				a.min.z <= b.max.z && a.max.z >= b.min.z;
			
			if colliding { println!("colliding") };
		}

		StateAction::None
	}
}