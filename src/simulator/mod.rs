mod core_availability;

use crate::problem::*;
use crate::simulator::core_availability::CoreAvailability;

fn create_predecessor_mapping(problem: &Problem) -> (Vec<Vec<Constraint>>, Time) {
	let mut maximum_suspension = 0;
	let mut mapping = vec![Vec::<Constraint>::new(); problem.jobs.len()];
	for constraint in &problem.constraints {
		mapping[constraint.get_after()].push(*constraint);
		maximum_suspension = Time::max(maximum_suspension, constraint.get_delay());
	}
	(mapping, maximum_suspension)
}

#[derive(Clone, Copy)]
struct RunningJob {
	job: usize,
	started_at: Time,
	finishes_at: Time,
}

/// This struct simulates a non-preemptive work-conserving scheduler. Users can invoke the
/// `schedule(Job)` method to dispatch jobs, and this simulator checks whether a non-preemptive
/// work-conserving scheduler would miss deadlines if the jobs were dispatched in that order.
///
/// This struct also has methods like `predict_start_time(Job)` to predict at what time a job would
/// start if it were dispatched next, *without* actually scheduling that job.
#[derive(Clone)]
pub struct Simulator {
	finished_jobs: Vec<bool>, // TODO Create IndexSet struct for this
	running_jobs: Vec<RunningJob>,
	core_availability: CoreAvailability,
	predecessor_mapping: Vec<Vec<Constraint>>,
	maximum_suspension: Time,
	num_finished_jobs: usize,
	missed_deadline: bool,
}

impl Simulator {

	/// Constructs a new simulator for the given problem.
	/// No jobs are dispatched initially.
	pub fn new(problem: &Problem) -> Self {
		let (predecessor_mapping, maximum_suspension) = create_predecessor_mapping(problem);
		Self {
			finished_jobs: vec![false; problem.jobs.len()],
			running_jobs: Vec::new(),
			core_availability: CoreAvailability::new(problem.num_cores as usize),
			predecessor_mapping,
			maximum_suspension,
			num_finished_jobs: 0,
			missed_deadline: false,
		}
	}

	/// Assuming that `job` is the next job that is dispatched, predicts at which time it would
	/// start executing. This method does **not** schedule `job`: it only provides information.
	pub fn predict_start_time(&self, job: Job) -> Time {
		let mut ready_time = job.earliest_start;
		for constraint in &self.predecessor_mapping[job.get_index()] {
			if self.finished_jobs[constraint.get_before()] {
				continue;
			}
			let running_job = self.running_jobs.iter().find(
				|rj| rj.job == constraint.get_before()
			).expect("All predecessors should have started already");
			let mut ready_bound = constraint.get_delay();
			if constraint.get_type() == ConstraintType::FinishToStart {
				ready_bound += running_job.finishes_at;
			} else {
				ready_bound += running_job.started_at;
			}
			ready_time = Time::max(ready_time, ready_bound);
		}

		Time::max(ready_time, self.core_availability.next_start_time())
	}

	/// Assuming that `job` is the next job that is dispatched, this method predicts the earliest
	/// time at or after the start of `job` where at least 1 core is available. The core
	/// that executes `job` is not counted 'available' at the start time of `job`.
	pub fn predict_next_start_time(&self, job: Job) -> Time {
		let current_start_time = self.predict_start_time(job);
		let mut next_start_time = current_start_time + job.get_execution_time();
		if self.core_availability.get_num_cores() > 1 {
			next_start_time = Time::min(next_start_time, self.core_availability.second_start_time());
		}
		Time::max(current_start_time, next_start_time)
	}

	/// Ensures that `job` is the next job that starts. It will start as early as possible. The
	/// start time can be predicted using `predict_start_time(job)`.
	pub fn schedule(&mut self, job: Job) {
		let start_time = self.predict_start_time(job);
		if start_time > job.latest_start {
			self.missed_deadline = true;
		}
		debug_assert!(start_time >= job.earliest_start);
		debug_assert!(!self.finished_jobs[job.get_index()]);
		self.core_availability.schedule(start_time, job.get_execution_time());

		let mut index = 0;
		while index < self.running_jobs.len() {
			let running_job = self.running_jobs[index];
			debug_assert!(running_job.job != job.get_index());
			if self.core_availability.next_start_time() >= running_job.finishes_at + self.maximum_suspension {
				debug_assert!(!self.finished_jobs[running_job.job]);
				self.finished_jobs[running_job.job] = true;
				self.num_finished_jobs += 1;
				self.running_jobs.swap_remove(index);
			} else {
				index += 1;
			}
		}

		self.running_jobs.push(RunningJob {
			job: job.get_index(),
			started_at: start_time,
			finishes_at: start_time + job.get_execution_time()
		})
	}

	/// Predicts the earliest next time at which at least 1 core is available.
	pub fn next_core_available(&self) -> Time {
		self.core_availability.next_start_time()
	}

	/// Returns true if and only if a deadline miss was encountered, which is if and only if
	/// a job was `schedule`d after its latest start time.
	pub fn has_missed_deadline(&self) -> bool {
		self.missed_deadline
	}

	/// Returns the number of jobs that have been `schedule`d so far
	pub fn num_dispatched_jobs(&self) -> usize {
		self.num_finished_jobs + self.running_jobs.len()
	}
}

#[cfg(test)]
mod tests {
	use crate::bounds::strengthen_bounds_using_constraints;
	use crate::problem::*;
	use crate::simulator::Simulator;

	#[test]
	fn small_simple_problem_with_one_core() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 0, 20, 50),
				Job::release_to_deadline(1, 10, 30, 50)
			],
			constraints: vec![],
			num_cores: 1
		};
		problem.validate();

		let mut good_simulator = Simulator::new(&problem);
		assert_eq!(0, good_simulator.num_dispatched_jobs());
		good_simulator.schedule(problem.jobs[0]);
		assert_eq!(1, good_simulator.num_dispatched_jobs());
		good_simulator.schedule(problem.jobs[1]);
		assert_eq!(2, good_simulator.num_dispatched_jobs());
		assert!(!good_simulator.has_missed_deadline());

		let mut bad_simulator = Simulator::new(&problem);
		assert_eq!(0, bad_simulator.num_dispatched_jobs());
		bad_simulator.schedule(problem.jobs[1]);
		assert_eq!(1, bad_simulator.num_dispatched_jobs());
		bad_simulator.schedule(problem.jobs[0]);
		assert_eq!(2, bad_simulator.num_dispatched_jobs());
		assert!(bad_simulator.has_missed_deadline());
	}

	#[test]
	fn test_larger_problem_with_one_core() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 0, 2, 10),
				Job::release_to_deadline(1, 10, 2, 20),
				Job::release_to_deadline(2, 20, 2, 30),
				Job::release_to_deadline(3, 30, 2, 40),
				Job::release_to_deadline(4, 40, 2, 50),
				Job::release_to_deadline(5, 50, 2, 60),

				Job::release_to_deadline(6, 0, 8, 30),
				Job::release_to_deadline(7, 30, 8, 60),

				Job::release_to_deadline(8, 0, 13, 60),
			],
			constraints: vec![],
			num_cores: 1
		};
		problem.validate();

		let mut good_simulator = Simulator::new(&problem);
		good_simulator.schedule(problem.jobs[0]);
		good_simulator.schedule(problem.jobs[6]);
		good_simulator.schedule(problem.jobs[1]);
		good_simulator.schedule(problem.jobs[8]);
		good_simulator.schedule(problem.jobs[2]);
		good_simulator.schedule(problem.jobs[3]);
		good_simulator.schedule(problem.jobs[7]);
		good_simulator.schedule(problem.jobs[4]);
		good_simulator.schedule(problem.jobs[5]);
		assert_eq!(9, good_simulator.num_dispatched_jobs());
		assert!(!good_simulator.has_missed_deadline());

		let mut bad_simulator = Simulator::new(&problem);
		bad_simulator.schedule(problem.jobs[0]);
		bad_simulator.schedule(problem.jobs[6]);
		bad_simulator.schedule(problem.jobs[8]);
		bad_simulator.schedule(problem.jobs[1]);
		assert!(bad_simulator.has_missed_deadline());
		assert_eq!(4, bad_simulator.num_dispatched_jobs());
	}

	#[test]
	fn test_with_finish_to_start_constraints() {
		let mut problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 0, 20, 100),
				Job::release_to_deadline(1, 0, 30, 52),
				Job::release_to_deadline(2, 0, 30, 100)
			],
			constraints: vec![
				Constraint::new(0, 1, 2, ConstraintType::FinishToStart)
			],
			num_cores: 1
		};
		problem.validate();
		strengthen_bounds_using_constraints(&mut problem);

		assert!(std::panic::catch_unwind(|| Simulator::new(&problem).schedule(problem.jobs[1])).is_err());

		let mut bad_simulator = Simulator::new(&problem);
		bad_simulator.schedule(problem.jobs[2]);
		assert!(std::panic::catch_unwind(|| bad_simulator.clone().schedule(problem.jobs[2])).is_err());
		bad_simulator.schedule(problem.jobs[0]);
		assert!(bad_simulator.has_missed_deadline());
		assert_eq!(2, bad_simulator.num_dispatched_jobs());

		let mut good_simulator = Simulator::new(&problem);
		good_simulator.schedule(problem.jobs[0]);
		good_simulator.schedule(problem.jobs[1]);
		good_simulator.schedule(problem.jobs[2]);
		assert!(!good_simulator.has_missed_deadline());
		assert_eq!(3, good_simulator.num_dispatched_jobs());
	}

	#[test]
	fn test_with_mixed_constraints_and_two_cores() {
		let mut problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 0, 20, 20),
				Job::release_to_deadline(1, 0, 30, 32),
				Job::release_to_deadline(2, 0, 30, 100)
			],
			constraints: vec![
				Constraint::new(0, 1, 2, ConstraintType::StartToStart),
				Constraint::new(0, 2, 10, ConstraintType::FinishToStart)
			],
			num_cores: 2
		};
		problem.validate();
		strengthen_bounds_using_constraints(&mut problem);

		assert!(std::panic::catch_unwind(|| Simulator::new(&problem).schedule(problem.jobs[1])).is_err());

		let mut bad_simulator = Simulator::new(&problem);
		bad_simulator.schedule(problem.jobs[0]);
		bad_simulator.schedule(problem.jobs[2]);
		bad_simulator.schedule(problem.jobs[1]);
		assert!(bad_simulator.has_missed_deadline());
		assert_eq!(3, bad_simulator.num_dispatched_jobs());

		let mut good_simulator = Simulator::new(&problem);
		good_simulator.schedule(problem.jobs[0]);
		good_simulator.schedule(problem.jobs[1]);
		good_simulator.schedule(problem.jobs[2]);
		assert!(!good_simulator.has_missed_deadline());
		assert_eq!(3, good_simulator.num_dispatched_jobs());
	}

	#[test]
	fn test_predict_start_time_with_one_core() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 0, 20, 50),
				Job::release_to_deadline(1, 10, 30, 50)
			],
			constraints: vec![],
			num_cores: 1
		};
		problem.validate();

		let mut simulator = Simulator::new(&problem);
		assert_eq!(0, simulator.predict_start_time(problem.jobs[0]));
		assert_eq!(20, simulator.predict_next_start_time(problem.jobs[0]));
		assert_eq!(10, simulator.predict_start_time(problem.jobs[1]));
		assert_eq!(40, simulator.predict_next_start_time(problem.jobs[1]));

		simulator.schedule(problem.jobs[0]);
		assert_eq!(20, simulator.predict_start_time(problem.jobs[1]));
		assert_eq!(50, simulator.predict_next_start_time(problem.jobs[1]));
	}

	#[test]
	fn test_predict_start_time_with_two_cores() {
		let problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 0, 20, 50),
				Job::release_to_deadline(1, 10, 30, 50)
			],
			constraints: vec![],
			num_cores: 2
		};
		problem.validate();

		let mut simulator = Simulator::new(&problem);
		assert_eq!(0, simulator.predict_start_time(problem.jobs[0]));
		assert_eq!(0, simulator.predict_next_start_time(problem.jobs[0]));
		assert_eq!(10, simulator.predict_start_time(problem.jobs[1]));
		assert_eq!(10, simulator.predict_next_start_time(problem.jobs[1]));

		simulator.schedule(problem.jobs[0]);
		assert_eq!(10, simulator.predict_start_time(problem.jobs[1]));
		assert_eq!(20, simulator.predict_next_start_time(problem.jobs[1]));

		simulator = Simulator::new(&problem);
		simulator.schedule(problem.jobs[1]);
		assert_eq!(10, simulator.predict_start_time(problem.jobs[0]));
		assert_eq!(30, simulator.predict_next_start_time(problem.jobs[0]));
	}
}
