use crate::problem::*;

struct JobBuilder {
	/// The job index
	job: usize,

	/// The total number of predecessors/successors
	count: usize,

	/// The offset into the `sorted_constraints` vector
	offset: usize,

	/// The number of remaining successors/predecessors
	remaining: i32,
}

impl JobBuilder {
	fn new(job: usize) -> JobBuilder {
		JobBuilder { job, count: 0, offset: 0, remaining: 0 }
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StrengthenConstraintsResult {
	Nothing,
	Modified,
	Cyclic,
}

pub fn strengthen_bounds_using_constraints(problem: &mut Problem) -> StrengthenConstraintsResult {
	let mut builders: Vec<JobBuilder> = problem.jobs.iter().map(
		|job| JobBuilder::new(job.get_index())
	).collect();

	// Sort the constraints by their successor/after job
	let mut sorted_constraints = vec![
		Constraint::new(0, 0, 0, ConstraintType::StartToStart);
		problem.constraints.len()
	];

	for constraint in &problem.constraints {
		builders[constraint.get_before()].remaining += 1;
		builders[constraint.get_after()].count += 1;
	}

	{
		let mut offset = 0;
		for builder in &mut builders {
			let old_offset = offset;
			offset += builder.count;
			builder.offset = old_offset;
			builder.count = 0;
		}
	}

	for constraint in &problem.constraints {
		let successor = &mut builders[constraint.get_after()];
		sorted_constraints[successor.offset + successor.count] = *constraint;
		successor.count += 1;
	}

	// Update the latest_start of each job
	let mut completed_jobs: Vec<usize> = Vec::with_capacity(builders.len());
	let mut next_jobs: Vec<usize> = Vec::with_capacity(builders.len());
	for builder in &builders {
		if builder.remaining == 0 {
			next_jobs.push(builder.job);
		}
	}

	let mut result = StrengthenConstraintsResult::Nothing;
	while !next_jobs.is_empty() {
		let successor = next_jobs.pop().unwrap();
		let latest_successor_start = problem.jobs[successor].latest_start;
		let start_index = builders[successor].offset;
		let bound_index = start_index + builders[successor].count;

		for constraint in &sorted_constraints[start_index..bound_index] {
			let predecessor = &mut builders[constraint.get_before()];
			predecessor.remaining -= 1;
			assert!(predecessor.remaining >= 0);

			let mut max_time_gap = constraint.get_delay();
			if constraint.get_type() == ConstraintType::FinishToStart {
				max_time_gap += problem.jobs[predecessor.job].get_execution_time();
			}

			let predecessor_job = &mut problem.jobs[predecessor.job];
			let constraint_bound = latest_successor_start - max_time_gap;
			if constraint_bound < predecessor_job.latest_start {
				predecessor_job.latest_start = constraint_bound;
				result = StrengthenConstraintsResult::Modified;
			}

			if predecessor.remaining == 0 {
				next_jobs.push(predecessor.job);
			}
		}
		completed_jobs.push(successor);
	}

	// If we did not encounter all jobs, the problem is cyclic
	if completed_jobs.len() != problem.jobs.len() {
		return StrengthenConstraintsResult::Cyclic;
	}

	// Sort the constraints by their predecessor job
	for builder in &mut builders {
		builder.count = 0;
		builder.remaining = 0;
	}

	for constraint in &problem.constraints {
		builders[constraint.get_before()].count += 1;
		builders[constraint.get_after()].remaining += 1;
	}

	{
		let mut offset = 0;
		for builder in &mut builders {
			let old_offset = offset;
			offset += builder.count;
			builder.offset = old_offset;
			builder.count = 0;
		}
	}

	for constraint in &problem.constraints {
		let predecessor = &mut builders[constraint.get_before()];
		sorted_constraints[predecessor.offset + predecessor.count] = *constraint;
		predecessor.count += 1;
	}

	// Update the earliest_start of each job
	for predecessor in completed_jobs.iter().rev() {
		let earliest_predecessor_start = problem.jobs[*predecessor].earliest_start;
		let predecessor_execution_time = problem.jobs[*predecessor].get_execution_time();
		let start_index = builders[*predecessor].offset;
		let bound_index = start_index + builders[*predecessor].count;

		for constraint in &sorted_constraints[start_index..bound_index] {
			let successor = &mut builders[constraint.get_after()];
			successor.remaining -= 1;
			assert!(successor.remaining >= 0);

			let mut max_time_gap = constraint.get_delay();
			if constraint.get_type() == ConstraintType::FinishToStart {
				max_time_gap += predecessor_execution_time;
			}

			let successor_job = &mut problem.jobs[successor.job];
			let constraint_bound = earliest_predecessor_start + max_time_gap;
			if constraint_bound > successor_job.earliest_start {
				successor_job.earliest_start = constraint_bound;
				result = StrengthenConstraintsResult::Modified;
			}
		}
	}

	result
}

#[cfg(test)]
mod tests {
	use crate::bounds::*;
	use crate::parse_problem;
	use crate::problem::Job;

	#[test]
	fn sanity_check_without_precedence_constraints() {
		let jobs_file = "./test-problems/infeasible/difficulty0/case1-cores1.csv";
		let mut problem = parse_problem(jobs_file, None, 1);
		assert_eq!(strengthen_bounds_using_constraints(&mut problem), StrengthenConstraintsResult::Nothing);
		assert_eq!(Job::release_to_deadline(0, 40, 10, 100), problem.jobs[0]);
		assert_eq!(Job::release_to_deadline(1, 0, 20, 100), problem.jobs[1]);
		assert_eq!(Job::release_to_deadline(2, 75, 30, 100), problem.jobs[2]);
	}

	#[test]
	fn test_simple_feasible_chain() {
		let jobs_file = "./test-problems/feasible/1core/case1.csv";
		let constraints_file = "./test-problems/feasible/1core/case1.prec.csv";
		let mut problem = parse_problem(jobs_file, Some(constraints_file), 1);
		assert_eq!(strengthen_bounds_using_constraints(&mut problem), StrengthenConstraintsResult::Modified);
		assert_eq!(8, problem.jobs[0].latest_start);
		assert_eq!(16, problem.jobs[2].latest_start);
		assert_eq!(27, problem.jobs[1].latest_start);

		assert_eq!(4, problem.jobs[0].earliest_start);
		assert_eq!(11, problem.jobs[2].earliest_start); // 4 + 2 + 5
		assert_eq!(22, problem.jobs[1].earliest_start); // 11 + 9 + 2

		assert!(!problem.is_certainly_infeasible());
	}

	#[test]
	fn test_simple_infeasible_chain() {
		let jobs_file = "./test-problems/infeasible/difficulty1/case1-cores1.csv";
		let constraints_file = "./test-problems/infeasible/difficulty1/case1.prec.csv";
		let mut problem = parse_problem(jobs_file, Some(constraints_file), 1);
		assert_eq!(strengthen_bounds_using_constraints(&mut problem), StrengthenConstraintsResult::Modified);
		assert_eq!(0, problem.jobs[0].latest_start);
		assert_eq!(7, problem.jobs[2].latest_start);
		assert_eq!(18, problem.jobs[1].latest_start);

		assert_eq!(4, problem.jobs[0].earliest_start);
		assert_eq!(11, problem.jobs[2].earliest_start); // 4 + 2 + 5
		assert_eq!(22, problem.jobs[1].earliest_start); // 11 + 9 + 2

		assert!(problem.is_certainly_infeasible());
	}
}
