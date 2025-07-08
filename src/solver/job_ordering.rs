use std::ops::Index;
use crate::problem::*;

#[derive(Clone)]
pub struct JobOrdering {
	jobs: Vec<usize>
}

impl JobOrdering {
	pub fn new<F>(problem: &Problem, compare: F) -> Self where F: Fn(&Job, &Job) -> core::cmp::Ordering {
		let mut jobs: Vec<_> = problem.jobs.iter().map(|j| j.get_index()).collect();
		jobs.sort_by(|j1, j2| compare(&problem.jobs[*j1], &problem.jobs[*j2]));
		Self { jobs }
	}
}

impl Index<usize> for JobOrdering {
	type Output = usize;

	fn index(&self, index: usize) -> &Self::Output {
		&self.jobs[index]
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_simple() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 100, 10, 200),
				Job::release_to_deadline(1, 0, 10, 200),
			],
			constraints: vec![],
			num_cores: 1
		};

		let ordering = JobOrdering::new(
			&problem, |j1, j2| j1.earliest_start.cmp(&j2.earliest_start)
		);
		assert_eq!(0, ordering[1]);
		assert_eq!(1, ordering[0]);
	}
}
