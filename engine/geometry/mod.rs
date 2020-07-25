pub mod triangle;
pub use triangle::Triangle;

pub mod plane;
pub use plane::Plane;

pub mod r#box;
pub use r#box::Box;

pub mod text;
pub use text::Text;

pub trait Geometry {
	fn get_vertex_indices(&self) -> &[u16];
	fn get_vertex_attributes(&self) -> &[f32];
}