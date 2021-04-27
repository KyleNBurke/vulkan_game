use utilities::Window;

use engine::{
	Renderer,
	Camera,
	lights::{AmbientLight, PointLight},
	Scene,
	pool::Pool,
	math::Vector3,
	Geometry3D,
	mesh::{Material, Mesh}
};

fn main() {
	let mut window = Window::new("Instancing");
	let mut renderer = Renderer::new(&window.glfw, &window.glfw_window);

	let mut geometries = Pool::<Geometry3D>::new();
	let g = geometries.add(Geometry3D::create_box());
	let mut static_meshes = vec![];

	for i in 0..10 {
		for j in 0..10 {
			let mut mesh = Mesh::new(g, Material::Lambert);
			mesh.transform.position.set(i as f32 * 1.5, -1.5, j as f32 * 1.5);
			mesh.transform.update_matrix();
			static_meshes.push(mesh);
		}
	}

	renderer.submit_static_meshes(&geometries, &static_meshes);

	let extent = renderer.get_swapchain_extent();
	let mut camera = Camera::new(extent.width as f32 / extent.height as f32, 75.0, 0.1, 50.0);
	camera.transform.position.set(-2.0, -3.0, -2.0);
	camera.transform.rotate_y(3.14 / 4.0);
	camera.transform.rotate_x(-3.14 / 6.0);
	camera.transform.update_matrix();

	let ambient_light = AmbientLight::from(Vector3::from_scalar(1.0), 0.01);
	let mut scene = Scene::new(camera, ambient_light);

	let box_geo = scene.geometries.add(Geometry3D::create_box());

	for i in 0..10 {
		for j in 0..10 {
			let mut mesh = Mesh::new(box_geo, Material::Normal);
			mesh.transform.position.set(i as f32 * 1.5, 0.0, j as f32 * 1.5);
			mesh.transform.update_matrix();
			scene.meshes.add(mesh);

			let mut mesh = Mesh::new(box_geo, Material::Basic);
			mesh.transform.position.set(i as f32 * 1.5, 1.5, j as f32 * 1.5);
			mesh.transform.update_matrix();
			scene.meshes.add(mesh);
		}
	}

	let mut point_light = PointLight::new();
	point_light.position.set(-5.0, -8.0, 0.0);
	scene.point_lights.add(point_light);

	let mut surface_changed = false;

	window.main_loop(|resized, width, height| {
		if resized || surface_changed {
			renderer.resize(width, height);
			let extent = renderer.get_swapchain_extent();
			scene.camera.projection_matrix.make_perspective(extent.width as f32 / extent.height as f32, 75.0, 0.1, 50.0);
		}

		surface_changed = renderer.render(&mut scene);
	});
}