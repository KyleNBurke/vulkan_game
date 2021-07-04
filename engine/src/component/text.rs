use crate::pool::Handle;

pub struct Text {
	pub font: Handle,
	pub string: String,
	pub(crate) indices: Vec<u16>,
	pub(crate) attributes: Vec<f32>
}

impl Text {
	pub fn new(font: Handle, string: String) -> Self {
		Self {
			font,
			string,
			indices: Vec::new(),
			attributes: Vec::new()
		}
	}

	pub fn indices(&self) -> &[u16] {
		&self.indices
	}

	pub fn attributes(&self) -> &[f32] {
		&self.attributes
	}
}