use std::marker::PhantomData;

#[derive(Debug, Eq, PartialEq)]
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
			index: 0,
			generation: 0
		}
	}

	pub fn new_first_gen(index: usize) -> Self {
		Self {
			type_marker: PhantomData,
			index,
			generation: 1
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
			let generation = 1;

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

	pub fn handle_valid(&self, handle: &Handle<T>) -> bool {
		return handle.index < self.records.len() && handle.generation == self.records[handle.index].generation;
	}

	fn get_record(&self, handle: &Handle<T>) -> Option<&Record<T>> {
		if self.handle_valid(handle) {
			Some(&self.records[handle.index])
		}
		else {
			None
		}
	}

	fn get_record_mut(&mut self, handle: &Handle<T>) -> Option<&mut Record<T>> {
		if self.handle_valid(handle) {
			Some(&mut self.records[handle.index])
		}
		else {
			None
		}
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

	pub fn total_len(&self) -> usize {
		self.records.len()
	}

	pub fn available_len(&self) -> usize {
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

impl<T> Default for Pool<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T> Iterator for Pool<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		let mut current_record = self.records.pop()?;

		while current_record.payload.is_none() {
			current_record = self.records.pop()?;
		}

		current_record.payload
	}
}

pub struct Iter<'a, T> {
	records: &'a Vec<Record<T>>,
	current_index: usize
}

impl<'a, T> Iterator for Iter<'a, T> {
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		if self.current_index == self.records.len() {
			return None;
		}

		let mut current_record = &self.records[self.current_index];

		while current_record.payload.is_none() {
			self.current_index += 1;

			if self.current_index == self.records.len() {
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
		if self.current_index == self.records.len() {
			return None;
		}

		let mut current_record = &mut self.records[self.current_index] as *mut Record<T>;

		while unsafe { &*current_record }.payload.is_none() {
			self.current_index += 1;

			if self.current_index == self.records.len() {
				return None;
			}

			current_record = &mut self.records[self.current_index];
		}

		self.current_index += 1;

		unsafe { &mut *current_record }.payload.as_mut()
	}
}

#[cfg(test)]
mod handle_tests {
	use super::*;

	#[test]
	fn null() {
		let handle = Handle::<u32>::null();

		assert_eq!(handle.index, 0);
		assert_eq!(handle.generation, 0);
	}

	#[test]
	fn clone() {
		let handle = Handle::<u32> {
			type_marker: PhantomData,
			index: 4,
			generation: 2
		};

		assert_eq!(handle, handle.clone());
	}
}

#[cfg(test)]
mod pool_tests {
	use super::*;

	#[test]
	fn new() {
		let pool = Pool::<u32>::new();

		assert!(pool.records.is_empty());
		assert!(pool.vacant_records.is_empty());
	}

	#[test]
	fn add() {
		let mut pool = Pool::<u32>::new();
		
		let handle = pool.add(4);
		assert_eq!(handle.index, 0);
		assert_eq!(handle.generation, 1);

		let handle = pool.add(5);
		assert_eq!(handle.index, 1);
		assert_eq!(handle.generation, 1);

		pool.remove(&handle);
		let handle = pool.add(6);
		assert_eq!(handle.index, 1);
		assert_eq!(handle.generation, 2);
	}

	#[test]
	fn get_record() {
		let mut pool = Pool::<u32>::new();
		let handle = Handle::<u32>::null();

		assert!(pool.get_record(&handle).is_none());

		pool.add(4);
		assert!(pool.get_record(&handle).is_none());
		
		let handle = Handle::<u32> {
			type_marker: PhantomData,
			index: 0,
			generation: 1
		};

		let record = pool.get_record(&handle).unwrap();
		assert_eq!(record.generation, 1);
		assert_eq!(record.payload, Some(4));
	}

	#[test]
	fn get_record_mut() {
		let mut pool = Pool::<u32>::new();
		let handle = Handle::<u32>::null();

		assert!(pool.get_record_mut(&handle).is_none());

		pool.add(4);
		assert!(pool.get_record_mut(&handle).is_none());
		
		let handle = Handle::<u32> {
			type_marker: PhantomData,
			index: 0,
			generation: 1
		};

		let record = pool.get_record_mut(&handle).unwrap();
		assert_eq!(record.generation, 1);
		assert_eq!(record.payload, Some(4));
	}

	#[test]
	fn remove() {
		let mut pool = Pool::<u32>::new();
		let handle = pool.add(4);
		pool.remove(&handle);

		assert!(pool.records[0].payload.is_none());
		assert_eq!(pool.vacant_records[0], 0);
	}

	#[test]
	fn get() {
		let mut pool = Pool::<u32>::new();
		let handle = Handle::<u32>::null();

		assert!(pool.get(&handle).is_none());

		let handle = pool.add(4);
		assert_eq!(pool.get(&handle), Some(&4));
	}

	#[test]
	fn get_mut() {
		let mut pool = Pool::<u32>::new();
		let handle = Handle::<u32>::null();

		assert!(pool.get_mut(&handle).is_none());

		let handle = pool.add(4);
		assert_eq!(pool.get_mut(&handle), Some(&mut 4));
	}

	#[test]
	fn total_len() {
		let mut pool = Pool::<u32>::new();
		let handle = pool.add(4);
		pool.add(6);
		pool.remove(&handle);

		assert_eq!(pool.total_len(), 2);
	}

	#[test]
	fn available_len() {
		let mut pool = Pool::<u32>::new();
		let handle = pool.add(4);
		pool.add(6);
		pool.remove(&handle);

		assert_eq!(pool.available_len(), 1);
	}

	#[test]
	fn into_iter() {
		let mut pool = Pool::<u32>::new();
		
		pool.add(0);
		let handle_1 = pool.add(1);
		let handle_2 = pool.add(2);
		pool.add(3);

		pool.remove(&handle_1);
		pool.remove(&handle_2);

		let mut iter = pool.into_iter();
		assert_eq!(iter.next(), Some(3));
		assert_eq!(iter.next(), Some(0));
		assert_eq!(iter.next(), None);
	}

	#[test]
	fn iter() {
		let mut pool = Pool::<u32>::new();
		
		pool.add(0);
		let handle_1 = pool.add(1);
		let handle_2 = pool.add(2);
		pool.add(3);

		pool.remove(&handle_1);
		pool.remove(&handle_2);

		let mut iter = pool.iter();
		assert_eq!(iter.next(), Some(&0));
		assert_eq!(iter.next(), Some(&3));
		assert_eq!(iter.next(), None);
	}

	#[test]
	fn iter_mut() {
		let mut pool = Pool::<u32>::new();
		
		pool.add(0);
		let handle_1 = pool.add(1);
		let handle_2 = pool.add(2);
		pool.add(3);

		pool.remove(&handle_1);
		pool.remove(&handle_2);

		let mut iter = pool.iter_mut();
		assert_eq!(iter.next(), Some(&mut 0));
		assert_eq!(iter.next(), Some(&mut 3));
		assert_eq!(iter.next(), None);
	}
}