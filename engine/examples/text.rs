use utilities::Window;

use engine::{
	vulkan::{Context, Renderer},
	mesh::{Mesh, Material},
	geometry3d,
	geometry2d,
	Object3D,
	Camera,
	lights::AmbientLight,
	Font,
	UIElement
};

use std::boxed::Box;

fn main() {
	let mut window = Window::new("Text");

	let context = Context::new(&window.glfw, &window.glfw_window);
	let (width, height) = window.glfw_window.get_framebuffer_size();
	let mut renderer = Renderer::new(&context, width, height);

	let mut camera = Camera::new(width as f32 / height as f32, 75.0, 0.1, 10.0);
	camera.position.set(0.0, 0.0, -2.0);

	let geometry = Box::new(geometry3d::Box::new());
	let mesh = Mesh::new(geometry, Material::Basic);
	let mut meshes = [mesh];

	let ambient_light = AmbientLight::new();

	let (_, scale_y) = window.glfw_window.get_content_scale();
	let font = Font::new("engine/examples/res/roboto.ttf", (32.0 * scale_y) as u32);
	renderer.submit_font(&font);

	let text = geometry2d::Text::new(&font, "Text rendering example!");
	let mut ui_element = UIElement::new(&text);
	ui_element.position.set(50.0, 50.0);
	let mut ui_elements = [ui_element];

	let mut surface_changed = false;

	window.main_loop(|resized, width, height| {
		if resized || surface_changed {
			renderer.recreate_swapchain(width, height);
			camera.projection_matrix.make_perspective(width as f32 / height as f32, 75.0, 0.1, 10.0);
		}

		meshes[0].rotate_y(0.0001);

		surface_changed = renderer.render(&mut camera, &mut meshes, &ambient_light, &[], &mut ui_elements);
	});
}