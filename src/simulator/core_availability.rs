use crate::problem::Time;

#[derive(Clone)]
pub struct CoreAvailability {
	finish_times: Vec<Time>,
	last_start_time: Time,
}

impl CoreAvailability {
	pub fn new(num_cores: usize) -> Self {
		Self {
			finish_times: vec![0; num_cores],
			last_start_time: 0,
		}
	}

	pub fn next_start_time(&self) -> Time {
		Time::max(self.finish_times[0], self.last_start_time)
	}

	pub fn second_start_time(&self) -> Time {
		Time::max(self.finish_times[1], self.last_start_time)
	}

	pub fn schedule(&mut self, start: Time, duration: Time) {
		debug_assert!(start >= self.next_start_time());
		self.finish_times[0] = start + duration;
		self.finish_times.sort();
		self.last_start_time = start;
	}

	pub fn merge(&mut self, other: &Self) {
		debug_assert_eq!(self.finish_times.len(), other.finish_times.len());
		for index in 0 .. self.finish_times.len() {
			self.finish_times[index] = Time::max(
				self.finish_times[index], other.finish_times[index]
			);
		}
		self.last_start_time = Time::max(self.last_start_time, other.last_start_time);
	}

	pub fn get_num_cores(&self) -> usize {
		self.finish_times.len()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_with_one_core() {
		let mut availability = CoreAvailability::new(1);
		assert_eq!(0, availability.next_start_time());
		availability.schedule(0, 4);
		assert_eq!(4, availability.next_start_time());
		availability.schedule(4, 5);
		assert_eq!(9, availability.next_start_time());
		availability.schedule(20, 3);
		assert_eq!(23, availability.next_start_time());
		availability.schedule(23, 1);
		assert_eq!(24, availability.next_start_time());

		assert_eq!(1, availability.get_num_cores());
	}

	#[test]
	fn test_with_three_cores() {
		let mut availability = CoreAvailability::new(3);
		assert_eq!(0, availability.next_start_time());
		assert_eq!(0, availability.second_start_time());

		availability.schedule(0, 10);
		assert_eq!(0, availability.next_start_time());
		assert_eq!(0, availability.second_start_time());

		availability.schedule(0, 5);
		assert_eq!(0, availability.next_start_time());
		assert_eq!(5, availability.second_start_time());

		availability.schedule(0, 20);
		assert_eq!(5, availability.next_start_time());
		assert_eq!(10, availability.second_start_time());

		availability.schedule(7, 1);
		assert_eq!(8, availability.next_start_time());
		assert_eq!(10, availability.second_start_time());

		availability.schedule(8, 5);
		assert_eq!(10, availability.next_start_time());
		assert_eq!(13, availability.second_start_time());

		availability.schedule(10, 20);
		assert_eq!(13, availability.next_start_time());
		assert_eq!(20, availability.second_start_time());

		availability.schedule(13, 100);
		assert_eq!(20, availability.next_start_time());
		assert_eq!(30, availability.second_start_time());

		assert_eq!(3, availability.get_num_cores());
	}

	#[test]
	fn test_merging() {
		let mut availability1 = CoreAvailability::new(2);
		availability1.schedule(1, 2);
		availability1.schedule(1, 6);
		assert_eq!(3, availability1.next_start_time());
		assert_eq!(7, availability1.second_start_time());

		let mut availability2 = CoreAvailability::new(2);
		availability2.schedule(2, 3);
		availability2.schedule(4, 2);
		assert_eq!(5, availability2.next_start_time());
		assert_eq!(6, availability2.second_start_time());

		availability2.merge(&availability1);
		assert_eq!(5, availability2.next_start_time());
		assert_eq!(7, availability2.second_start_time());

		availability2.schedule(5, 6);
		assert_eq!(7, availability2.next_start_time());
		assert_eq!(11, availability2.second_start_time());

		availability2.schedule(7, 8);
		assert_eq!(11, availability2.next_start_time());
		assert_eq!(15, availability2.second_start_time());

		assert_eq!(2, availability2.get_num_cores());
	}

	#[test]
	fn test_get_number_of_cores() {
		for num_cores in 1 .. 100 {
			assert_eq!(num_cores, CoreAvailability::new(num_cores).get_num_cores());
		}
	}
}
