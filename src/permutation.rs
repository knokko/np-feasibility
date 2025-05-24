use crate::problem::*;

struct JobBuilder {
	job: usize,
	num_successors: usize,

	/// The offset into the `sorted_constraints` vector
	offset: usize,
	remaining_predecessors: i32,
}

impl JobBuilder {
	fn new(job: usize) -> JobBuilder {
		JobBuilder { job, num_successors: 0, offset: 0, remaining_predecessors: 0 }
	}
}

/// Represents a permutation (reordering) of the jobs and constraints of a problem. Sometimes,
/// sorting/reordering the jobs and constraints makes later analysis steps much simpler.
/// Currently, we always use a **possible** permutation.
pub struct ProblemPermutation {
	jobs: Vec<usize>,
	constraints: Vec<usize>,
}

impl ProblemPermutation {

	/// This function creates a **possible** permutation of the jobs: the jobs will be sorted such
	/// that they can be started in order, *without violating any constraints*. More formally,
	/// after this function returns, it holds for any constraint `c` that `c.before < c.after`.
	/// Furthermore, all constraints are sorted by their `before` job.
	///
	/// This is very convenient for many steps in the feasibility analysis.
	///
	/// When no **possible** permutation exists (so when the constraints are cyclic), this function
	/// returns `None`.
	pub fn possible(problem: &mut Problem) -> Option<ProblemPermutation> {
		let mut builders: Vec<JobBuilder> = problem.jobs.iter().map(
			|job| JobBuilder::new(job.get_index())
		).collect();

		let mut sorted_constraints = vec![Constraint::dummy(); problem.constraints.len()];

		for constraint in &problem.constraints {
			builders[constraint.get_before()].num_successors += 1;
			builders[constraint.get_after()].remaining_predecessors += 1;
		}

		{
			let mut offset = 0;
			for builder in &mut builders {
				let old_offset = offset;
				offset += builder.num_successors;
				builder.offset = old_offset;
				builder.num_successors = 0;
			}
		}

		let mut constraint_permutation = vec![0usize; problem.constraints.len()];
		for index in 0 .. problem.constraints.len() {
			let constraint = problem.constraints[index];
			let predecessor = &mut builders[constraint.get_before()];
			sorted_constraints[predecessor.offset + predecessor.num_successors] = constraint;
			constraint_permutation[index] = predecessor.offset + predecessor.num_successors;
			predecessor.num_successors += 1;
		}

		let mut completed_jobs: Vec<usize> = Vec::with_capacity(builders.len());
		let mut next_jobs: Vec<usize> = Vec::with_capacity(builders.len());
		for builder in &builders {
			if builder.remaining_predecessors == 0 {
				next_jobs.push(builder.job);
			}
		}

		while !next_jobs.is_empty() {
			let predecessor = next_jobs.pop().unwrap();
			let start_index = builders[predecessor].offset;
			let bound_index = start_index + builders[predecessor].num_successors;

			for constraint in &sorted_constraints[start_index..bound_index] {
				let successor = &mut builders[constraint.get_after()];
				successor.remaining_predecessors -= 1;
				assert!(successor.remaining_predecessors >= 0);

				if successor.remaining_predecessors == 0 {
					next_jobs.push(successor.job);
				}
			}
			completed_jobs.push(predecessor);
		}

		// If we did not encounter all jobs, the problem is cyclic
		if completed_jobs.len() != problem.jobs.len() {
			return None;
		}

		let mut new_jobs: Vec<Job> = Vec::with_capacity(completed_jobs.len());
		for job_index in &completed_jobs {
			new_jobs.push(problem.jobs[*job_index]);
		}
		problem.jobs = new_jobs;
		problem.update_job_indices();

		for index in 0 .. sorted_constraints.len() {
			let old = sorted_constraints[index];
			let new = Constraint::new(
				completed_jobs[old.get_before()], completed_jobs[old.get_after()],
				old.get_delay(), old.get_type()
			);
			problem.constraints[index] = new;
		}

		Some(ProblemPermutation { jobs: completed_jobs, constraints: constraint_permutation })
	}

	/// Puts all jobs and precedence constraints back at their original position (index), and fixes
	/// all the indices.
	pub fn transform_back(self, problem: &mut Problem) {
		let mut reverse_job_mapping = vec![0usize; problem.jobs.len()];

		let mut new_jobs = vec![Job::dummy(); problem.jobs.len()];
		for original_index in 0 .. self.jobs.len() {
			let current_index = self.jobs[original_index];
			new_jobs[original_index] = problem.jobs[current_index];
			reverse_job_mapping[current_index] = original_index;
		}
		problem.jobs = new_jobs;
		problem.update_job_indices();

		let mut new_constraints = vec![Constraint::dummy(); problem.constraints.len()];
		for original_index in 0 .. self.constraints.len() {
			let current_index = self.constraints[original_index];
			let current_constraint = problem.constraints[current_index];
			let original_before = reverse_job_mapping[current_constraint.get_before()];
			let original_after = reverse_job_mapping[current_constraint.get_after()];
			let original_constraint = Constraint::new(
				original_before, original_after, current_constraint.get_delay(), current_constraint.get_type()
			);
			new_constraints[current_index] = original_constraint
		}
		problem.constraints = new_constraints;
	}
}

#[cfg(test)]
mod tests {
	use crate::parse_problem;
	use super::ProblemPermutation;

	#[test]
	fn sanity_check_without_precedence_constraints() {
		let jobs_file = "./test-problems/infeasible/difficulty0/case1-cores1.csv";
		let mut problem = parse_problem(jobs_file, None, 1);
		assert!(ProblemPermutation::possible(&mut problem).is_some());
		assert_eq!(problem.jobs.len(), 3);
	}

	#[test]
	fn test_simple_chain() {
		let jobs_file = "./test-problems/feasible/1core/case1.csv";
		let constraints_file = "./test-problems/feasible/1core/case1.prec.csv";
		let mut problem = parse_problem(jobs_file, Some(constraints_file), 1);
		let permutation = ProblemPermutation::possible(&mut problem).unwrap();
		problem.validate();
		assert_eq!(permutation.jobs, vec![0, 2, 1]);
		assert_eq!(problem.jobs[0].get_execution_time(), 2);
		assert_eq!(problem.jobs[1].get_execution_time(), 9);
		assert_eq!(problem.jobs[2].get_execution_time(), 3);
		assert_eq!(problem.constraints[0].get_before(), 0);
		assert_eq!(problem.constraints[0].get_after(), 1);
		assert_eq!(problem.constraints[0].get_delay(), 5);
		assert_eq!(problem.constraints[1].get_before(), 1);
		assert_eq!(problem.constraints[1].get_after(), 2);
		assert_eq!(problem.constraints[1].get_delay(), 2);

		permutation.transform_back(&mut problem);
		assert_eq!(problem, parse_problem(jobs_file, Some(constraints_file), 1));
	}

	#[test]
	fn test_simple_mixed_chain() {
		let jobs_file = "./test-problems/feasible/1core/case2.csv";
		let constraints_file = "./test-problems/feasible/1core/case2.prec.csv";
		let mut problem = parse_problem(jobs_file, Some(constraints_file), 123);
		let permutation = ProblemPermutation::possible(&mut problem).unwrap();
		problem.validate();
		assert_eq!(permutation.jobs.len(), 3);
		assert_eq!(permutation.jobs[0], 0);

		permutation.transform_back(&mut problem);
		assert_eq!(problem, parse_problem(jobs_file, Some(constraints_file), 123));
	}
}
