use std::{time::Duration, vec, f32::consts::PI, convert::TryInto};

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
	camera_controller_enabled: bool,
	box_handle: Handle
}

impl GameplayState {
	pub fn new() -> Self {
		Self {
			camera_controller: CameraController::new(),
			camera_controller_enabled: false,
			box_handle: Handle::null()
		}
	}

	fn load(&mut self, scene: &mut Scene) {
		let (document, buffers, _) = gltf::import("game/res/monkey.gltf").unwrap();
		let mut geometry_map: Vec<Handle> = vec![];

		for mesh in document.meshes() {
			let name = if let Some(name) = mesh.name() { name } else { "unamed" };
			let primitive = mesh.primitives().last().unwrap();
			assert_eq!(primitive.mode(), gltf::mesh::Mode::Triangles);

			let indices_accessor = primitive.indices()
				.unwrap_or_else(|| panic!("Cannot load mesh {}, no indices found", name));
			assert_eq!(indices_accessor.data_type(), gltf::accessor::DataType::U16);
			assert_eq!(indices_accessor.dimensions(), gltf::accessor::Dimensions::Scalar);

			let indices_view = indices_accessor.view().unwrap();
			let buffer = &buffers[indices_view.buffer().index()];
			let stride = if let Some(stride) = indices_view.stride() { stride } else { 2 };
			let mut indices: Vec<u16> = Vec::with_capacity(indices_accessor.count());

			for i in 0..indices_accessor.count() {
				let start = indices_accessor.offset() + indices_view.offset() + i * stride;
				let end = start + 2;
				let bytes = buffer[start..end].try_into().unwrap();
				let index = u16::from_le_bytes(bytes);
				indices.push(index);
			}

			let positions_accessor = primitive.get(&gltf::Semantic::Positions)
				.unwrap_or_else(|| panic!("Cannot load mesh {}, no positions attribute found", name));
			assert_eq!(positions_accessor.data_type(), gltf::accessor::DataType::F32);
			assert_eq!(positions_accessor.dimensions(), gltf::accessor::Dimensions::Vec3);

			let normals_accessor = primitive.get(&gltf::Semantic::Normals)
				.unwrap_or_else(|| panic!("Cannot load mesh {}, no normals attribute found", name));
			assert_eq!(normals_accessor.data_type(), gltf::accessor::DataType::F32);
			assert_eq!(normals_accessor.dimensions(), gltf::accessor::Dimensions::Vec3);

			assert_eq!(positions_accessor.count(), normals_accessor.count());

			let positions_view = positions_accessor.view().unwrap();
			let positions_buffer = &buffers[positions_view.buffer().index()];
			let positions_stride = if let Some(stride) = positions_view.stride() { stride } else { 12 };

			let normals_view = normals_accessor.view().unwrap();
			let normals_buffer = &buffers[normals_view.buffer().index()];
			let normals_stride = if let Some(stride) = normals_view.stride() { stride } else { 12 };
			
			let mut attributes: Vec<f32> = Vec::with_capacity(positions_accessor.count() * 3 * 2);

			for i in 0..positions_accessor.count() {
				let start = positions_accessor.offset() + positions_view.offset() + i * positions_stride;
				let end = start + 12;
				let bytes = positions_buffer[start..end].as_ptr() as *const [f32; 3];
				let position = unsafe { *bytes };
				attributes.extend_from_slice(&position);

				let start = normals_accessor.offset() + normals_view.offset() + i * normals_stride;
				let end = start + 12;
				let bytes = normals_buffer[start..end].as_ptr() as *const [f32; 3];
				let normal = unsafe { *bytes };
				attributes.extend_from_slice(&normal);
			}

			let geometry = Geometry3D::new(indices, attributes, Topology::Triangle);
			let handle = scene.geometries.add(geometry);
			geometry_map.push(handle);
		}

		for gltf_scene in document.scenes() {
			let mut nodes: Vec<(gltf::Node, Matrix4)> = vec![];

			for node in gltf_scene.nodes() {
				let mut matrix = Matrix4::new(node.transform().matrix());
				matrix.transpose();
				
				if let Some(gltf_mesh) = node.mesh() {
					let geometry_handle = geometry_map[gltf_mesh.index()];
					let mesh = Mesh::new(geometry_handle, Material::Lambert);
					let mut node = Node::new(Object::Mesh(mesh));
					node.transform.matrix = matrix;
					
					scene.graph.add(node);
				}

				nodes.push((node, matrix));
			}

			while let Some((parent_node, parent_matrix)) = nodes.pop() {
				for child_node in parent_node.children() {
					let mut child_matrix = Matrix4::new(child_node.transform().matrix());
					child_matrix.transpose();
					child_matrix = parent_matrix * child_matrix;

					if let Some(gltf_mesh) = child_node.mesh() {
						let geometry_handle = geometry_map[gltf_mesh.index()];
						let mesh = Mesh::new(geometry_handle, Material::Lambert);
						let mut node = Node::new(Object::Mesh(mesh));
						node.transform.matrix = child_matrix;
						
						scene.graph.add(node);
					}

					nodes.push((child_node, child_matrix));
				}
			}
		}
	}
}

impl State for GameplayState {
	fn enter(&mut self, resources: &mut EngineResources) {
		let scene = &mut resources.scene;

		// Static
		let mut geometries = Pool::<Geometry3D>::new();
		let plane_geo = geometries.add(Geometry3D::create_plane());
		let box_geo = geometries.add(Geometry3D::create_box());

		let mut static_plane = StaticMesh::new(plane_geo, Material::Lambert);
		static_plane.transform.position.set(0.0, -1.0, 2.0);
		static_plane.transform.update_matrix();

		let mut static_box = StaticMesh::new(box_geo, Material::Normal);
		static_box.transform.position.set(2.0, 0.0, 0.0);
		static_box.transform.update_matrix();

		let mut point_light_box1 = StaticMesh::new(box_geo, Material::Basic);
		point_light_box1.transform.position.set(0.0, 2.0, 0.0);
		point_light_box1.transform.scale.set_from_scalar(0.2);
		point_light_box1.transform.update_matrix();

		let mut point_light_box2 = StaticMesh::new(box_geo, Material::Basic);
		point_light_box2.transform.position.set(1.0, 1.0, 3.0);
		point_light_box2.transform.scale.set_from_scalar(0.2);
		point_light_box2.transform.update_matrix();

		let axis_helper_geometry = geometries.add(Geometry3D::create_axis_helper());
		let mut axis_helper = StaticMesh::new(axis_helper_geometry, Material::Line);
		axis_helper.transform.position.x = -4.0;
		axis_helper.transform.update_matrix();

		resources.renderer.submit_static_meshes(&geometries, &vec![static_plane, static_box, point_light_box1, point_light_box2, axis_helper]);

		// Dynamic
		let box_geo = scene.geometries.add(Geometry3D::create_box());

		let box_lambert_mesh = Mesh::new(box_geo, Material::Lambert);
		let mut box_lambert_node = Node::new(Object::Mesh(box_lambert_mesh));
		box_lambert_node.transform.position.set(-2.0, 0.0, 0.0);
		box_lambert_node.transform.update_matrix();
		self.box_handle = scene.graph.add(box_lambert_node);

		let point_light_1 = PointLight::new();
		let mut point_light_1_node = Node::new(Object::PointLight(point_light_1));
		point_light_1_node.transform.position.set(0.0, 2.0, 0.0);
		point_light_1_node.transform.update_matrix();
		let point_light_1_handle = scene.graph.add(point_light_1_node);
		scene.graph.update_at(point_light_1_handle);

		let point_light_2 = PointLight::new();
		let mut point_light_2_node = Node::new(Object::PointLight(point_light_2));
		point_light_2_node.transform.position.set(1.0, 1.0, 3.0);
		point_light_2_node.transform.update_matrix();
		let point_light_2_handle = scene.graph.add(point_light_2_node);
		scene.graph.update_at(point_light_2_handle);

		let box_2 = Mesh::new(box_geo, Material::Lambert);
		let mut box_2_node = Node::new(Object::Mesh(box_2));
		box_2_node.transform.translate_z(2.5);
		scene.graph.add_to(self.box_handle, box_2_node);

		let axis_helper_geometry = scene.geometries.add(Geometry3D::create_axis_helper());
		let axis_helper = Mesh::new(axis_helper_geometry, Material::Line);
		let mut axis_helper_node = Node::new(Object::Mesh(axis_helper));
		axis_helper_node.transform.translate_x(4.0);
		let axis_helper_handle = scene.graph.add(axis_helper_node);
		scene.graph.update_at(axis_helper_handle);

		let box3 = Box3::new(Vector3::from_scalar(-0.5), Vector3::from_scalar(0.5));
		let box_helper_geometry = scene.geometries.add(Geometry3D::create_box_helper(&box3));
		let box_helper = Mesh::new(box_helper_geometry, Material::Line);
		let mut box_helper = Node::new(Object::Mesh(box_helper));
		box_helper.transform.translate_x(6.0);
		let box_helper_handle = scene.graph.add(box_helper);
		scene.graph.update_at(box_helper_handle);

		// Load gltf
		self.load(&mut resources.scene);
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

		let mesh_node = resources.scene.graph.borrow_mut(self.box_handle);
		mesh_node.transform.rotate_y(frame_time.as_secs_f32());
		resources.scene.graph.update_at(self.box_handle);

		StateAction::None
	}
}