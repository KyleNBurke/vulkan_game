pub struct Handle {
	index: usize,
	generation: u32
}

impl Handle {
	pub fn null() -> Self {
		Self {
			index: std::usize::MAX,
			generation: std::u32::MAX
		}
	}
}

struct Record<T> {
	generation: u32,
	payload: Option<T>
}

pub struct Pool<T> {
	records: Vec<Record<T>>,
	vacant_records: Vec<usize>
}

impl<T> Pool<T> {
	pub fn new() -> Self {
		Self {
			records: vec![],
			vacant_records: vec![]
		}
	}

	pub fn add(&mut self, payload: T) -> Handle {
		if let Some(index) = self.vacant_records.pop() {
			let record = &mut self.records[index];
			let new_generation = record.generation + 1;

			record.generation = new_generation;
			record.payload = Some(payload);

			Handle {
				generation: new_generation,
				index
			}
		}
		else {
			let generation = 0;

			self.records.push(Record {
				generation,
				payload: Some(payload)
			});

			Handle {
				generation,
				index: self.records.len() - 1
			}
		}
	}

	fn get_record(&self, handle: &Handle) -> Option<&Record<T>> {
		if handle.index >= self.records.len() {
			return None;
		}

		let record = &self.records[handle.index];

		if handle.generation != record.generation {
			return None;
		}

		Some(record)
	}

	fn get_record_mut(&mut self, handle: &Handle) -> Option<&mut Record<T>> {
		if handle.index >= self.records.len() {
			return None;
		}

		let record = &mut self.records[handle.index];

		if handle.generation != record.generation {
			return None;
		}

		Some(record)
	}

	pub fn remove(&mut self, handle: &Handle) {
		if let Some(record) = self.get_record_mut(handle) {
			record.payload = None;
			self.vacant_records.push(handle.index);
		}
	}

	pub fn get(&self, handle: &Handle) -> Option<&T> {
		if let Some(record) = self.get_record(handle) {
			record.payload.as_ref()
		}
		else {
			None
		}
	}

	pub fn get_mut(&mut self, handle: &Handle) -> Option<&mut T> {
		if let Some(record) = self.get_record_mut(handle) {
			record.payload.as_mut()
		}
		else {
			None
		}
	}

	pub fn iter(&self) -> Iter<T> {
		Iter {
			records: &self.records,
			current_index: 0
		}
	}

	pub fn iter_mut(&mut self) -> IterMut<T> {
		IterMut {
			records: &mut self.records,
			current_index: 0
		}
	}
}

pub struct Iter<'a, T> {
	records: &'a Vec<Record<T>>,
	current_index: usize
}

impl<'a, T> Iterator for Iter<'a, T> {
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		if self.current_index >= self.records.len() {
			return None;
		}

		let mut current_record = &self.records[self.current_index];

		while current_record.payload.is_none() {
			self.current_index += 1;

			if self.current_index >= self.records.len() {
				return None;
			}

			current_record = &self.records[self.current_index];
		}

		self.current_index += 1;

		current_record.payload.as_ref()
	}
}

pub struct IterMut<'a, T> {
	records: &'a mut Vec<Record<T>>,
	current_index: usize
}

impl<'a, T> Iterator for IterMut<'a, T> {
	type Item = &'a mut T;

	fn next(&mut self) -> Option<Self::Item> {
		if self.current_index >= self.records.len() {
			return None;
		}

		let mut current_record = &mut self.records[self.current_index] as *mut Record<T>;

		while unsafe { &*current_record }.payload.is_none() {
			self.current_index += 1;

			if self.current_index >= self.records.len() {
				return None;
			}

			current_record = &mut self.records[self.current_index];
		}

		self.current_index += 1;

		unsafe { &mut *current_record }.payload.as_mut()
	}
}

// Implment into iter so you don't have to call .iter() or .iter_mut()