use std::marker::PhantomData;

pub struct Handle<T> {
	type_marker: PhantomData<T>,
	pub index: usize,
	pub generation: u32
}

impl<T> Clone for Handle<T> {
	fn clone(&self) -> Self {
		Self {
			type_marker: PhantomData,
			index: self.index,
			generation: self.generation
		}
	}
}

impl<T> Copy for Handle<T> {}

impl<T> Handle<T> {
	pub fn null() -> Self {
		Self {
			type_marker: PhantomData,
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

	pub fn add(&mut self, payload: T) -> Handle<T> {
		if let Some(index) = self.vacant_records.pop() {
			let record = &mut self.records[index];
			let new_generation = record.generation + 1;

			record.generation = new_generation;
			record.payload = Some(payload);

			Handle {
				type_marker: PhantomData,
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
				type_marker: PhantomData,
				generation,
				index: self.records.len() - 1
			}
		}
	}

	pub fn valid(&self, handle: &Handle<T>) -> bool {
		return handle.index < self.records.len() && handle.generation == self.records[handle.index].generation;
	}

	fn get_record(&self, handle: &Handle<T>) -> Option<&Record<T>> {
		if handle.index >= self.records.len() {
			return None;
		}

		let record = &self.records[handle.index];

		if handle.generation != record.generation {
			return None;
		}

		Some(record)
	}

	fn get_record_mut(&mut self, handle: &Handle<T>) -> Option<&mut Record<T>> {
		if handle.index >= self.records.len() {
			return None;
		}

		let record = &mut self.records[handle.index];

		if handle.generation != record.generation {
			return None;
		}

		Some(record)
	}

	pub fn remove(&mut self, handle: &Handle<T>) {
		if let Some(record) = self.get_record_mut(handle) {
			record.payload = None;
			self.vacant_records.push(handle.index);
		}
	}

	pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
		if let Some(record) = self.get_record(handle) {
			record.payload.as_ref()
		}
		else {
			None
		}
	}

	pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
		if let Some(record) = self.get_record_mut(handle) {
			record.payload.as_mut()
		}
		else {
			None
		}
	}

	pub fn len(&self) -> usize {
		self.records.len()
	}

	pub fn present_len(&self) -> usize {
		self.records.len() - self.vacant_records.len()
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