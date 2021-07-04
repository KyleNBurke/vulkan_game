use std::time::Duration;
use engine::{
	Camera,
	EntityManager,
	Font,
	Geometry3D,
	component::{ComponentList, Light, Mesh, MeshBoundsHelper, Text, TextComponentList, Transform2D, Transform3D, Transform3DComponentList, mesh::Material},
	glfw::{self, Glfw},
	math::{Vector3, box3, vector3},
	pool::Pool,
	system::{MeshBoundsHelperSystem, RenderSystem}
};
use crate::{CameraController, component::RigidBody, system::{FrameMetricsSystem, PhysicsSystem}};

pub struct Game {
	camera: Camera,
	camera_controller: CameraController,
	camera_controller_enabled: bool,
	geometries: Pool<Geometry3D>,
	fonts: Pool<Font>,
	render_system: RenderSystem,
	frame_metrics_system: FrameMetricsSystem,
	physics_system: PhysicsSystem,
	mesh_bounds_helper_system: MeshBoundsHelperSystem,
	text_components: TextComponentList,
	transform2d_components: ComponentList<Transform2D>,
	light_components: ComponentList<Light>,
	mesh_components: ComponentList<Mesh>,
	transform3d_components: Transform3DComponentList,
	rigid_body_components: ComponentList<RigidBody>,
	mesh_bounds_helper_components: ComponentList<MeshBoundsHelper>
}

impl Game {
	pub fn new(glfw: &Glfw, window: &glfw::Window) -> Self {
		let mut render_system = RenderSystem::new(glfw, window);
		let (extent_width, extent_height) = render_system.get_swapchain_extent();
		let mut camera = Camera::new(extent_width as f32 / extent_height as f32, 75.0, 0.1, 50.0);
		camera.transform.position.z = -4.0;
		camera.update();

		let mut geometries = Pool::<Geometry3D>::new();
		let mut fonts = Pool::<Font>::new();
		let mut entity_manager = EntityManager::new();

		let mut text_components = TextComponentList::new();
		let mut transform2d_components = ComponentList::<Transform2D>::new();
		let light_components = ComponentList::<Light>::new();
		let mut mesh_components = ComponentList::<Mesh>::new();
		let mut transform3d_components = Transform3DComponentList::new();
		let mut rigid_body_components = ComponentList::<RigidBody>::new();
		let mut mesh_bounds_helper_components = ComponentList::<MeshBoundsHelper>::new();

		let mut physics_system = PhysicsSystem::new();
		let mut mesh_bounds_helper_system = MeshBoundsHelperSystem::new();

		let label_entitiy = entity_manager.create();
		let font_handle = fonts.add(Font::new("game/res/roboto.ttf", 14));
		render_system.submit_fonts(&mut fonts);
		text_components.add(label_entitiy, Text::new(font_handle, String::from("...")));
		let mut transform = Transform2D::new();
		transform.position.set(10.0, 20.0);
		transform.update_matrix();
		transform2d_components.add(label_entitiy, transform);
		render_system.entities.push(label_entitiy);

		let frame_metrics_system = FrameMetricsSystem::new(label_entitiy);

		let box_1_bounds_helper = entity_manager.create();
		transform3d_components.add(box_1_bounds_helper, Transform3D::new());
		let geometry_handle = geometries.add(Geometry3D::create_box_helper(&box3::DEFAULT_SQUARE));
		mesh_components.add(box_1_bounds_helper, Mesh { geometry_handle, material: Material::Line });
		render_system.entities.push(box_1_bounds_helper);

		let box_1 = entity_manager.create();
		let mut transform = Transform3D::new();
		transform.position.set(0.0, 10.0, 5.0);
		transform.rotate_x(0.5);
		transform.rotate_z(0.3);
		transform.scale.set_from_scalar(0.5);
		transform3d_components.add(box_1, transform);
		let geometry_handle = geometries.add(Geometry3D::create_box());
		mesh_components.add(box_1, Mesh { geometry_handle, material: Material::Normal });
		rigid_body_components.add(box_1, RigidBody { velocity: vector3::ZERO, acceleration: Vector3::new(0.0, -0.00001, 0.0) });
		mesh_bounds_helper_components.add(box_1, MeshBoundsHelper { bounds_entity: box_1_bounds_helper });
		render_system.entities.push(box_1);
		physics_system.entities.push(box_1);
		mesh_bounds_helper_system.entities.push(box_1);


		let plane = entity_manager.create();
		let mut transform = Transform3D::new();
		transform.scale.set_from_scalar(10.0);
		transform3d_components.add(plane, transform);
		let geometry_handle = geometries.add(Geometry3D::create_plane());
		mesh_components.add(plane, Mesh { geometry_handle, material: Material::Normal });
		render_system.entities.push(plane);

		Self {
			camera,
			camera_controller: CameraController::new(window),
			camera_controller_enabled: false,
			geometries,
			fonts,
			render_system,
			frame_metrics_system,
			physics_system,
			mesh_bounds_helper_system,
			text_components,
			transform2d_components,
			light_components,
			mesh_components,
			transform3d_components,
			rigid_body_components,
			mesh_bounds_helper_components
		}
	}

	pub fn handle_event(&mut self, event: &glfw::WindowEvent, window: &mut glfw::Window) {
		match event {
			glfw::WindowEvent::Key(glfw::Key::Tab, _, glfw::Action::Press, _) => {
				self.camera_controller_enabled = !self.camera_controller_enabled;

				if self.camera_controller_enabled {
					self.camera_controller.poll_mouse_pos(window);
					window.set_cursor_mode(glfw::CursorMode::Disabled);
				}
				else {
					window.set_cursor_mode(glfw::CursorMode::Normal);
				}
			},
			_ => ()
		}
	}

	pub fn handle_resize(&mut self, width: i32, height: i32) {
		let (extent_width, extent_height) = self.render_system.resize(width, height);
		self.camera.projection_matrix.make_perspective(extent_width as f32 / extent_height as f32, 75.0, 0.1, 50.0);
	}

	pub fn update(&mut self, window: &glfw::Window, delta_time: &Duration) {
		self.frame_metrics_system.update(&mut self.text_components, delta_time);

		if self.camera_controller_enabled {
			self.camera_controller.update(window, &mut self.camera, delta_time);
		}

		self.physics_system.update(&mut self.transform3d_components, &mut self.rigid_body_components);
		self.mesh_bounds_helper_system.update(&self.transform3d_components, &self.mesh_components, &mut self.geometries, &self.mesh_bounds_helper_components);
		
		self.text_components.generate_dirties(&self.fonts);
		self.transform3d_components.check_for_dirties();
	}

	pub fn render(&mut self) -> bool {
		self.render_system.render(&self.camera, &self.light_components, &self.geometries, &self.mesh_components, &self.transform3d_components, &self.fonts, &self.text_components, &self.transform2d_components)
	}
}