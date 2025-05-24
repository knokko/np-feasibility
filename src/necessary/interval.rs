use crate::necessary::interval_tree::{IntervalTree, JobInterval};
use crate::necessary::pack::is_certainly_unpackable;
use crate::problem::*;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum IntervalResult {
	Finished,
	Running,
	CertainlyInfeasible,
}

struct IntervalTest<'a> {
	problem: &'a Problem,
	interval_tree: IntervalTree,

	next_job_index: usize,

	relevant_jobs: Vec<JobInterval>,
	start_time: Time,
	end_time: Time,

	required_loads: Vec<Time>,
	corresponding_jobs: Vec<usize>,
}

impl<'a> IntervalTest<'a> {
	fn new(problem: &'a Problem) -> Self {
		let mut interval_tree = IntervalTree::new();
		for job in &problem.jobs {
			interval_tree.insert(JobInterval {
				job: job.get_index(),
				start: job.earliest_start,
				end: job.get_latest_finish()
			});
		}
		interval_tree.split();

		Self {
			problem, interval_tree,
			next_job_index: 0,
			relevant_jobs: Vec::new(),
			start_time: 0,
			end_time: 0,
			required_loads: Vec::new(),
			corresponding_jobs: Vec::new(),
		}
	}

	fn next(&mut self) -> IntervalResult {
		let next_job = self.problem.jobs[self.next_job_index];
		self.next_job_index += 1;

		self.start_time = next_job.earliest_start;
		self.end_time = next_job.get_latest_finish();

		// Find all jobs that satisfy both conditions:
		// - their latest start time is smaller than end_time
		// - their earliest finish time is larger than start_time
		self.interval_tree.query(JobInterval {
			job: next_job.get_index(),
			start: self.start_time,
			end: self.end_time
		}, &mut self.relevant_jobs);

		self.required_loads.clear();
		self.corresponding_jobs.clear();

		for interval in &self.relevant_jobs {
			let mut non_overlapping_time = 0;
			if interval.start < self.start_time {
				non_overlapping_time = self.start_time - interval.start;
			}
			if interval.end > self.end_time {
				non_overlapping_time = Time::max(
					non_overlapping_time, interval.end - self.end_time
				);
			}

			let exec_time = self.problem.jobs[interval.job].get_execution_time();
			if exec_time > non_overlapping_time {
				self.required_loads.push(Time::min(
					exec_time - non_overlapping_time, self.end_time - self.start_time
				));
				self.corresponding_jobs.push(interval.job);
			}
		}

		self.relevant_jobs.clear();
		if is_certainly_unpackable(self.problem.num_cores, self.end_time - self.start_time, &mut self.required_loads) {
			IntervalResult::CertainlyInfeasible
		} else if self.next_job_index < self.problem.jobs.len() {
			IntervalResult::Running
		} else {
			IntervalResult::Finished
		}
	}
}

pub fn run_feasibility_interval_test(problem: &Problem) -> bool {
	let mut test = IntervalTest::new(problem);
	loop {
		match test.next() {
			IntervalResult::Finished => return false,
			IntervalResult::Running => continue,
			IntervalResult::CertainlyInfeasible => return true,
		}
	}
}
