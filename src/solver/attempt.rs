use crate::simulator::Simulator;
use crate::solver::FatProblem;
use crate::solver::forced::ForcedJobTracker;
use crate::solver::queue::HeuristicJobQueue;
use crate::solver::job_ordering::JobOrdering;
use crate::solver::skip_distribution::SkipDistribution;

#[derive(Debug, Eq, PartialEq)]
pub struct HeuristicResult {
	pub job_ordering: Vec<usize>,
	pub missed_deadline: bool,
}

pub fn heuristic_attempt<S>(
	problem: &FatProblem, heuristic: &JobOrdering, mut skip_distribution: S
) -> HeuristicResult where S : SkipDistribution {
	let mut queue = HeuristicJobQueue::new(problem);
	let mut simulator = Simulator::new(&problem.problem);
	let mut job_ordering = Vec::with_capacity(problem.problem.jobs.len());
	let mut forced_tracker = ForcedJobTracker::new();
	let mut dispatched_jobs = vec![false; problem.problem.jobs.len()];

	while job_ordering.len() < problem.problem.jobs.len() {
		forced_tracker.update(&problem.forced, &dispatched_jobs);
		let next_job = queue.choose_next(
			problem, skip_distribution.next_to_skip(), heuristic, &forced_tracker,
			|job_index| simulator.predict_next_start_time(
				problem.problem.jobs[job_index]
			)
		);
		println!("chose job {}", next_job);
		simulator.schedule(problem.problem.jobs[next_job]);
		dispatched_jobs[next_job] = true;
		job_ordering.push(next_job);
		if simulator.has_missed_deadline() {
			return HeuristicResult { job_ordering, missed_deadline: true };
		}
	}

	HeuristicResult { job_ordering, missed_deadline: false }
}

#[cfg(test)]
mod tests {
	use crate::problem::*;
	use crate::solver::skip_distribution::{ExponentialSkipDistribution, ZeroSkipDistribution};
	use super::*;

	#[test]
	fn test_on_mini_problem() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 10, 5, 15),
				Job::release_to_deadline(1, 0, 8, 20)
			],
			constraints: vec![],
			num_cores: 1
		};
		let earliest_deadline_first = JobOrdering::new(
			&problem, |j1, j2| j1.get_latest_finish().cmp(&j2.get_latest_finish())
		);
		let earliest_start_first = JobOrdering::new(
			&problem, |j1, j2| j1.earliest_start.cmp(&j2.earliest_start)
		);
		let fat = FatProblem::new(problem);

		// This heuristic should work
		assert_eq!(
			HeuristicResult { job_ordering: vec![1, 0], missed_deadline: false },
			heuristic_attempt(&fat, &earliest_start_first, ZeroSkipDistribution)
		);

		// The following heuristic would fail, but the force-job mechanism prevents it
		assert_eq!(
			HeuristicResult { job_ordering: vec![1, 0], missed_deadline: false },
			heuristic_attempt(&fat, &earliest_deadline_first, ZeroSkipDistribution)
		);

		for _counter in 0 .. 100 {
			assert_eq!(
				HeuristicResult { job_ordering: vec![1, 0], missed_deadline: false },
				heuristic_attempt(
					&fat, &earliest_deadline_first,
					ExponentialSkipDistribution::new(0.5)
				)
			);
		}
	}

	#[test]
	fn test_on_slightly_harder_problem() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 10, 5, 15),
				Job::release_to_deadline(1, 0, 8, 29),
				Job::release_to_deadline(2, 1, 8, 30),
			],
			constraints: vec![],
			num_cores: 1
		};
		let earliest_deadline_first = JobOrdering::new(
			&problem, |j1, j2| j1.get_latest_finish().cmp(&j2.get_latest_finish())
		);
		let earliest_start_first = JobOrdering::new(
			&problem, |j1, j2| j1.earliest_start.cmp(&j2.earliest_start)
		);
		let fat = FatProblem::new(problem);

		// This heuristic should work
		assert_eq!(
			HeuristicResult { job_ordering: vec![1, 2, 0], missed_deadline: false },
			heuristic_attempt(&fat, &earliest_start_first, ZeroSkipDistribution)
		);

		// This heuristic does not
		assert_eq!(
			HeuristicResult { job_ordering: vec![0, 1, 2], missed_deadline: true },
			heuristic_attempt(&fat, &earliest_deadline_first, ZeroSkipDistribution)
		);

		// This should work if, and only if, job 0 is skipped (75% chance
		let mut missed_deadlines = 0;
		for _counter in 0 .. 10_000 {
			let result = heuristic_attempt(
				&fat, &earliest_deadline_first,
				ExponentialSkipDistribution::new(0.75)
			);
			if result.missed_deadline {
				missed_deadlines += 1;
			}
		}

		assert!(missed_deadlines > 1500);
		assert!(missed_deadlines < 3500);
	}

	#[test]
	fn test_on_small_hard_problem() {
		// let problem = Problem {
		// 	jobs: vec![
		// 		Job::release_to_deadline(0, 0, 5, 12)
		// 	]
		// }
	}
}
