use utilities::Window;

use engine::{
	Renderer,
	Camera,
	lights::PointLight,
	Scene,
	graph::{Node, Object},
	pool::Pool,
	Geometry3D,
	mesh::{Material, Mesh, StaticMesh}
};

fn main() {
	let mut window = Window::new("Instancing");
	let mut renderer = Renderer::new(&window.glfw, &window.glfw_window);

	let mut geometries = Pool::<Geometry3D>::new();
	let g = geometries.add(Geometry3D::create_box());
	let mut static_meshes = vec![];

	for i in 0..10 {
		for j in 0..10 {
			let mut mesh = StaticMesh::new(g, Material::Lambert);
			mesh.transform.position.set(i as f32 * 1.5, -1.5, j as f32 * 1.5);
			mesh.transform.update_matrix();
			static_meshes.push(mesh);
		}
	}

	renderer.submit_static_meshes(&geometries, &static_meshes);

	let mut scene = Scene::new();

	let extent = renderer.get_swapchain_extent();
	let camera = Camera::new(extent.width as f32 / extent.height as f32, 75.0, 0.1, 50.0);
	let mut camera_node = Node::new(Object::Camera(camera));
	camera_node.transform.position.set(-2.0, 3.0, -2.0);
	camera_node.transform.rotate_y(3.14 / 4.0);
	camera_node.transform.rotate_x(3.14 / 6.0);
	scene.camera_handle = scene.graph.add(camera_node);

	let box_geo = scene.geometries.add(Geometry3D::create_box());

	for i in 0..10 {
		for j in 0..10 {
			let mesh = Mesh::new(box_geo, Material::Normal);
			let mut node = Node::new(Object::Mesh(mesh));
			node.transform.position.set(i as f32 * 1.5, 0.0, j as f32 * 1.5);
			scene.graph.add(node);

			let mesh = Mesh::new(box_geo, Material::Basic);
			let mut node = Node::new(Object::Mesh(mesh));
			node.transform.position.set(i as f32 * 1.5, 1.5, j as f32 * 1.5);
			scene.graph.add(node);
		}
	}

	let point_light = PointLight::new();
	let mut point_light_node = Node::new(Object::PointLight(point_light));
	point_light_node.transform.position.set(-5.0, 8.0, 0.0);
	scene.graph.add(point_light_node);

	scene.graph.update();

	let mut surface_changed = false;

	window.main_loop(|resized, width, height| {
		if resized || surface_changed {
			renderer.resize(width, height);
			let extent = renderer.get_swapchain_extent();
			let camera_node = scene.graph.borrow_mut(scene.camera_handle);
			let camera_object = &mut camera_node.object;
			let camera = camera_object.as_camera_mut();
			camera.projection_matrix.make_perspective(extent.width as f32 / extent.height as f32, 75.0, 0.1, 50.0);
		}

		surface_changed = renderer.render(&mut scene);
	});
}