pub mod triangle;
pub use triangle::Triangle;

pub mod plane;
pub use plane::Plane;

pub trait Geometry {
	fn get_vertex_indices(&self) -> &[u16];
	fn get_vertex_attributes(&self) -> &[f32];
}