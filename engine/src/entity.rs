use std::fmt;

#[derive(Clone, Copy)]
pub struct Entity {
	pub(crate) index: usize,
	pub(crate) generation: u32
}

impl Entity {
	pub(crate) fn new(index: usize, generation: u32) -> Self {
		Self {
			index,
			generation
		}
	}

	pub fn index(&self) -> usize {
		self.index
	}

	pub fn generation(&self) -> u32 {
		self.generation
	}

	pub fn decompose(&self) -> (usize, u32) {
		(self.index, self.generation)
	}
}

impl PartialEq for Entity {
	fn eq(&self, other: &Self) -> bool {
		self.index == other.index && self.generation == other.generation
	}
}

impl Eq for Entity {}

impl fmt::Display for Entity {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{{ index: {} generation: {} }}", self.index, self.generation)
	}
}