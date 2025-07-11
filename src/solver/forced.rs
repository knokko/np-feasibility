use crate::problem::*;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
struct FatJob {
	latest_start: Time,
	index: usize,
}

#[derive(Debug)]
pub struct ForcedJobsContext {
	jobs: Vec<FatJob>
}

impl ForcedJobsContext {
	pub fn new(problem: &Problem) -> Self {
		let mut jobs: Vec<_> = problem.jobs.iter().map(|job| FatJob {
			index: job.get_index(), latest_start: job.latest_start
		}).collect();
		jobs.sort();
		Self { jobs }
	}
}

#[derive(Debug)]
pub struct ForcedJobTracker {
	next_index: usize,
	after_index: usize,
}

impl ForcedJobTracker {
	pub fn new() -> Self {
		Self { next_index: 0, after_index: 1 }
	}

	pub fn update(&mut self, context: &ForcedJobsContext, dispatched_jobs: &[bool]) {
		println!("start update {:?} and context {:?} and dispatched {:?}", self, context, dispatched_jobs);
		while self.next_index < dispatched_jobs.len() && dispatched_jobs[context.jobs[self.next_index].index] {
			self.next_index += 1;
		}
		if self.after_index <= self.next_index {
			self.after_index = self.next_index + 1;
		}
		while self.after_index < dispatched_jobs.len() && dispatched_jobs[context.jobs[self.after_index].index] {
			self.after_index += 1;
		}
		println!("finish update {:?} and context {:?}", self, context);
	}

	pub fn can_schedule_safely(
		&self, context: &ForcedJobsContext, candidate_job: usize, next_start_time: Time
	) -> bool {
		println!("next index is {} and after index is {}", self.next_index, self.after_index);
		if context.jobs[self.next_index].index == candidate_job {
			if self.after_index >= context.jobs.len() {
				true
			} else {
				context.jobs[self.after_index].latest_start >= next_start_time
			}
		} else {
			context.jobs[self.next_index].latest_start >= next_start_time
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_problem_with_one_job() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 1, 2, 3),
			],
			constraints: vec![],
			num_cores: 1
		};

		let context = ForcedJobsContext::new(&problem);
		let mut tracker = ForcedJobTracker::new();
		assert!(tracker.can_schedule_safely(&context, 0, 10));
		tracker.update(&context, &vec![false]);
		assert!(tracker.can_schedule_safely(&context, 0, 10));
		tracker.update(&context, &vec![true]);
	}

	#[test]
	fn test_problem_with_three_jobs() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 10, 2, 15),
				Job::release_to_deadline(1, 20, 2, 30),
				Job::release_to_deadline(2, 0, 2, 20),
			],
			constraints: vec![],
			num_cores: 1
		};
		let context = ForcedJobsContext::new(&problem);

		let mut tracker = ForcedJobTracker::new();
		tracker.update(&context, &vec![false, false, false]);
		for job in &problem.jobs {
			assert!(tracker.can_schedule_safely(&context, job.get_index(), 0));
			assert!(tracker.can_schedule_safely(&context, job.get_index(), 13));
		}

		assert!(tracker.can_schedule_safely(&context, 0, 14));
		assert!(!tracker.can_schedule_safely(&context, 1, 14)); // Job 0 would miss its deadline
		assert!(!tracker.can_schedule_safely(&context, 2, 14)); // Job 0 would miss its deadline

		assert!(tracker.can_schedule_safely(&context, 0, 18));
		assert!(!tracker.can_schedule_safely(&context, 0, 19)); // Job 2 would miss its deadline

		tracker.update(&context, &vec![false, false, true]);
		assert!(tracker.can_schedule_safely(&context, 0, 19));
		assert!(tracker.can_schedule_safely(&context, 0, 20));
		assert!(!tracker.can_schedule_safely(&context, 0, 29)); // Job 1 would miss its deadline
		assert!(tracker.can_schedule_safely(&context, 2, 13));
		assert!(!tracker.can_schedule_safely(&context, 2, 14)); // Job 0 would miss its deadline

		tracker.update(&context, &vec![false, true, true]);
		assert!(tracker.can_schedule_safely(&context, 0, 29));
		assert!(tracker.can_schedule_safely(&context, 0, 99));
		tracker.update(&context, &vec![true, true, true]);
	}
}
