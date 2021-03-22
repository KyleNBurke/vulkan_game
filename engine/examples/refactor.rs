use utilities::Window;

use engine::{
	Renderer,
	Camera,
	lights::AmbientLight,
	scene::Scene,
	pool::Pool,
	math::Vector3,
	Geometry3D,
	mesh::{Material, Mesh}
};

fn main() {
	let mut window = Window::new("Instancing");
	let mut renderer = Renderer::new(&window.glfw, &window.glfw_window);

	let (width, height) = window.glfw_window.get_framebuffer_size();

	let mut camera = Camera::new(width as f32 / height as f32, 75.0, 0.1, 50.0);
	camera.transform.position.set(0.0, -2.0, 0.0);
	camera.transform.rotate_y(3.14 / 4.0);
	camera.transform.rotate_x(-3.14 / 6.0);

	let ambient_light = AmbientLight::from(Vector3::from_scalar(1.0), 0.01);
	let mut scene = Scene::new(camera, ambient_light);

	let box_geo = scene.geometries.add(Geometry3D::create_box());

	for i in 0..10 {
		for j in 0..10 {
			let mut mesh = Mesh::new(box_geo, Material::Basic);
			mesh.transform.position.set(i as f32 * 1.5, 0.0, j as f32 * 1.5);
			mesh.transform.update_matrix();
			scene.meshes.add(mesh);

			let mut mesh = Mesh::new(box_geo, Material::Normal);
			mesh.transform.position.set(i as f32 * 1.5, 1.5, j as f32 * 1.5);
			mesh.transform.update_matrix();
			scene.meshes.add(mesh);
		}
	}

	let mut geometries = Pool::<Geometry3D>::new();
	let g = geometries.add(Geometry3D::create_box());
	let mut mesh = Mesh::new(g, Material::Basic);
	mesh.transform.position.set(8.0, -2.0, 8.0);
	mesh.transform.update_matrix();

	let mut mesh2 = Mesh::new(g, Material::Lambert);
	mesh2.transform.position.set(8.0, -1.0, 8.0);
	mesh2.transform.update_matrix();

	renderer.submit_static_meshes(&geometries, &mut [mesh, mesh2]);

	let mut surface_changed = false;

	window.main_loop(|resized, width, height| {
		if resized || surface_changed {
			renderer.handle_resize(width, height);
			scene.camera.projection_matrix.make_perspective(width as f32 / height as f32, 75.0, 0.1, 50.0);
		}

		surface_changed = renderer.render(&mut scene);
	});
}