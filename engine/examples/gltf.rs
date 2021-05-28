use std::convert::TryInto;
use utilities::Window;

use engine::{
	Renderer,
	Camera,
	Scene,
	graph::{Node, Object},
	pool::Handle,
	geometry3d::{Geometry3D, Topology},
	mesh::{Material, Mesh},
	math::Matrix4
};

fn main() {
	let mut window = Window::new("Instancing");
	let mut renderer = Renderer::new(&window.glfw, &window.glfw_window);

	let mut scene = Scene::new();

	let (extent_width, extent_height) = renderer.get_swapchain_extent();
	let camera = Camera::new(extent_width as f32 / extent_height as f32, 75.0, 0.1, 50.0);
	let mut camera_node = Node::new(Object::Camera(camera));
	camera_node.transform.position.z = -3.0;
	scene.camera_handle = scene.graph.add(camera_node);
	scene.graph.update_at(scene.camera_handle);

	load(&mut scene);

	let mut surface_changed = false;

	window.main_loop(|resized, width, height| {
		if resized || surface_changed {
			let (extent_width, extent_height) = renderer.resize(width, height);
			let camera_node = scene.graph.borrow_mut(scene.camera_handle);
			let camera = camera_node.object.as_camera_mut();
			camera.projection_matrix.make_perspective(extent_width as f32 / extent_height as f32, 75.0, 0.1, 50.0);
		}

		surface_changed = renderer.render(&mut scene);
	});
}

fn load(scene: &mut Scene) {
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
				let mesh = Mesh::new(geometry_handle, Material::Normal);
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
					let mesh = Mesh::new(geometry_handle, Material::Normal);
					let mut node = Node::new(Object::Mesh(mesh));
					node.transform.matrix = child_matrix;
					
					scene.graph.add(node);
				}

				nodes.push((child_node, child_matrix));
			}
		}
	}
}