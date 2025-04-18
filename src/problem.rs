pub type Time = i64;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Job {
	index: usize,
	execution_time: Time,
	pub earliest_start: Time,
	pub latest_start: Time,
}

impl Job {
	pub fn release_to_deadline(
		index: usize, release_time: Time, execution_time: Time, deadline: Time
	) -> Job {
		assert!(execution_time > 0);
		Job {
			index,
			execution_time,
			earliest_start: release_time,
			latest_start: deadline - execution_time
		}
	}

	pub fn dummy() -> Job {
		Job { index: 0, execution_time: 1, earliest_start: 0, latest_start: 0 }
	}

	pub fn get_index(&self) -> usize { self.index }

	pub fn get_execution_time(&self) -> Time { self.execution_time }

	pub fn get_earliest_finish(&self) -> Time {
		self.earliest_start + self.execution_time
	}

	pub fn get_latest_finish(&self) -> Time {
		self.latest_start + self.execution_time
	}

	pub fn set_earliest_finish(&mut self, earliest_finish: Time) {
		self.earliest_start = earliest_finish - self.execution_time;
	}

	pub fn set_latest_finish(&mut self, latest_finish: Time) {
		self.latest_start = latest_finish - self.execution_time;
	}

	pub fn is_certainly_infeasible(&self) -> bool {
		self.earliest_start > self.latest_start
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ConstraintType {
	StartToStart,
	FinishToStart,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Constraint {
	before: usize,
	after: usize,
	constraint_type: ConstraintType,
	delay: Time,
}

impl Constraint {
	pub fn new(before: usize, after: usize, delay: Time, constraint_type: ConstraintType) -> Constraint {
		Constraint { before, after, constraint_type, delay }
	}

	pub fn dummy() -> Constraint {
		Constraint { before: 0, after: 0, constraint_type: ConstraintType::StartToStart, delay: 0 }
	}

	pub fn get_before(&self) -> usize { self.before }

	pub fn get_after(&self) -> usize { self.after }

	pub fn get_type(&self) -> ConstraintType { self.constraint_type }

	pub fn get_delay(&self) -> Time { self.delay }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Problem {
	pub jobs: Vec<Job>,
	pub constraints: Vec<Constraint>,
	pub num_cores: u32,
}

impl Problem {

	/// Checks whether this problem is valid:
	/// - `jobs[index].index = index` for all `0 <= index < jobs.len()`
	/// - `c.delay >= 0` for all constraints `c`
	/// - `c.before < jobs.len() && c.after < jobs.len()` for all constraints `c`
	pub fn validate(&self) {
		for (index, job) in self.jobs.iter().enumerate() {
			assert_eq!(job.get_index(), index);
		}

		for constraint in &self.constraints {
			assert!(constraint.get_delay() >= 0);
			assert!(constraint.get_before() < self.jobs.len());
			assert!(constraint.get_after() < self.jobs.len());
		}
	}

	/// A very simple sufficient test that checks whether this problem is certainly infeasible.
	pub fn is_certainly_infeasible(&self) -> bool {
		self.jobs.iter().any(|j| j.is_certainly_infeasible())
	}

	/// Changes `jobs[i].index` to `i`, for all `0 <= i < jobs.len()`
	pub fn update_job_indices(&mut self) {
		for index in 0 .. self.jobs.len() {
			self.jobs[index].index = index;
		}
	}

	/// Tests whether the 'default' job ordering of this problem is **possible**. More formally,
	/// whether `c.before < c.after` holds for all constraints.
	pub fn is_job_order_possible(&self) -> bool {
		self.constraints.iter().all(|constraint| constraint.before < constraint.after)
	}
}

#[cfg(test)]
mod tests {
	use super::{Job, Problem};

	#[test]
	fn test_job() {
		let mut job = Job::release_to_deadline(5, 2, 10, 15);
		assert_eq!(job.get_index(), 5);
		assert_eq!(job.get_execution_time(), 10);
		assert_eq!(job.earliest_start, 2);
		assert_eq!(job.latest_start, 5);
		assert_eq!(job.get_earliest_finish(), 12);
		assert_eq!(job.get_latest_finish(), 15);
		assert!(!job.is_certainly_infeasible());

		job.set_earliest_finish(11);
		assert_eq!(job.get_earliest_finish(), 11);
		assert_eq!(job.earliest_start, 1);
		assert!(!job.is_certainly_infeasible());

		job.set_latest_finish(10);
		assert_eq!(job.get_latest_finish(), 10);
		assert_eq!(job.latest_start, 0);
		assert!(job.is_certainly_infeasible());
	}

	#[test]
	fn test_problem() {
		let mut problem = Problem {
			jobs: vec![Job::release_to_deadline(0, 0, 10, 15)],
			constraints: vec![],
			num_cores: 2,
		};
		assert!(!problem.is_certainly_infeasible());
		problem.validate();

		problem.jobs.push(Job::release_to_deadline(1, 10, 10, 15));
		assert!(problem.is_certainly_infeasible());
		problem.validate();
	}
}
