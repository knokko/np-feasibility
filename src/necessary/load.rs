use std::collections::HashSet;
use crate::problem::*;
use crate::sorted_job_iterator::SortedJobIterator;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum LoadResult {
	Finished,
	Running,
	CertainlyInfeasible,
}

#[derive(Debug)]
struct LoadJob {
	job: usize,

	/// An upper bound on the time until this job is finished
	maximum_remaining_time: Time
}

impl LoadJob {
	fn get_minimum_spent_time(&self, execution_time: Time) -> Time {
		execution_time - self.maximum_remaining_time
	}
}

struct LoadTest<'a> {
	problem: &'a Problem,
	jobs_by_earliest_start: SortedJobIterator,
	jobs_by_latest_start: SortedJobIterator,

	times_of_interest: Vec<Time>,
	current_time: Time,
	time_index: usize,

	certainly_finished_jobs_load: Time,
	minimum_executed_load: Time,
	maximum_executed_load: Time,

	possibly_running_jobs: Vec<LoadJob>,
	certainly_started_jobs: Vec<LoadJob>,
}

impl<'a> LoadTest<'a> {
	fn new(problem: &'a Problem) -> Self {
		let jobs_by_earliest_start = SortedJobIterator::new(&problem.jobs, |j| j.earliest_start);
		let jobs_by_latest_start = SortedJobIterator::new(&problem.jobs, |j| j.latest_start);
		let mut times_of_interest: HashSet<Time> = HashSet::with_capacity(2 * problem.jobs.len());
		for job in &problem.jobs {
			times_of_interest.insert(job.latest_start);
			times_of_interest.insert(job.get_latest_finish());
		}
		times_of_interest.remove(&0);
		let mut sorted_times_of_interest = times_of_interest.into_iter().collect::<Vec<_>>();
		sorted_times_of_interest.sort();
		LoadTest {
			problem, jobs_by_earliest_start, jobs_by_latest_start,
			times_of_interest: sorted_times_of_interest,
			current_time: 0, time_index: 0,
			certainly_finished_jobs_load: 0,
			minimum_executed_load: 0,
			maximum_executed_load: 0,
			possibly_running_jobs: Vec::new(),
			certainly_started_jobs: Vec::new(),
		}
	}

	fn next(&mut self) -> LoadResult {
		let next_time = self.times_of_interest[self.time_index];
		self.time_index += 1;
		let spent_time = next_time - self.current_time;

		let mut earliest_step_arrival = next_time;
		let earliest_possibly_running_job = self.possibly_running_jobs.iter().min_by_key(
			|j| self.problem.jobs[j.job].earliest_start
		);
		if let Some(earliest) = earliest_possibly_running_job {
			earliest_step_arrival = Time::min(
				earliest_step_arrival, self.problem.jobs[earliest.job].earliest_start
			);
		}

		let mut maximum_load_this_step = 0;
		self.possibly_running_jobs.retain_mut(|running_job| {
			if running_job.maximum_remaining_time > spent_time {
				maximum_load_this_step += spent_time;
				running_job.maximum_remaining_time -= spent_time;
				true
			} else {
				self.certainly_finished_jobs_load += self.problem.jobs[running_job.job].get_execution_time();
				maximum_load_this_step += running_job.maximum_remaining_time;
				false
			}
		});

		while let Some(early_index) = self.jobs_by_earliest_start.next(|time| time <= next_time) {
			let early_job = &self.problem.jobs[early_index];
			if early_job.get_latest_finish() > next_time {
				self.possibly_running_jobs.push(LoadJob {
					job: early_index,
					maximum_remaining_time: early_job.get_latest_finish() - next_time,
				});
				maximum_load_this_step += Time::min(early_job.get_execution_time(), next_time - early_job.earliest_start);
			} else {
				self.certainly_finished_jobs_load += early_job.get_execution_time();
				maximum_load_this_step += early_job.get_execution_time();
				earliest_step_arrival = Time::min(earliest_step_arrival, early_job.earliest_start);
			}
		}

		self.certainly_started_jobs.retain_mut(|started| {
			if started.maximum_remaining_time > spent_time {
				started.maximum_remaining_time = self.problem.jobs[started.job].get_latest_finish() - next_time;
				true
			} else {
				false
			}
		});

		while let Some(late_index) = self.jobs_by_latest_start.next(|time| time <= next_time) {
			let late_job = &self.problem.jobs[late_index];
			if late_job.get_latest_finish() > next_time {
				self.certainly_started_jobs.push(LoadJob {
					job: late_index,
					maximum_remaining_time: late_job.get_latest_finish() - next_time,
				});
			}
		}

		// Minimize (sum worst_case_exec_time() of finished jobs) + (sum minimum_spent_time() of unfinished jobs)
		self.certainly_started_jobs.sort_by_key(|j| j.maximum_remaining_time);
		self.minimum_executed_load = self.certainly_finished_jobs_load;
		let mut start_index = 0;

		// Since all these jobs must have started already, at least num_started_jobs - num_cores must have finished already
		let num_cores = self.problem.num_cores as usize;
		if num_cores < self.certainly_started_jobs.len() {
			while start_index < self.certainly_started_jobs.len() - num_cores {
				let job = self.problem.jobs[self.certainly_started_jobs[start_index].job];
				self.minimum_executed_load += job.get_execution_time();
				start_index += 1;
			}
		}

		while start_index < self.certainly_started_jobs.len() {
			let job = self.problem.jobs[self.certainly_started_jobs[start_index].job];
			self.minimum_executed_load += self.certainly_started_jobs[start_index].get_minimum_spent_time(job.get_execution_time());
			start_index += 1;
		}

		let mut max_load_bound2 = self.certainly_finished_jobs_load;
		for running_job in &self.possibly_running_jobs {
			let job = self.problem.jobs[running_job.job];
			max_load_bound2 += job.get_execution_time();
			earliest_step_arrival = Time::min(earliest_step_arrival, job.earliest_start);
		}

		earliest_step_arrival = Time::max(earliest_step_arrival, self.current_time);
		self.maximum_executed_load += Time::min(
			num_cores as Time * (next_time - earliest_step_arrival), maximum_load_this_step
		);
		self.maximum_executed_load = Time::min(self.maximum_executed_load, max_load_bound2);
		self.current_time = next_time;

		if self.minimum_executed_load > self.maximum_executed_load {
			LoadResult::CertainlyInfeasible
		} else if self.time_index < self.times_of_interest.len() {
			LoadResult::Running
		} else {
			LoadResult::Finished
		}
	}
}

/// Runs the Feasibility Load Test and returns `true` if `problem` is certainly infeasible. When
/// this function returns `false`, `problem` may or may not be feasible.
///
/// The Feasibility Load Test works by creating a set of potentially interesting intervals of time.
/// During each interval, it computes the minimum amount of time that must be spent on executing
/// jobs (assuming no deadlines are missed), as well as the maximum amount of time that can
/// possibly be spent on executing jobs.
///
/// If the minimum amount of time spent in any interval is larger than the maximum amount of time
/// spent in that interval, `problem` is certainly infeasible.
pub fn run_feasibility_load_test(problem: &Problem) -> bool {
	let mut load_test = LoadTest::new(problem);
	loop {
		let result = load_test.next();
		if result == LoadResult::CertainlyInfeasible {
			return true;
		}
		if result == LoadResult::Finished {
			return false;
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::bounds::*;
	use super::*;

	#[test]
	fn test_feasibility_load_with_1_job_variant1() {
		let problem = Problem {
			jobs: vec![Job::release_to_deadline(0, 0, 1000, 1000)],
			constraints: vec![],
			num_cores: 1,
		};
		let mut load_test = LoadTest::new(&problem);
		assert_eq!(load_test.next(), LoadResult::Finished);
		assert_eq!(load_test.current_time, 1000);
		assert_eq!(load_test.minimum_executed_load, 1000);
		assert_eq!(load_test.maximum_executed_load, 1000);

		assert!(!run_feasibility_load_test(&problem));
	}

	#[test]
	fn test_feasibility_load_with_1_job_variant2() {
		let problem = Problem {
			jobs: vec![Job::release_to_deadline(0, 0, 999, 1000)],
			constraints: vec![],
			num_cores: 1,
		};
		let mut load_test = LoadTest::new(&problem);
		assert_eq!(load_test.next(), LoadResult::Running);
		assert_eq!(load_test.current_time, 1);
		assert_eq!(load_test.minimum_executed_load, 0);
		assert_eq!(load_test.maximum_executed_load, 1);

		assert_eq!(load_test.next(), LoadResult::Finished);
		assert_eq!(load_test.current_time, 1000);
		assert_eq!(load_test.minimum_executed_load, 999);
		assert_eq!(load_test.maximum_executed_load, 999);

		assert!(!run_feasibility_load_test(&problem));
	}

	#[test]
	fn test_feasibility_load_with_1_job_variant3() {
		let problem = Problem {
			jobs: vec![Job::release_to_deadline(0, 0, 1001, 1000)],
			constraints: vec![],
			num_cores: 1,
		};
		assert!(run_feasibility_load_test(&problem));
	}

	#[test]
	fn test_tight_feasible_case_arriving_at0() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 0, 5, 16),
				Job::release_to_deadline(1, 0, 3, 10),
				Job::release_to_deadline(2, 0, 8, 11),
			],
			constraints: vec![],
			num_cores: 1,
		};

		let mut load_test = LoadTest::new(&problem);
		assert_eq!(load_test.next(), LoadResult::Running);
		assert_eq!(load_test.current_time, 3);
		assert_eq!(load_test.minimum_executed_load, 0);
		assert_eq!(load_test.maximum_executed_load, 3);

		assert_eq!(load_test.next(), LoadResult::Running);
		assert_eq!(load_test.current_time, 7);
		assert_eq!(load_test.minimum_executed_load, 7);
		assert_eq!(load_test.maximum_executed_load, 7);

		assert_eq!(load_test.next(), LoadResult::Running);
		assert_eq!(load_test.current_time, 10);
		assert_eq!(load_test.minimum_executed_load, 10);
		assert_eq!(load_test.maximum_executed_load, 10);

		assert_eq!(load_test.next(), LoadResult::Running);
		assert_eq!(load_test.current_time, 11);
		assert_eq!(load_test.minimum_executed_load, 11);
		assert_eq!(load_test.maximum_executed_load, 11);

		assert_eq!(load_test.next(), LoadResult::Finished);
		assert_eq!(load_test.current_time, 16);
		assert_eq!(load_test.minimum_executed_load, 16);
		assert_eq!(load_test.maximum_executed_load, 16);

		assert!(!run_feasibility_load_test(&problem));
	}

	#[test]
	fn test_tight_infeasible_case_arriving_at0() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 0, 5, 16),
				Job::release_to_deadline(1, 0, 3, 10),
				Job::release_to_deadline(2, 0, 8, 10),
			],
			constraints: vec![],
			num_cores: 1,
		};

		let mut load_test = LoadTest::new(&problem);
		assert_eq!(load_test.next(), LoadResult::Running);
		assert_eq!(load_test.current_time, 2);
		assert_eq!(load_test.minimum_executed_load, 0);
		assert_eq!(load_test.maximum_executed_load, 2);

		assert_eq!(load_test.next(), LoadResult::CertainlyInfeasible);
		assert_eq!(load_test.current_time, 7);
		assert_eq!(load_test.minimum_executed_load, 8);
		assert_eq!(load_test.maximum_executed_load, 7);

		assert!(run_feasibility_load_test(&problem));
	}

	#[test]
	fn test_feasible_when_longest_job_first() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 3, 6, 18),
				Job::release_to_deadline(1, 4, 5, 19)
			],
			constraints: vec![],
			num_cores: 1
		};

		let mut load_test = LoadTest::new(&problem);
		assert_eq!(load_test.next(), LoadResult::Running);
		assert_eq!(load_test.current_time, 12);
		assert_eq!(load_test.minimum_executed_load, 0);
		assert_eq!(load_test.maximum_executed_load, 9);

		assert_eq!(load_test.next(), LoadResult::Running);
		assert_eq!(load_test.current_time, 14);
		assert_eq!(load_test.minimum_executed_load, 6);
		assert_eq!(load_test.maximum_executed_load, 11);

		assert_eq!(load_test.next(), LoadResult::Running);
		assert_eq!(load_test.current_time, 18);
		assert_eq!(load_test.minimum_executed_load, 10);
		assert_eq!(load_test.maximum_executed_load, 11);

		assert_eq!(load_test.next(), LoadResult::Finished);
		assert_eq!(load_test.current_time, 19);
		assert_eq!(load_test.minimum_executed_load, 11);
		assert_eq!(load_test.maximum_executed_load, 11);

		assert!(!run_feasibility_load_test(&problem));
	}

	#[test]
	fn test_feasible_when_shortest_job_first() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 3, 6, 18),
				Job::release_to_deadline(1, 4, 7, 20)
			],
			constraints: vec![],
			num_cores: 1
		};

		let mut load_test = LoadTest::new(&problem);
		assert_eq!(load_test.next(), LoadResult::Running);
		assert_eq!(load_test.current_time, 12);
		assert_eq!(load_test.minimum_executed_load, 0);
		assert_eq!(load_test.maximum_executed_load, 9);

		assert_eq!(load_test.next(), LoadResult::Running);
		assert_eq!(load_test.current_time, 13);
		assert_eq!(load_test.minimum_executed_load, 6);
		assert_eq!(load_test.maximum_executed_load, 10);

		assert_eq!(load_test.next(), LoadResult::Running);
		assert_eq!(load_test.current_time, 18);
		assert_eq!(load_test.minimum_executed_load, 11);
		assert_eq!(load_test.maximum_executed_load, 13);

		assert_eq!(load_test.next(), LoadResult::Finished);
		assert_eq!(load_test.current_time, 20);
		assert_eq!(load_test.minimum_executed_load, 13);
		assert_eq!(load_test.maximum_executed_load, 13);
	}

	#[test]
	fn test_tight_feasible_with_2_cores_and_more_jobs() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 2, 5, 10),
				Job::release_to_deadline(1, 0, 13, 30),
				Job::release_to_deadline(2, 0, 3, 25),
				Job::release_to_deadline(3, 10, 2, 25),
				Job::release_to_deadline(4, 0, 7, 20),

				Job::release_to_deadline(5, 2, 5, 10),
				Job::release_to_deadline(6, 0, 8, 25),
				Job::release_to_deadline(7, 0, 3, 30),
				Job::release_to_deadline(8, 10, 8, 30),
				Job::release_to_deadline(9, 0, 6, 20)
			],
			constraints: vec![],
			num_cores: 2
		};

		let mut load_test = LoadTest::new(&problem);
		loop {
			let next = load_test.next();
			if next == LoadResult::Finished {
				break;
			}
			assert_eq!(next, LoadResult::Running);
		}
		assert_eq!(load_test.current_time, 30);
		assert_eq!(load_test.minimum_executed_load, 60);
		assert_eq!(load_test.maximum_executed_load, 60);
	}

	#[test]
	fn test_tight_infeasible_with_2_cores_and_more_jobs() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 2, 5, 10),
				Job::release_to_deadline(1, 0, 13, 30),
				Job::release_to_deadline(2, 0, 3, 25),
				Job::release_to_deadline(3, 10, 2, 25),
				Job::release_to_deadline(4, 0, 7, 20),

				Job::release_to_deadline(5, 2, 5, 10),

				// Job 6 takes 1 time unit longer than in the previous test
				Job::release_to_deadline(6, 0, 9, 25),
				Job::release_to_deadline(7, 0, 3, 30),
				Job::release_to_deadline(8, 10, 8, 30),
				Job::release_to_deadline(9, 0, 6, 20)
			],
			constraints: vec![],
			num_cores: 2
		};

		assert!(run_feasibility_load_test(&problem));
	}

	#[test]
	fn test_almost_infeasible_early_load() {
		let mut problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 1, 3, 10),
				Job::release_to_deadline(1, 1, 3, 10),
				Job::release_to_deadline(2, 1, 3, 10),

				Job::release_to_deadline(3, 8, 5, 20),

				Job::release_to_deadline(4, 30, 5, 40),
			],
			constraints: vec![],
			num_cores: 1
		};

		assert!(!run_feasibility_load_test(&problem));
		assert_ne!(OccupationStrengthenResult::Infeasible, strengthen_bounds_using_core_occupation(&mut problem));
	}

	#[test]
	fn test_infeasible_early_overload() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 1, 3, 10),
				Job::release_to_deadline(1, 1, 4, 10),

				// Job 2 takes 1 time unit longer than in the previous test
				Job::release_to_deadline(2, 1, 3, 10),

				Job::release_to_deadline(3, 8, 5, 20),

				Job::release_to_deadline(4, 30, 5, 40),
			],
			constraints: vec![],
			num_cores: 1
		};

		assert!(run_feasibility_load_test(&problem));
	}

	#[test]
	fn test_infeasible_early_overload_with_irrelevant_background_job() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 1, 3, 10),
				Job::release_to_deadline(1, 1, 4, 10),

				// Job 2 takes 1 time unit longer than in the previous test
				Job::release_to_deadline(2, 1, 3, 10),

				Job::release_to_deadline(3, 8, 5, 20),

				Job::release_to_deadline(4, 30, 5, 40),

				Job::release_to_deadline(5, 0, 50, 100),
			],
			constraints: vec![],
			num_cores: 1
		};

		// TODO For the feasibility interval test
	}

	#[test]
	fn test_almost_infeasible_middle_load() {
		let mut problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 1, 3, 10),
				Job::release_to_deadline(1, 1, 3, 10),

				Job::release_to_deadline(2, 12, 3, 20),
				Job::release_to_deadline(3, 12, 5, 20),

				Job::release_to_deadline(4, 30, 5, 40),
			],
			constraints: vec![],
			num_cores: 1
		};
		assert!(!run_feasibility_load_test(&problem));
		assert_ne!(OccupationStrengthenResult::Infeasible, strengthen_bounds_using_core_occupation(&mut problem));
	}

	fn middle_overload_jobs() -> Vec<Job> {
		vec![
			Job::release_to_deadline(0, 1, 3, 10),
			Job::release_to_deadline(1, 1, 3, 10),

			// Job 2 takes 1 time unit longer than in the previous test
			Job::release_to_deadline(2, 12, 4, 20),
			Job::release_to_deadline(3, 12, 5, 20),

			Job::release_to_deadline(4, 30, 5, 40),
		]
	}

	#[test]
	fn test_infeasible_middle_overload() {
		let problem = Problem {
			jobs: middle_overload_jobs(),
			constraints: vec![],
			num_cores: 1
		};
		assert!(run_feasibility_load_test(&problem));
	}

	#[test]
	fn test_infeasible_middle_overload_with_irrelevant_background_job() {
		let mut problem = Problem {
			jobs: middle_overload_jobs(),
			constraints: vec![],
			num_cores: 1
		};
		problem.jobs.push(Job::release_to_deadline(4, 30, 5, 40));
		assert_eq!(OccupationStrengthenResult::Infeasible, strengthen_bounds_using_core_occupation(&mut problem));
		// TODO Solve with feasibility interval test
	}

	#[test]
	fn test_almost_infeasible_late_load() {
		let mut problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 1, 3, 10),
				Job::release_to_deadline(1, 1, 3, 10),

				Job::release_to_deadline(2, 12, 6, 20),

				Job::release_to_deadline(3, 30, 5, 40),
				Job::release_to_deadline(4, 30, 5, 40),
			],
			constraints: vec![],
			num_cores: 1
		};
		assert!(!run_feasibility_load_test(&problem));
		assert_ne!(OccupationStrengthenResult::Infeasible, strengthen_bounds_using_core_occupation(&mut problem));
	}

	#[test]
	fn test_infeasible_late_overload() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 1, 3, 10),
				Job::release_to_deadline(1, 1, 3, 10),

				Job::release_to_deadline(2, 12, 6, 20),

				// Job 3 takes 1 time unit longer than in the previous test
				Job::release_to_deadline(3, 30, 6, 40),
				Job::release_to_deadline(4, 30, 5, 40),
			],
			constraints: vec![],
			num_cores: 1
		};
		assert!(run_feasibility_load_test(&problem));
	}

	#[test]
	fn test_infeasible_late_overload_with_irrelevant_background_job() {
		let mut problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 1, 3, 10),
				Job::release_to_deadline(1, 1, 3, 10),

				Job::release_to_deadline(2, 12, 6, 20),

				// Job 3 takes 1 time unit longer than in the previous test
				Job::release_to_deadline(3, 30, 6, 40),
				Job::release_to_deadline(4, 30, 5, 40),

				Job::release_to_deadline(5, 0, 50, 100),
			],
			constraints: vec![],
			num_cores: 1
		};
		assert_eq!(OccupationStrengthenResult::Infeasible, strengthen_bounds_using_core_occupation(&mut problem));
		// TODO Use feasibility interval test
	}

	#[test]
	fn feasibility_interval_regression_test() {
		let mut problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 0, 98, 100),
				Job::release_to_deadline(1, 38, 16, 88),
				Job::release_to_deadline(2, 0, 48, 65),
				Job::release_to_deadline(3, 60, 34, 100),
			],
			constraints: vec![],
			num_cores: 2
		};

		assert!(!run_feasibility_load_test(&problem));
		assert_ne!(OccupationStrengthenResult::Infeasible, strengthen_bounds_using_core_occupation(&mut problem));
		assert!(!run_feasibility_load_test(&problem));
		// TODO run interval test
	}

	#[test]
	fn feasibility_interval_regression_suboptimal() {
		let mut problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 23, 68, 100),
				Job::release_to_deadline(1, 10, 78, 100),
				Job::release_to_deadline(2, 0, 18, 20),
				Job::release_to_deadline(3, 0, 34, 38),
			],
			constraints: vec![],
			num_cores: 2
		};

		assert!(!run_feasibility_load_test(&problem));
		// TODO Interval test should detect this!
		assert_eq!(OccupationStrengthenResult::Infeasible, strengthen_bounds_using_core_occupation(&mut problem));
	}
}
