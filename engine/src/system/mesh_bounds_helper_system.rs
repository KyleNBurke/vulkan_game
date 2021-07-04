use crate::{Geometry3D, component::{ComponentList, Mesh, MeshBoundsHelper, Transform3DComponentList}, math::{Box3, Vector3}, pool::Pool};

pub struct MeshBoundsHelperSystem {
	pub entities: Vec<usize>
}

impl MeshBoundsHelperSystem {
	pub fn new() -> Self {
		Self {
			entities: Vec::new()
		}
	}

	pub fn update(&self, transform_components: &Transform3DComponentList, mesh_components: &ComponentList<Mesh>, geometries: &mut Pool<Geometry3D>, mesh_bounds_helper_components: &ComponentList<MeshBoundsHelper>) {
		for entity in &self.entities {
			let global_matrix = transform_components.borrow(*entity).global_matrix();
			let mesh = mesh_components.borrow(*entity);
			let geometry = geometries.borrow(mesh.geometry_handle);
			let bounding_box_vertices = geometry.bounding_box().as_vertices();

			let mut min = Vector3::from_scalar(f32::INFINITY);
			let mut max = Vector3::from_scalar(f32::NEG_INFINITY);
			
			for vertex in &bounding_box_vertices {
				let transformed_vertex = global_matrix * vertex.expand(1.0);

				min.x = min.x.min(transformed_vertex.x);
				min.y = min.y.min(transformed_vertex.y);
				min.z = min.z.min(transformed_vertex.z);

				max.x = max.x.max(transformed_vertex.x);
				max.y = max.y.max(transformed_vertex.y);
				max.z = max.z.max(transformed_vertex.z);
			}

			let bounds_entity = mesh_bounds_helper_components.borrow(*entity).bounds_entity;
			let bounds_mesh = mesh_components.borrow(bounds_entity);
			let bounds_geo = geometries.borrow_mut(bounds_mesh.geometry_handle);
			bounds_geo.make_box_helper(&Box3::new(min, max));
		}
	}
}