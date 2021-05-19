#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Handle {
	index: usize,
	generation: u32
}

impl Handle {
	pub fn null() -> Self {
		Self {
			index: 0,
			generation: 0
		}
	}

	pub fn index(&self) -> usize {
		self.index
	}

	pub fn generation(&self) -> u32 {
		self.generation
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
			record.generation += 1;
			record.payload = Some(payload);

			Handle {
				index,
				generation: record.generation
			}
		}
		else {
			let generation = 1;

			self.records.push(Record {
				generation,
				payload: Some(payload)
			});

			Handle {
				index: self.records.len() - 1,
				generation
			}
		}
	}

	pub fn valid_handle(&self, handle: Handle) -> bool {
		if handle.index >= self.records.len() {
			return false;
		}

		let record = &self.records[handle.index];
		return handle.generation == record.generation && record.payload.is_some()
	}

	pub fn remove(&mut self, handle: Handle) {
		if self.valid_handle(handle) {
			let record = &mut self.records[handle.index];
			record.payload = None;
			self.vacant_records.push(handle.index);
		}
	}

	pub fn borrow(&self, handle: Handle) -> Option<&T> {
		if self.valid_handle(handle) {
			self.records[handle.index].payload.as_ref()
		}
		else {
			None
		}
	}

	pub fn borrow_mut(&mut self, handle: Handle) -> Option<&mut T> {
		if self.valid_handle(handle) {
			self.records[handle.index].payload.as_mut()
		}
		else {
			None
		}
	}

	pub(crate) fn borrow_unchecked(&self, handle: Handle) -> &T {
		self.records[handle.index].payload.as_ref().unwrap()
	}

	pub(crate) fn borrow_mut_unchecked(&mut self, handle: Handle) -> &mut T {
		self.records[handle.index].payload.as_mut().unwrap()
	}

	pub fn capacity(&self) -> usize {
		self.records.len()
	}

	pub fn occupied_record_count(&self) -> usize {
		self.records.len() - self.vacant_records.len()
	}

	pub fn is_empty(&self) -> bool {
		self.occupied_record_count() == 0
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
mod tests {
	use super::*;

	#[test]
	fn null() {
		let handle = Handle::null();

		assert_eq!(handle.index, 0);
		assert_eq!(handle.generation, 0);
	}

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

		pool.remove(handle);
		let handle = pool.add(6);
		assert_eq!(handle.index, 1);
		assert_eq!(handle.generation, 2);
	}

	#[test]
	fn remove() {
		let mut pool = Pool::<u32>::new();
		let handle = pool.add(4);
		pool.remove(handle);

		assert!(pool.records[0].payload.is_none());
		assert_eq!(pool.vacant_records[0], 0);
	}

	#[test]
	fn borrow() {
		let mut pool = Pool::<u32>::new();
		let handle = Handle::null();

		assert!(pool.borrow(handle).is_none());

		let handle = pool.add(4);
		assert_eq!(pool.borrow(handle), Some(&4));
	}

	#[test]
	fn borrow_mut() {
		let mut pool = Pool::<u32>::new();
		let handle = Handle::null();

		assert!(pool.borrow_mut(handle).is_none());

		let handle = pool.add(4);
		assert_eq!(pool.borrow_mut(handle), Some(&mut 4));
	}

	#[test]
	fn capacity() {
		let mut pool = Pool::<u32>::new();
		let handle = pool.add(4);
		pool.add(6);
		pool.remove(handle);

		assert_eq!(pool.capacity(), 2);
	}

	#[test]
	fn occupied_record_count() {
		let mut pool = Pool::<u32>::new();
		let handle = pool.add(4);
		pool.add(6);
		pool.remove(handle);

		assert_eq!(pool.occupied_record_count(), 1);
	}

	#[test]
	fn into_iter() {
		let mut pool = Pool::<u32>::new();
		
		pool.add(0);
		let handle_1 = pool.add(1);
		let handle_2 = pool.add(2);
		pool.add(3);

		pool.remove(handle_1);
		pool.remove(handle_2);

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

		pool.remove(handle_1);
		pool.remove(handle_2);

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

		pool.remove(handle_1);
		pool.remove(handle_2);

		let mut iter = pool.iter_mut();
		assert_eq!(iter.next(), Some(&mut 0));
		assert_eq!(iter.next(), Some(&mut 3));
		assert_eq!(iter.next(), None);
	}
}