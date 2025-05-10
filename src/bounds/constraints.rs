use crate::problem::*;

/// Attempts to strengthen the bounds of the jobs of the given problem (their `earliest_start` and
/// `latest_start`), by analyzing their successors and predecessors. This function ensures that
/// for all finish-to-start constraints `c` that:
/// - `problem.jobs[c.before].earliest_start + problem.jobs[c.before].execution_time + c.delay
///   <= problem.jobs[c.after].earliest_start`
///
/// Furthermore, for all start-to-start constraints `c`:
/// - `problem.jobs[c.before].earliest_start + c.delay <= problem.jobs[c.after].earliest_start`
///
/// Returns true if and only if the `earliest_start` or `latest_start` of at least 1 job has
/// been changed.
pub fn strengthen_bounds_using_constraints(problem: &mut Problem) -> bool {
	debug_assert!(problem.is_job_order_possible());

	let mut result = false;
	for index in 0 .. problem.constraints.len() {
		let constraint = problem.constraints[index];
		let mut earliest_start = problem.jobs[constraint.get_before()].earliest_start + constraint.get_delay();
		if constraint.get_type() == ConstraintType::FinishToStart {
			earliest_start += problem.jobs[constraint.get_before()].get_execution_time();
		}
		if earliest_start > problem.jobs[constraint.get_after()].earliest_start {
			problem.jobs[constraint.get_after()].earliest_start = earliest_start;
			result = true;
		}
	}

	for index in (0 .. problem.constraints.len()).rev() {
		let constraint = problem.constraints[index];
		let mut latest_start = problem.jobs[constraint.get_after()].latest_start - constraint.get_delay();
		if constraint.get_type() == ConstraintType::FinishToStart {
			latest_start -= problem.jobs[constraint.get_before()].get_execution_time();
		}
		if latest_start < problem.jobs[constraint.get_before()].latest_start {
			problem.jobs[constraint.get_before()].latest_start = latest_start;
			result = true;
		}
	}

	result
}

#[cfg(test)]
mod tests {
	use crate::bounds::*;
	use crate::parse_problem;
	use crate::permutation::ProblemPermutation;
	use crate::problem::Job;

	#[test]
	fn sanity_check_without_precedence_constraints() {
		let jobs_file = "./test-problems/infeasible/difficulty0/case1-cores1.csv";
		let mut problem = parse_problem(jobs_file, None, 1);
		let permutation = ProblemPermutation::possible(&mut problem).unwrap();
		assert!(!strengthen_bounds_using_constraints(&mut problem));
		permutation.transform_back(&mut problem);
		assert_eq!(Job::release_to_deadline(0, 40, 10, 100), problem.jobs[0]);
		assert_eq!(Job::release_to_deadline(1, 0, 20, 100), problem.jobs[1]);
		assert_eq!(Job::release_to_deadline(2, 75, 30, 100), problem.jobs[2]);
	}

	#[test]
	fn test_simple_feasible_chain() {
		let jobs_file = "./test-problems/feasible/1core/case1.csv";
		let constraints_file = "./test-problems/feasible/1core/case1.prec.csv";
		let mut problem = parse_problem(jobs_file, Some(constraints_file), 1);
		let permutation = ProblemPermutation::possible(&mut problem).unwrap();
		assert!(strengthen_bounds_using_constraints(&mut problem));
		permutation.transform_back(&mut problem);
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
		let permutation = ProblemPermutation::possible(&mut problem).unwrap();
		assert!(strengthen_bounds_using_constraints(&mut problem));
		permutation.transform_back(&mut problem);
		assert_eq!(0, problem.jobs[0].latest_start);
		assert_eq!(7, problem.jobs[2].latest_start);
		assert_eq!(18, problem.jobs[1].latest_start);

		assert_eq!(4, problem.jobs[0].earliest_start);
		assert_eq!(11, problem.jobs[2].earliest_start); // 4 + 2 + 5
		assert_eq!(22, problem.jobs[1].earliest_start); // 11 + 9 + 2

		assert!(problem.is_certainly_infeasible());
	}

	#[test]
	fn test_simple_mixed_feasible_chain() {
		let jobs_file = "./test-problems/feasible/1core/case2.csv";
		let constraints_file = "./test-problems/feasible/1core/case2.prec.csv";
		let mut problem = parse_problem(jobs_file, Some(constraints_file), 123);
		let permutation = ProblemPermutation::possible(&mut problem).unwrap();
		assert!(strengthen_bounds_using_constraints(&mut problem));
		permutation.transform_back(&mut problem);
		assert_eq!(5, problem.jobs[0].latest_start);
		assert_eq!(10, problem.jobs[1].latest_start);
		assert_eq!(10, problem.jobs[2].latest_start);

		assert_eq!(5, problem.jobs[0].earliest_start);
		assert_eq!(10, problem.jobs[1].earliest_start);
		assert_eq!(9, problem.jobs[2].earliest_start);

		assert!(!problem.is_certainly_infeasible());
	}

	#[test]
	fn test_simple_mixed_infeasible_chain() {
		let jobs_file = "./test-problems/infeasible/difficulty1/case2-1cores.csv";
		let constraints_file = "./test-problems/infeasible/difficulty1/case2.prec.csv";
		let mut problem = parse_problem(jobs_file, Some(constraints_file), 123);
		let permutation = ProblemPermutation::possible(&mut problem).unwrap();
		assert!(strengthen_bounds_using_constraints(&mut problem));
		permutation.transform_back(&mut problem);
		assert_eq!(3, problem.jobs[0].latest_start);
		assert_eq!(10, problem.jobs[1].latest_start);
		assert_eq!(10, problem.jobs[2].latest_start);

		assert_eq!(5, problem.jobs[0].earliest_start);
		assert_eq!(12, problem.jobs[1].earliest_start);
		assert_eq!(9, problem.jobs[2].earliest_start);

		assert!(problem.is_certainly_infeasible());
	}
}
