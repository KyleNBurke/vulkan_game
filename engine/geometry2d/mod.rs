pub trait Geometry2D {
	fn get_vertex_indices(&self) -> &[u16];
	fn get_vertex_attributes(&self) -> &[f32];
}

pub mod text;
pub use text::Text;