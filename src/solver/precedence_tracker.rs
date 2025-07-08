use crate::problem::*;

#[derive(Clone)]
pub struct PrecedenceTracker {
	total_predecessors: Vec<usize>,
	successors: Vec<usize>,
	successor_offsets: Vec<usize>,
}

impl PrecedenceTracker {
	pub fn new(problem: &Problem) -> Self {
		let num_jobs = problem.jobs.len();
		let mut total_predecessors = vec![0; num_jobs];
		let mut successor_offsets = vec![0; 2 * num_jobs];
		for constraint in &problem.constraints {
			total_predecessors[constraint.get_after()] += 1;
			successor_offsets[2 * constraint.get_before() + 1] += 1;
		}

		let mut next_successor_offset = 0;
		for job in 0 .. num_jobs {
			successor_offsets[2 * job] = next_successor_offset;
			next_successor_offset += successor_offsets[2 * job + 1];
			successor_offsets[2 * job + 1] = 0;
		}

		let mut successors = vec![0; next_successor_offset];
		for constraint in &problem.constraints {
			let base_index = 2 * constraint.get_before();
			successors[successor_offsets[base_index] + successor_offsets[base_index + 1]] = constraint.get_after();
			successor_offsets[base_index + 1] += 1;
		}

		Self { total_predecessors, successors, successor_offsets, }
	}

	pub fn clone_total_predecessors(&self) -> Vec<usize> {
		self.total_predecessors.clone()
	}

	pub fn update_remaining_predecessors(
		&self, finished_job: usize, remaining_predecessors: &mut [usize]
	) {
		let successor_index = self.successor_offsets[2 * finished_job];
		let num_successors = self.successor_offsets[2 * finished_job + 1];
		for index in successor_index .. successor_index + num_successors {
			let successor = self.successors[index];
			remaining_predecessors[successor] -= 1;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_without_constraints() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 1, 2, 3),
				Job::release_to_deadline(1, 2, 3, 4)
			],
			constraints: vec![],
			num_cores: 1
		};

		let tracker = PrecedenceTracker::new(&problem);
		assert_eq!(0, tracker.successors.len());
		assert_eq!(vec![0; 4], tracker.successor_offsets);
		assert_eq!(vec![0; 2], tracker.total_predecessors);

		let mut remaining_predecessors = tracker.clone_total_predecessors();
		assert_eq!(vec![0; 2], remaining_predecessors);
		tracker.update_remaining_predecessors(0, &mut remaining_predecessors);
		tracker.update_remaining_predecessors(1, &mut remaining_predecessors);
		assert_eq!(vec![0; 2], remaining_predecessors);
	}

	#[test]
	fn test_with_one_constraint() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 1, 2, 3),
				Job::release_to_deadline(1, 2, 3, 4),
				Job::release_to_deadline(2, 3, 4, 5)
			],
			constraints: vec![
				Constraint::new(2, 1, 10, ConstraintType::FinishToStart)
			],
			num_cores: 5
		};

		let tracker = PrecedenceTracker::new(&problem);
		assert_eq!(vec![1], tracker.successors);
		assert_eq!(vec![0, 0, 0, 0, 0, 1], tracker.successor_offsets);
		assert_eq!(vec![0, 1, 0], tracker.total_predecessors);

		let mut remaining_predecessors = tracker.clone_total_predecessors();
		assert_eq!(vec![0, 1, 0], remaining_predecessors);
		tracker.update_remaining_predecessors(0, &mut remaining_predecessors);
		assert_eq!(vec![0, 1, 0], remaining_predecessors);
		tracker.update_remaining_predecessors(2, &mut remaining_predecessors);
		assert_eq!(vec![0, 0, 0], remaining_predecessors);
		tracker.update_remaining_predecessors(1, &mut remaining_predecessors);
		assert_eq!(vec![0, 0, 0], remaining_predecessors);
	}

	#[test]
	fn test_with_two_constraints() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 0, 20, 20),
				Job::release_to_deadline(1, 0, 30, 32),
				Job::release_to_deadline(2, 0, 30, 100)
			],
			constraints: vec![
				Constraint::new(2, 1, 2, ConstraintType::StartToStart),
				Constraint::new(0, 2, 10, ConstraintType::FinishToStart)
			],
			num_cores: 2
		};

		let tracker = PrecedenceTracker::new(&problem);
		assert_eq!(vec![2, 1], tracker.successors);
		assert_eq!(vec![0, 1, 1, 0, 1, 1], tracker.successor_offsets);
		assert_eq!(vec![0, 1, 1], tracker.total_predecessors);

		let mut remaining_predecessors = tracker.clone_total_predecessors();
		assert_eq!(vec![0, 1, 1], remaining_predecessors);
		tracker.update_remaining_predecessors(0, &mut remaining_predecessors);
		assert_eq!(vec![0, 1, 0], remaining_predecessors);
		tracker.update_remaining_predecessors(2, &mut remaining_predecessors);
		assert_eq!(vec![0, 0, 0], remaining_predecessors);
		tracker.update_remaining_predecessors(1, &mut remaining_predecessors);
		assert_eq!(vec![0, 0, 0], remaining_predecessors);
	}
}
