use utilities::Window;

use engine::{
	Renderer,
	Camera,
	Scene,
	graph::{Node, Object},
	Geometry3D,
	mesh::{Material, Mesh}
};

fn main() {
	let mut window = Window::new("Instancing");
	let mut renderer = Renderer::new(&window.glfw, &window.glfw_window);

	let mut scene = Scene::new();

	let (extent_width, extent_height) = renderer.get_swapchain_extent();
	let camera = Camera::new(extent_width as f32 / extent_height as f32, 75.0, 0.1, 50.0);
	let mut camera_node = Node::new(Object::Camera(camera));
	camera_node.transform.position.set(-5.0, 3.0, -5.0);
	camera_node.transform.rotate_y(3.14 / 4.0);
	camera_node.transform.rotate_x(3.14 / 6.0);
	scene.camera_handle = scene.graph.add(camera_node);

	let empty_node = Node::new(Object::Empty);
	let empty_handle = scene.graph.add(empty_node);
	let box_geo = scene.geometries.add(Geometry3D::create_box());

	for i in -2..3 {
		for j in -2..3 {
			let mesh = Mesh::new(box_geo, Material::Normal);
			let mut node = Node::new(Object::Mesh(mesh));
			node.transform.position.set(i as f32 * 1.5, 0.0, j as f32 * 1.5);
			scene.graph.add_to(empty_handle, node);
		}
	}

	let mesh = Mesh::new(box_geo, Material::Normal);
	let mut node = Node::new(Object::Mesh(mesh));
	node.transform.position.y = 2.0;
	let a_handle = scene.graph.add_to(empty_handle, node);

	let mesh = Mesh::new(box_geo, Material::Normal);
	let mut node = Node::new(Object::Mesh(mesh));
	node.transform.position.y = 1.5;
	let b_handle = scene.graph.add_to(a_handle, node);

	let mesh = Mesh::new(box_geo, Material::Normal);
	let mut node = Node::new(Object::Mesh(mesh));
	node.transform.position.z = 1.5;
	let c_handle = scene.graph.add_to(b_handle, node);

	let mut scale = 0.0f32;
	let mut surface_changed = false;

	window.main_loop(|resized, width, height| {
		if resized || surface_changed {
			let (extent_width, extent_height) = renderer.resize(width, height);
			let camera_node = scene.graph.borrow_mut(scene.camera_handle);
			let camera = camera_node.object.as_camera_mut();
			camera.projection_matrix.make_perspective(extent_width as f32 / extent_height as f32, 75.0, 0.1, 50.0);
		}

		let empty_node = scene.graph.borrow_mut(empty_handle);
		empty_node.transform.rotate_y(0.005);

		let a_node = scene.graph.borrow_mut(a_handle);
		a_node.transform.rotate_x(0.005);

		let b_node = scene.graph.borrow_mut(b_handle);
		b_node.transform.rotate_y(0.005);
		scale += 0.005;
		b_node.transform.scale.set_from_scalar(scale.sin() * 0.5 + 1.0);

		let c_node = scene.graph.borrow_mut(c_handle);
		c_node.transform.rotate_z(0.005);

		scene.graph.update();
		surface_changed = renderer.render(&mut scene);
	});
}