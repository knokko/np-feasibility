use crate::problem::*;
use crate::solver::job_ordering::JobOrdering;
use crate::solver::precedence_tracker::PrecedenceTracker;

/// A queue of jobs sorted by some `JobOrdering`, but with some special properties. The
/// `choose_next` method should be used to take the next job from this queue.
///
/// The `ordering` parameter determines which jobs are 'small', and which jobs are 'large'. The
/// simplest case is when `to_skip == 0`. In this case, `choose_next` will return the smallest job
/// that is *allowed*. A job is *allowed* when the queue does *not* contain any of its predecessors.
///
/// When `to_skip > 0`, the `choose_next` method will ignore the smallest `to_skip` *allowed* jobs,
/// and return the smallest remaining *allowed* job. When there are only `to_skip` or fewer
/// *allowed* jobs left, the *largest allowed* job will be returned.
#[derive(Clone)]
pub struct HeuristicJobQueue {
	already_dispatched: Vec<bool>, // TODO index set?
	remaining_predecessors: Vec<usize>,
	first_unscheduled: usize
}

impl HeuristicJobQueue {

	pub fn new(problem: &Problem, precedence: &PrecedenceTracker) -> Self {
		Self {
			already_dispatched: vec![false; problem.jobs.len()],
			remaining_predecessors: precedence.clone_total_predecessors(),
			first_unscheduled: 0
		}
	}

	/// Returns the *smallest* job (determined by `ordering`) after skipping the smallest
	/// `to_skip` jobs.
	///
	/// All jobs with unscheduled predecessors are ignored.
	///
	/// When `to_skip` or fewer jobs have no unscheduled predecessors, no jobs are skipped, and
	/// the *largest* job is returned instead.
	pub fn choose_next(
		&mut self, to_skip: u32, ordering: &JobOrdering, precedence: &PrecedenceTracker
	) -> usize {
		let mut skip_remaining = to_skip;
		let mut next_order = self.first_unscheduled;

		let mut last_valid: Option<usize> = None;
		loop {
			if next_order >= self.already_dispatched.len() {
				next_order = last_valid.expect("Not a single job can be chosen");
				let next_job = ordering[next_order];
				self.already_dispatched[next_order] = true;
				precedence.update_remaining_predecessors(next_job, &mut self.remaining_predecessors);
				return next_job;
			}

			if self.already_dispatched[next_order] {
				if self.first_unscheduled == next_order {
					self.first_unscheduled += 1;
				}
				next_order += 1;
				continue
			}

			let next_job = ordering[next_order];
			if self.remaining_predecessors[next_job] != 0 {
				next_order += 1;
				continue
			}

			last_valid = Some(next_order);

			if skip_remaining > 0 {
				next_order += 1;
				skip_remaining -= 1;
				continue;
			}

			self.already_dispatched[next_order] = true;
			precedence.update_remaining_predecessors(next_job, &mut self.remaining_predecessors);
			return next_job;
		}
	}
}

#[cfg(test)]
mod tests {
	use std::cmp::Ordering;
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

		let ordering = JobOrdering::new(&problem, |_j1, _j2| Ordering::Equal);
		let tracker = PrecedenceTracker::new(&problem);
		let mut queue = HeuristicJobQueue::new(&problem, &tracker);
		assert_eq!(0, queue.choose_next(0, &ordering, &tracker));

		assert!(std::panic::catch_unwind(
			move || queue.choose_next(0, &ordering, &tracker)
		).is_err());
	}

	// TODO more testing
	#[test]
	fn test_problem_with_three_jobs() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 10, 2, 30),
				Job::release_to_deadline(1, 20, 2, 30),
				Job::release_to_deadline(2, 0, 2, 30),
			],
			constraints: vec![],
			num_cores: 1
		};

		let ordering = JobOrdering::new(
			&problem, |j1, j2| j1.earliest_start.cmp(&j2.earliest_start)
		);
		let tracker = PrecedenceTracker::new(&problem);

		let mut queue0 = HeuristicJobQueue::new(&problem, &tracker);
		assert_eq!(2, queue0.choose_next(0, &ordering, &tracker));
		assert_eq!(0, queue0.choose_next(0, &ordering, &tracker));
		assert_eq!(1, queue0.choose_next(0, &ordering, &tracker));
		assert!(std::panic::catch_unwind(
			|| queue0.clone().choose_next(0, &ordering, &tracker)
		).is_err());

		let mut queue1 = HeuristicJobQueue::new(&problem, &tracker);
		assert_eq!(0, queue1.choose_next(1, &ordering, &tracker));
		assert_eq!(1, queue1.choose_next(1, &ordering, &tracker));
		assert_eq!(2, queue1.choose_next(1, &ordering, &tracker));
		assert!(std::panic::catch_unwind(
			|| queue1.clone().choose_next(1, &ordering, &tracker)
		).is_err());

		let mut queue2 = HeuristicJobQueue::new(&problem, &tracker);
		assert_eq!(1, queue2.choose_next(2, &ordering, &tracker));
		assert_eq!(0, queue2.choose_next(2, &ordering, &tracker));
		assert_eq!(2, queue2.choose_next(2, &ordering, &tracker));
		assert!(std::panic::catch_unwind(
			|| queue2.clone().choose_next(2, &ordering, &tracker)
		).is_err());

		let mut queue3 = HeuristicJobQueue::new(&problem, &tracker);
		assert_eq!(1, queue3.choose_next(3, &ordering, &tracker));
		assert_eq!(0, queue3.choose_next(3, &ordering, &tracker));
		assert_eq!(2, queue3.choose_next(3, &ordering, &tracker));
		assert!(std::panic::catch_unwind(
			|| queue3.clone().choose_next(3, &ordering, &tracker)
		).is_err());
	}

	#[test]
	fn test_problem_with_constraint() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 10, 2, 30),
				Job::release_to_deadline(1, 20, 2, 30),
				Job::release_to_deadline(2, 0, 2, 30),
			],
			constraints: vec![
				Constraint::new(0, 2, 3, ConstraintType::StartToStart)
			],
			num_cores: 1
		};

		let ordering = JobOrdering::new(
			&problem, |j1, j2| j1.earliest_start.cmp(&j2.earliest_start)
		);
		let tracker = PrecedenceTracker::new(&problem);

		let mut queue0 = HeuristicJobQueue::new(&problem, &tracker);
		assert_eq!(0, queue0.choose_next(0, &ordering, &tracker));
		assert_eq!(2, queue0.choose_next(0, &ordering, &tracker));
		assert_eq!(1, queue0.choose_next(0, &ordering, &tracker));

		let mut queue1 = HeuristicJobQueue::new(&problem, &tracker);
		assert_eq!(1, queue1.choose_next(1, &ordering, &tracker));
		assert_eq!(0, queue1.choose_next(1, &ordering, &tracker));
		assert_eq!(2, queue1.choose_next(1, &ordering, &tracker));

		let mut queue2 = HeuristicJobQueue::new(&problem, &tracker);
		assert_eq!(1, queue2.choose_next(2, &ordering, &tracker));
		assert_eq!(0, queue2.choose_next(2, &ordering, &tracker));
		assert_eq!(2, queue2.choose_next(2, &ordering, &tracker));
	}
}
