pub struct DenseIndexSet {
	raw: Vec<u64>
}

impl DenseIndexSet {
	pub fn new(max_value: usize) -> DenseIndexSet {
		let capacity = (max_value + 1).div_ceil(64);
		DenseIndexSet { raw: vec![0; capacity] }
	}

	pub fn contains(&self, index: usize) -> bool {
		self.raw[index / 64] & (1 << (index % 64)) != 0
	}

	pub fn insert(&mut self, index: usize) {
		self.raw[index / 64] |= 1 << (index % 64);
	}

	pub fn remove(&mut self, index: usize) {
		self.raw[index / 64] &= !(1 << (index % 64));
	}
}

impl<'a> IntoIterator for DenseIndexSet {
	type Item = usize ;
	type IntoIter = DenseIterator<'a>;

	fn into_iter(self) -> Self::IntoIter {
		DenseIterator { set: &self, next: 0 }
	}
}

struct DenseIterator<'a> {
	set: &'a DenseIndexSet,
	next: usize,
}

impl Iterator for DenseIterator {
	type Item = usize;

	fn next(&mut self) -> Option<Self::Item> {
		// TODO
		while self.next < self.set.len() && self.set.raw[self.next / 64] == 0 {
			self.next = (1 + self.next / 64) * 64;
		}
		todo!()
	}
}

#[cfg(test)]
mod tests {
	use super::DenseIndexSet;

	#[test]
	fn test_max0() {
		let mut set = DenseIndexSet::new(0);
		assert_eq!(set.raw.len(), 1);
		assert!(!set.contains(0));
		set.insert(0);
		assert!(set.contains(0));
		set.remove(0);
		assert!(!set.contains(0));
	}

	#[test]
	fn test_max63() {
		let mut set = DenseIndexSet::new(63);
		assert_eq!(set.raw.len(), 1);
		assert!(!set.contains(0));
		assert!(!set.contains(10));
		assert!(!set.contains(63));
		set.insert(0);
		set.insert(10);
		set.insert(63);
		assert!(set.contains(0));
		assert!(set.contains(10));
		assert!(set.contains(63));
		assert!(!set.contains(1));
		assert!(!set.contains(11));
		assert!(!set.contains(62));
		set.remove(63);
		assert!(!set.contains(63));
		assert!(set.contains(10));
	}

	#[test]
	fn test_max64() {
		let mut set = DenseIndexSet::new(64);
		assert_eq!(set.raw.len(), 2);
		set.insert(0);
		set.insert(64);
		assert!(set.contains(0));
		assert!(set.contains(64));
		set.remove(64);
		assert!(!set.contains(64));
		assert!(set.contains(0));
	}
}
