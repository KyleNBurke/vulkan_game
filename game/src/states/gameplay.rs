use engine::{
	Scene, 
	state::{State, StateAction},
	geometry3d,
	Mesh,
	mesh::Material,
	lights::PointLight,
	math::Vector3
};

use crate::StateData;

pub struct GameplayState;

impl GameplayState {
	pub fn create_static_meshes(&self) -> Vec<Mesh> {
		let triangle_geo = Box::new(geometry3d::Triangle::new());
		let mut static_triangle = Mesh::new(triangle_geo, Material::Basic);
		static_triangle.position.set(0.0, 1.0, 1.7);

		let plane_geo = Box::new(geometry3d::Plane::new());
		let mut static_plane = Mesh::new(plane_geo, Material::Basic);
		static_plane.position.set(0.0, 1.0, 2.0);

		let box_geo = Box::new(geometry3d::Box::new());
		let mut static_box = Mesh::new(box_geo, Material::Lambert);
		static_box.position.set(-2.0, 0.0, 0.0);

		let box_geo = Box::new(geometry3d::Box::new());
		let mut point_light_box1 = Mesh::new(box_geo, Material::Basic);
		point_light_box1.position.set(0.0, -1.0, 0.0);
		point_light_box1.scale.set_from_scalar(0.2);

		let box_geo = Box::new(geometry3d::Box::new());
		let mut point_light_box2 = Mesh::new(box_geo, Material::Basic);
		point_light_box2.position.set(-1.0, -1.0, 0.0);
		point_light_box2.scale.set_from_scalar(0.2);

		vec![static_triangle, static_plane, static_box, point_light_box1, point_light_box2]
	}
}

impl State<StateData> for GameplayState {
	fn enter(&self, scene: &mut Scene) {
		scene.camera.position.set(0.0, 0.0, -2.0);

		let triangle_geo = Box::new(geometry3d::Triangle::new());
		let mut dynamic_triangle = Mesh::new(triangle_geo, Material::Lambert);
		dynamic_triangle.position.set(-0.5, -0.6, 2.0);
		scene.meshes.push(dynamic_triangle);

		let plane_geo = Box::new(geometry3d::Plane::new());
		let mut dynamic_plane = Mesh::new(plane_geo, Material::Lambert);
		dynamic_plane.position.set(0.5, -0.6, 2.0);
		scene.meshes.push(dynamic_plane);

		let box_geo = Box::new(geometry3d::Box::new());
		let mut dynamic_box = Mesh::new(box_geo, Material::Lambert);
		dynamic_box.position.set(2.0, 0.0, 0.0);
		scene.meshes.push(dynamic_box);

		let mut point_light1 = PointLight::from(Vector3::from_scalar(1.0), 0.3);
		point_light1.position.set(0.0, -1.0, 0.0);
		scene.point_lights.push(point_light1);

		let mut point_light2 = PointLight::from(Vector3::from_scalar(1.0), 0.3);
		point_light2.position.set(-1.0, -1.0, 0.0);
		scene.point_lights.push(point_light2);
	}

	fn leave(&self, _: &mut Scene) {}

	fn update(&self, _scene: &mut Scene, _data: &mut StateData) -> StateAction<StateData> {
		StateAction::None
	}
}