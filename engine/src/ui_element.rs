use crate::{Transform2D, geometry2d::Geometry2D};
use std::boxed::Box;

pub struct UIElement {
	pub transform: Transform2D,
	pub auto_update_matrix: bool,
	pub geometry: Box<dyn Geometry2D>,
	pub(crate) index_offset: usize,
	pub(crate) attribute_offset: usize,
	pub(crate) uniform_offset: usize
}

impl UIElement {
	pub fn new(geometry: Box<dyn Geometry2D>) -> Self {
		Self {
			transform: Transform2D::new(),
			auto_update_matrix: true,
			geometry,
			index_offset: 0,
			attribute_offset: 0,
			uniform_offset: 0
		}
	}
}