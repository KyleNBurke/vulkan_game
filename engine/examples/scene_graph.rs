use utilities::Window;
use engine::{Renderer, Camera, lights::AmbientLight, scene::{Scene, Entity}, math::Vector3, Geometry3D, mesh::{Mesh, Material}};

fn main() {
	let mut window = Window::new("Scene graph");
	let mut renderer = Renderer::new(&window.glfw, &window.glfw_window);

	let (width, height) = window.glfw_window.get_framebuffer_size();
	let mut camera = Camera::new(width as f32 / height as f32, 75.0, 0.1, 10.0);
	camera.transform.position.set(0.0, 0.0, -5.0);
	let ambient_light = AmbientLight::from(Vector3::from_scalar(1.0), 0.01);
	let mut scene = Scene::new(camera, ambient_light);

	let box_geometry_handle = scene.geometries.add(Geometry3D::create_box());
	let box_handle = scene.meshes.add(Mesh::new(box_geometry_handle, Material::Basic));

	let node_a_handle = scene.graph.add_node(Entity::Mesh(box_handle));

	let node_b_handle = scene.graph.add_child_node(node_a_handle, Entity::Mesh(box_handle)).unwrap();
	scene.graph.get_node_mut(&node_b_handle).unwrap().transform.position.set(0.0, 0.0, 2.0);

	let node_c_handle = scene.graph.add_child_node(node_b_handle, Entity::Mesh(box_handle)).unwrap();
	scene.graph.get_node_mut(&node_c_handle).unwrap().transform.position.set(0.0, 2.0, 0.0);

	let mut surface_changed = false;

	window.main_loop(|resized, width, height| {
		if resized || surface_changed {
			renderer.handle_resize(width, height);
			scene.camera.projection_matrix.make_perspective(width as f32 / height as f32, 75.0, 0.1, 10.0);
		}

		let node_a = scene.graph.get_node_mut(&node_a_handle).unwrap();
		node_a.transform.rotate_y(0.01);

		let node_b = scene.graph.get_node_mut(&node_b_handle).unwrap();
		node_b.transform.rotate_z(0.01);

		surface_changed = renderer.render(&mut scene);
	});
}