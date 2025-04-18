use crate::problem::*;
use std::collections::HashMap;
use std::fs::read_to_string;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct SagJobID {
	task_id: u32,
	job_id: u32,
}

fn parse_jobs(file_path: &str) -> (Vec<Job>, HashMap<SagJobID, usize>) {
	let raw_text = read_to_string(file_path).expect("Couldn't read jobs file");

	let mut jobs = Vec::<Job>::new();
	let mut id_map = HashMap::<SagJobID, usize>::new();

	let mut allow_header = true;

	for line in raw_text.lines() {
		if line.trim().is_empty() { continue; }
		if allow_header {
			allow_header = false;
			if line.chars().any(|c| c.is_alphabetic()) { continue; }
		}
		let string_values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

		let latest_arrival: Time;
		let worst_case_execution_time: Time;
		let deadline: Time;

		if string_values.len() == 8 {
			let task_id = string_values[0].parse::<u32>().expect("Couldn't parse task ID");
			let job_id = string_values[1].parse::<u32>().expect("Couldn't parse job ID");
			latest_arrival = string_values[3].parse::<Time>()
				.expect("Couldn't parse latest arrival time");
			worst_case_execution_time = string_values[5].parse::<Time>()
				.expect("Couldn't parse worst-case execution time");
			deadline = string_values[6].parse::<Time>().expect("Couldn't parse deadline");
			id_map.insert(SagJobID { task_id, job_id }, jobs.len());
		} else if string_values.len() == 3 {
			latest_arrival = string_values[0].parse::<Time>()
				.expect("Couldn't parse latest arrival time");
			worst_case_execution_time = string_values[1].parse::<Time>()
				.expect("Couldn't parse worst-case execution time");
			deadline = string_values[2].parse::<Time>().expect("Couldn't parse deadline");
		} else {
			panic!("Unexpected line in jobs file: {}", line);
		}

		jobs.push(Job::release_to_deadline(jobs.len(), latest_arrival, worst_case_execution_time, deadline));
	}

	(jobs, id_map)
}

fn parse_constraints(file_path: &str, id_map: &HashMap<SagJobID, usize>) -> Vec<Constraint> {
	let raw_text = read_to_string(file_path).expect("Couldn't read jobs file");
	let mut constraints = Vec::<Constraint>::new();

	let mut allow_header = true;

	for line in raw_text.lines() {
		if line.trim().is_empty() { continue; }
		if allow_header {
			allow_header = false;
			if line.chars().any(|c| c != 's' && c != 'f' && c.is_alphabetic()) { continue; }
		}
		let string_values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

		if string_values.len() < 4 || (string_values.len() == 4 && string_values[3].chars().any(|c| c == 's' || c == 'f')) {
			let before = string_values[0].parse::<usize>()
				.expect("Couldn't parse the index of the 'before' job of a constraint");
			let after = string_values[1].parse::<usize>()
				.expect("Couldn't parse the index of the 'after' job of a constraint");

			let mut delay = 0;
			if string_values.len() >= 3 {
				delay = string_values[2].parse::<Time>()
					.expect("Couldn't parse the delay of a constraint");
			}

			let mut constraint_type = ConstraintType::FinishToStart;
			if string_values.len() >= 4 {
				if string_values[3] == "f-s" {
					constraint_type = ConstraintType::FinishToStart;
				} else if string_values[3] == "s-s" {
					constraint_type = ConstraintType::StartToStart;
				} else {
					panic!("Unexpected constraint type {} in line in constraint file: {}", string_values[3], line);
				}
			}

			constraints.push(Constraint::new(before, after, delay, constraint_type));
		} else {
			let before_task = string_values[0].parse::<u32>()
				.expect("Couldn't parse the task ID of the 'before' job of a constraint");
			let before_job = string_values[1].parse::<u32>()
				.expect("Couldn't parse the job ID of the 'before' job of a constraint");
			let before = id_map[&SagJobID { task_id: before_task, job_id: before_job }];

			let after_task = string_values[2].parse::<u32>()
				.expect("Couldn't parse the task ID of the 'after' job of a constraint");
			let after_job = string_values[3].parse::<u32>()
				.expect("Couldn't parse the job ID of the 'after' job of a constraint");
			let after = id_map[&SagJobID { task_id: after_task, job_id: after_job }];

			let mut delay = 0;
			if string_values.len() >= 6 {
				delay = string_values[5].parse::<Time>()
					.expect("Couldn't parse the delay of a constraint");
			}

			let mut constraint_type = ConstraintType::FinishToStart;
			if string_values.len() >= 7 {
				if string_values[6] == "f-s" {
					constraint_type = ConstraintType::FinishToStart;
				} else if string_values[6] == "s-s" {
					constraint_type = ConstraintType::StartToStart;
				} else {
					panic!("Unexpected constraint type {} in line in constraint file: {}", string_values[6], line);
				}
			}

			constraints.push(Constraint::new(before, after, delay, constraint_type));
		}
	}

	constraints
}

pub fn parse_problem(
	jobs_file_path: &str, constraints_file_path: Option<&str>, num_cores: u32
) -> Problem {
	let (jobs, id_map) = parse_jobs(jobs_file_path);
	if let Some(constraints_path) = constraints_file_path {
		let constraints = parse_constraints(constraints_path, &id_map);
		Problem { jobs, constraints, num_cores }
	} else {
		Problem { jobs, constraints: Vec::new(), num_cores }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_jobs_classic() {
		let (jobs, id_map) = parse_jobs(
			"./test-problems/infeasible/difficulty0/case1-cores1.csv"
		);
		assert_eq!(jobs.len(), 3);
		assert_eq!(id_map.len(), 3);

		assert_eq!(id_map[&SagJobID { task_id: 1, job_id: 1 }], 0);
		assert_eq!(id_map[&SagJobID { task_id: 1, job_id: 2 }], 1);
		assert_eq!(id_map[&SagJobID { task_id: 1, job_id: 3 }], 2);

		assert_eq!(Job::release_to_deadline(0, 40, 10, 100), jobs[0]);
		assert_eq!(Job::release_to_deadline(1, 0, 20, 100), jobs[1]);
		assert_eq!(Job::release_to_deadline(2, 75, 30, 100), jobs[2]);
	}

	#[test]
	fn test_parse_jobs_short() {
		let (jobs, id_map) = parse_jobs(
			"./test-problems/infeasible/cyclic/self-short.csv"
		);
		assert_eq!(jobs, vec![Job::release_to_deadline(0, 500, 209, 2000)]);
		assert_eq!(id_map.len(), 0);
	}

	#[test]
	fn test_parse_constraints_classic4() {
		let (_jobs, id_map) = parse_jobs(
			"./test-problems/infeasible/cyclic/self-classic.csv"
		);
		let constraints = parse_constraints(
			"./test-problems/infeasible/cyclic/self-classic4.prec.csv", &id_map
		);
		assert_eq!(vec![Constraint::new(0, 0, 0, ConstraintType::FinishToStart)], constraints);
	}

	#[test]
	fn test_parse_constraints_classic6() {
		let (_jobs, id_map) = parse_jobs(
			"./test-problems/infeasible/cyclic/self-classic.csv"
		);
		let constraints = parse_constraints(
			"./test-problems/infeasible/cyclic/self-classic6.prec.csv", &id_map
		);
		assert_eq!(vec![Constraint::new(0, 0, 5, ConstraintType::FinishToStart)], constraints);
	}

	#[test]
	fn test_parse_constraints_classic7() {
		let (_jobs, id_map) = parse_jobs(
			"./test-problems/infeasible/cyclic/self-classic.csv"
		);
		let constraints = parse_constraints(
			"./test-problems/infeasible/cyclic/self-classic7.prec.csv", &id_map
		);
		assert_eq!(vec![Constraint::new(0, 0, 5, ConstraintType::StartToStart)], constraints);
	}

	#[test]
	fn test_parse_constraints_short2() {
		let (_jobs, id_map) = parse_jobs(
			"./test-problems/infeasible/cyclic/self-classic.csv"
		);
		let constraints = parse_constraints(
			"./test-problems/infeasible/cyclic/self-short2.prec.csv", &id_map
		);
		assert_eq!(vec![Constraint::new(0, 0, 0, ConstraintType::FinishToStart)], constraints);
	}

	#[test]
	fn test_parse_constraints_short3() {
		let (_jobs, id_map) = parse_jobs(
			"./test-problems/infeasible/cyclic/self-classic.csv"
		);
		let constraints = parse_constraints(
			"./test-problems/infeasible/cyclic/self-short3.prec.csv", &id_map
		);
		assert_eq!(vec![Constraint::new(0, 0, 123, ConstraintType::FinishToStart)], constraints);
	}

	#[test]
	fn test_parse_constraints_short4() {
		let (_jobs, id_map) = parse_jobs(
			"./test-problems/infeasible/cyclic/self-classic.csv"
		);
		let constraints = parse_constraints(
			"./test-problems/infeasible/cyclic/self-short4.prec.csv", &id_map
		);
		assert_eq!(vec![Constraint::new(0, 0, 123, ConstraintType::StartToStart)], constraints);
	}

	#[test]
	fn test_parse_problem_without_constraints() {
		let jobs_file_path = "./test-problems/infeasible/difficulty0/case1-cores1.csv";
		let problem = parse_problem(jobs_file_path, None, 1);
		assert_eq!(Problem { jobs: parse_jobs(jobs_file_path).0, constraints: Vec::new(), num_cores: 1 }, problem);
	}

	#[test]
	fn test_parse_classic_problem() {
		let jobs_file_path = "./test-problems/infeasible/cyclic/self-classic.csv";
		let constraints_file_path = "./test-problems/infeasible/cyclic/self-classic6.prec.csv";
		let problem = parse_problem(jobs_file_path, Some(constraints_file_path), 12);
		assert_eq!(Problem {
			jobs: parse_jobs(jobs_file_path).0,
			constraints: vec![Constraint::new(0, 0, 5, ConstraintType::FinishToStart)],
			num_cores: 12
		}, problem);
	}

	#[test]
	fn test_parse_short_problem() {
		let jobs_file_path = "./test-problems/infeasible/cyclic/self-short.csv";
		let constraints_file_path = "./test-problems/infeasible/cyclic/self-short3.prec.csv";
		let problem = parse_problem(jobs_file_path, Some(constraints_file_path), 3);
		assert_eq!(Problem {
			jobs: parse_jobs(jobs_file_path).0,
			constraints: vec![Constraint::new(0, 0, 123, ConstraintType::FinishToStart)],
			num_cores: 3
		}, problem);
	}
}
