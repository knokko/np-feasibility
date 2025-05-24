use crate::problem::*;

#[derive(Clone, Copy, Debug)]
struct FatJob {
	job: usize,
	value: Time,
}

/// An iterator of jobs sorted by a given function. This struct has a `next()` method that can be
/// used to get the next job (in order) if and only if it satisfies a given condition.
///
/// This struct is important for the feasibility load test.
#[derive(Debug)]
pub struct SortedJobIterator {
	jobs: Vec<FatJob>,
	index: usize
}

impl SortedJobIterator {

	/// Creates a new `SortedJobIterator`, where all jobs are sorted by their result of the
	/// `compute_value` function.
	pub fn new<T>(jobs: &Vec<Job>, compute_value: T) -> SortedJobIterator where T : Fn(&Job) -> Time {
		let mut fat_jobs: Vec<_> = jobs.iter().map(
			|j| FatJob { job: j.get_index(), value : compute_value(j) }
		).collect();
		fat_jobs.sort_by_key(|j| j.value);
		SortedJobIterator { jobs: fat_jobs, index: 0 }
	}

	/// Gets the next job (in order) if it satisfies `condition`. When the next job does *not*
	/// satisfy `condition`, this method returns `None`.
	pub fn next<T>(&mut self, condition: T) -> Option<usize> where T : Fn(Time) -> bool {
		let index = self.index;
		if index < self.jobs.len() && condition(self.jobs[index].value) {
			self.index += 1;
			Some(self.jobs[index].job)
		} else {
			None
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::problem::Job;
	use super::SortedJobIterator;

	#[test]
	fn test_sorted_job_iterator() {
		let jobs = vec![
			Job::release_to_deadline(0, 10, 15, 50),
			Job::release_to_deadline(1, 5, 15, 50),
			Job::release_to_deadline(2, 5, 15, 50),
			Job::release_to_deadline(3, 15, 150, 50),
		];

		let mut iterator = SortedJobIterator::new(&jobs, |job| job.earliest_start);
		assert_eq!(None, iterator.next(|time| time < 5));
		let first = iterator.next(|time| time <= 5).unwrap();
		let second = iterator.next(|time| time <= 6).unwrap();
		assert_eq!(None, iterator.next(|time| time <= 6));

		if first == 1 {
			assert_eq!(2, second);
		} else {
			assert_eq!(2, first);
			assert_eq!(1, second);
		}

		assert_eq!(Some(0), iterator.next(|time| time <= 15));
		assert_eq!(Some(3), iterator.next(|time| time <= 15));
		assert_eq!(None, iterator.next(|time| time <= 15));
	}
}
