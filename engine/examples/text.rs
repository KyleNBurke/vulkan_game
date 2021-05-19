use utilities::Window;

use engine::{
	Renderer,
	Camera,
	Scene,
	graph::{Node, Object},
	Geometry3D,
	mesh::{Material, Mesh},
	Font,
	Text
};

fn main() {
	let mut window = Window::new("Text");
	let mut renderer = Renderer::new(&window.glfw, &window.glfw_window);

	let mut scene = Scene::new();

	let extent = renderer.get_swapchain_extent();
	let camera = Camera::new(extent.width as f32 / extent.height as f32, 75.0, 0.1, 50.0);
	let camera_node = Node::new(Object::Camera(camera));
	scene.camera_handle = scene.graph.add(camera_node);

	let box_geo = scene.geometries.add(Geometry3D::create_box());
	let mesh = Mesh::new(box_geo, Material::Normal);
	let mut mesh_node = Node::new(Object::Mesh(mesh));
	mesh_node.transform.translate_z(2.0);
	let mesh_handle = scene.graph.add(mesh_node);

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
			let camera_node = scene.graph.borrow_mut(scene.camera_handle);
			let camera = camera_node.object.as_camera_mut();
			camera.projection_matrix.make_perspective(extent.width as f32 / extent.height as f32, 75.0, 0.1, 50.0);
		}

		let node = scene.graph.borrow_mut(mesh_handle);
		node.transform.rotate_y(0.005);
		
		scene.graph.update();
		surface_changed = renderer.render(&mut scene);
	});
}