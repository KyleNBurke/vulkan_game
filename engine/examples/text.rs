use utilities::Window;

use engine::{
	Renderer,
	Camera,
	lights::AmbientLight,
	scene::Scene,
	math::Vector3,
	Geometry3D,
	mesh::{Material, Mesh},
	Font,
	Text
};

fn main() {
	let mut window = Window::new("Text");
	let mut renderer = Renderer::new(&window.glfw, &window.glfw_window);

	let extent = renderer.get_swapchain_extent();
	let mut camera = Camera::new(extent.width as f32 / extent.height as f32, 75.0, 0.1, 50.0);
	camera.transform.position.set(0.0, 0.0, -2.0);
	camera.transform.update_matrix();

	let ambient_light = AmbientLight::from(Vector3::from_scalar(1.0), 0.01);
	let mut scene = Scene::new(camera, ambient_light);

	let box_geo = scene.geometries.add(Geometry3D::create_box());
	let mesh = Mesh::new(box_geo, Material::Normal);
	let mesh_handle = scene.meshes.add(mesh);

	let font = Font::new("game/res/roboto.ttf", 32);
	let font_handle = scene.fonts.add(font);
	renderer.submit_fonts(&mut scene.fonts);

	let mut text = Text::new(font_handle, String::from("This is some text!"));
	text.transform.position.set(50.0, 80.0);
	text.transform.update_matrix();
	scene.text.add(text);

	let mut surface_changed = false;

	window.main_loop(|resized, width, height| {
		if resized || surface_changed {
			renderer.resize(width, height);
			let extent = renderer.get_swapchain_extent();
			scene.camera.projection_matrix.make_perspective(extent.width as f32 / extent.height as f32, 75.0, 0.1, 50.0);
		}

		let mesh = scene.meshes.get_mut(&mesh_handle).unwrap();
		mesh.transform.rotate_y(0.005);
		mesh.transform.update_matrix();

		surface_changed = renderer.render(&mut scene);
	});
}