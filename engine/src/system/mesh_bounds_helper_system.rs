use crate::{Geometry3D, component::{ComponentList, MultiComponentList, Mesh, MeshBoundsHelper, Transform3DComponentList}, math::{Box3, Vector3}, pool::Pool};

pub struct MeshBoundsHelperSystem {
	pub entities: Vec<usize>
}

impl MeshBoundsHelperSystem {
	pub fn new() -> Self {
		Self {
			entities: Vec::new()
		}
	}

	pub fn update(&self, transform_components: &mut Transform3DComponentList, mesh_components: &MultiComponentList<Mesh>, geometries: &mut Pool<Geometry3D>, mesh_bounds_helper_components: &ComponentList<MeshBoundsHelper>) {
		for entity in &self.entities {
			let transform = transform_components.borrow(*entity);
			let transform_position = transform.position;
			let global_matrix = transform.global_matrix().truncate();
			let mesh = mesh_components.borrow(*entity);
			let geometry = geometries.borrow(mesh.geometry_handle);
			let bounding_box_vertices = geometry.bounding_box().as_vertices();

			let mut min = Vector3::from_scalar(f32::INFINITY);
			let mut max = Vector3::from_scalar(f32::NEG_INFINITY);
			
			for vertex in &bounding_box_vertices {
				let transformed_vertex = global_matrix * vertex;

				min.min(&transformed_vertex);
				max.max(&transformed_vertex);
			}

			let bounds_entity = mesh_bounds_helper_components.borrow(*entity).bounds_entity;
			
			let bounds_mesh = mesh_components.borrow(bounds_entity);
			let bounds_geo = geometries.borrow_mut(bounds_mesh.geometry_handle);
			bounds_geo.make_box_helper(&Box3::new(min, max));

			let bounds_transform = transform_components.borrow_mut(bounds_entity);
			bounds_transform.position = transform_position;
			transform_components.update(bounds_entity);
		}
	}
}