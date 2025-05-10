use std::cmp::{max, min};
use crate::problem::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OccupationStrengthenResult {
	Unchanged,
	Modified,
	Infeasible
}

/// Attempts to strengthen the bounds of the jobs of the given problem (their `earliest_start` and
/// `latest_start`), by analyzing intervals during which cores are certainly occupied by jobs.
///
/// For instance, consider a job that:
/// - is released at time 10
/// - has an execution time of 15 time units
/// - has a deadline at time 30
///
/// Note that:
/// - when the job is started at time 10 (the earliest possible time), it completes at time 25
/// - when the job is finished at time 30 (the latest acceptable time), it started at time 15
///
/// Most importantly, in *all cases where its deadline is met*, the job *must* be executing between
/// time 15 and time 25, and therefor occupy a core between time 15 and 25.
///
/// When the problem has `c` cores and more than `c` cores are certainly occupied at some time
/// `t`, the problem is certainly infeasible. This function tests whether such a point in time
/// exists, and declares the problem infeasible if such a point is found.
///
/// When exactly `c` cores are certainly occupied at some time `t`, we can sometimes constrain
/// the `earliest_start` and `latest_start` of some jobs. For instance, assume that:
/// - exactly `c` cores are certainly occupied between time 10 and 20
/// - some job `j` is released at time 5, takes 6 time units, and has a deadline at time 30
///
/// When `j` is started at the earliest time 5, it would finish at time 11, during which all `c`
/// cores are already certainly occupied by other jobs. Therefor, `j` cannot start at time 5. In
/// fact, it cannot start until time 20, so we can update its `earliest_start` from 5 to 20.
/// Since its deadline is still at time 30, it certainly occupies a core between time 24 and 26.
/// That certain core occupation might influence the `earliest_start` or `latest_start` of yet
/// another job `j2`...
///
/// This function will repeatedly try to strengthen the `earliest_start` and `latest_start` of all
/// jobs using this reasoning.
pub fn strengthen_bounds_using_core_occupation(problem: &mut Problem) -> OccupationStrengthenResult {
	let mut timeline = OccupationTimeline::new(problem.num_cores);
	for job in &problem.jobs {
		if timeline.insert(*job) {
			return OccupationStrengthenResult::Infeasible;
		}
	}

	let mut modified_anything = false;
	loop {
		let mut modified_interval = false;
		for job in &mut problem.jobs {
			let result = timeline.refine(job);
			if result == RefineResult::Infeasible {
				return OccupationStrengthenResult::Infeasible;
			}
			if result == RefineResult::ModifiedJobAndIntervals {
				modified_interval = true;
				modified_anything = true;
			}
			if result == RefineResult::ModifiedJob {
				modified_anything = true;
			}
		}

		if !modified_interval {
			break;
		}
	}

	if modified_anything {
		OccupationStrengthenResult::Modified
	} else {
		OccupationStrengthenResult::Unchanged
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct OccupationInterval {
	start: Time,
	num_cores: u32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum RefineResult {
	Unchanged,
	ModifiedJob,
	ModifiedJobAndIntervals,
	Infeasible
}

#[derive(Debug, Clone)]
struct OccupationTimeline {
	intervals: Vec<OccupationInterval>,
	max_num_cores: u32,
}

impl OccupationTimeline {
	pub fn new(num_cores: u32) -> Self {
		OccupationTimeline {
			intervals: vec![OccupationInterval { start: 0, num_cores: 0 }],
			max_num_cores: num_cores
		}
	}

	/// Returns true if the problem is certainly infeasible
	pub fn insert(&mut self, job: Job) -> bool {
		if job.get_earliest_finish() <= job.latest_start {
			return false;
		}

		let mut end_index = match self.intervals.binary_search_by_key(
			&job.get_earliest_finish(), |i| i.start
		) {
			Ok(exact_bound_index) => {
				exact_bound_index - 1
			},
			Err(bound_index) => {
				let end_index = bound_index - 1;
				self.intervals.insert(bound_index, OccupationInterval {
					start: job.get_earliest_finish(),
					num_cores: self.intervals[end_index].num_cores }
				);
				end_index
			}
		};

		let start_index = match self.intervals.binary_search_by_key(
			&job.latest_start, |i| i.start
		) {
			Ok(exact_start_index) => exact_start_index,
			Err(next_start_index) => {
				let num_cores = self.intervals[next_start_index - 1].num_cores;
				if next_start_index < self.intervals.len() &&
					num_cores + 1 == self.intervals[next_start_index].num_cores &&
					self.intervals[next_start_index].start >= job.get_earliest_finish() {
					self.intervals[next_start_index].start = job.latest_start;
				} else {
					self.intervals.insert(next_start_index, OccupationInterval {
						start: job.latest_start, num_cores
					});
					end_index += 1;
				}
				next_start_index
			}
		};

		for index in start_index ..= end_index {
			let more_cores = self.intervals[index].num_cores + 1;
			if more_cores > self.max_num_cores {
				return true;
			}
			self.intervals[index].num_cores = more_cores;
		}

		while start_index > 0 && self.intervals[start_index].num_cores == self.intervals[start_index - 1].num_cores {
			self.intervals.remove(start_index);
			end_index -= 1;
		}
		while end_index + 1 < self.intervals.len() && self.intervals[end_index].num_cores == self.intervals[end_index + 1].num_cores {
			self.intervals.remove(end_index + 1);
		}
		false
	}

	fn find_interruption(&self, start: Time, bound: Time) -> Option<usize> {
		let start_index = self.intervals.binary_search_by_key(
			&start, |i| i.start
		).unwrap_or_else(|next_start_index| next_start_index - 1);

		let bound_index = self.intervals.binary_search_by_key(
			&bound, |i| i.start
		).unwrap_or_else(|next_bound_index| next_bound_index);

		for index in start_index .. bound_index {
			if self.intervals[index].num_cores == self.max_num_cores {
				return Some(index);
			}
		}

		None
	}

	pub fn refine(&mut self, job: &mut Job) -> RefineResult {
		if job.earliest_start >= job.latest_start {
			return RefineResult::Unchanged;
		}

		let old = *job;
		loop {
			let mut interruption_bound = job.get_earliest_finish();
			if old.get_earliest_finish() > old.latest_start {
				interruption_bound = min(interruption_bound, old.latest_start);
			}
			let maybe_interruption_index = self.find_interruption(
				job.earliest_start, interruption_bound
			);
			if let Some(interruption_index) = maybe_interruption_index {
				job.earliest_start = self.intervals[interruption_index + 1].start;
				if old.get_earliest_finish() > old.latest_start {
					job.earliest_start = min(job.earliest_start, old.latest_start);
					if job.earliest_start == job.latest_start {
						break;
					}
				}
			} else {
				break;
			}
		}

		loop {
			let maybe_interruption_index = self.find_interruption(
				max(job.latest_start, job.get_earliest_finish()), job.get_latest_finish()
			);
			if let Some(interruption_index) = maybe_interruption_index {
				job.set_latest_finish(self.intervals[interruption_index].start);
				if old.get_earliest_finish() > old.latest_start {
					job.set_latest_finish(max(job.get_latest_finish(), old.get_earliest_finish()));
					if job.earliest_start == job.latest_start {
						break;
					}
				}
			} else {
				break;
			}
		}

		if job.is_certainly_infeasible() {
			return RefineResult::Infeasible;
		}

		let mut result = RefineResult::Unchanged;
		if *job != old {
			result = RefineResult::ModifiedJob;
			if old.get_earliest_finish() > old.latest_start {
				if job.latest_start < old.latest_start {
					self.insert(Job::release_to_deadline(
						job.get_index(), job.latest_start,
						old.latest_start - job.latest_start,
						old.latest_start
					));
					result = RefineResult::ModifiedJobAndIntervals;
				}
				if job.get_earliest_finish() > old.get_earliest_finish() {
					self.insert(Job::release_to_deadline(
						job.get_index(), old.get_earliest_finish(),
						job.get_earliest_finish() - old.get_earliest_finish(),
						job.get_earliest_finish()
					));
					result = RefineResult::ModifiedJobAndIntervals;
				}
			} else if job.get_earliest_finish() > job.latest_start {
				self.insert(*job);
				result = RefineResult::ModifiedJobAndIntervals;
			}
		}

		result
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_interval_starts_at_zero() {
		let mut timeline = OccupationTimeline::new(1);
		assert!(!timeline.insert(Job::release_to_deadline(0, 0, 15, 15)));
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 1
		}, OccupationInterval {
			start: 15, num_cores: 0
		}], timeline.intervals);

		assert_eq!(Some(0), timeline.find_interruption(0, 100));
		assert_eq!(Some(0), timeline.find_interruption(14, 100));
		assert_eq!(None, timeline.find_interruption(15, 100));
		assert_eq!(None, timeline.find_interruption(50, 100));
	}

	#[test]
	fn test_intervals_with_overlap() {
		let mut timeline = OccupationTimeline::new(6);

		// Certainly occupies time 15 to 25
		assert!(!timeline.insert(Job::release_to_deadline(0, 10, 15, 30)));
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 15, num_cores: 1
		}, OccupationInterval {
			start: 25, num_cores: 0
		}], timeline.intervals);

		// Certainly occupies time 20 to 42
		assert!(!timeline.insert(Job::release_to_deadline(10, 12, 30, 50)));
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 15, num_cores: 1
		}, OccupationInterval {
			start: 20, num_cores: 2
		}, OccupationInterval {
			start: 25, num_cores: 1
		}, OccupationInterval {
			start: 42, num_cores: 0
		}], timeline.intervals);

		// Also certainly occupies time 20 to 42
		assert!(!timeline.insert(Job::release_to_deadline(8, 20, 22, 42)));
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 15, num_cores: 1
		}, OccupationInterval {
			start: 20, num_cores: 3
		}, OccupationInterval {
			start: 25, num_cores: 2
		}, OccupationInterval {
			start: 42, num_cores: 0
		}], timeline.intervals);

		// Certainly occupies time 21 to 24
		assert!(!timeline.insert(Job::release_to_deadline(2, 21, 3, 24)));
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 15, num_cores: 1
		}, OccupationInterval {
			start: 20, num_cores: 3
		}, OccupationInterval {
			start: 21, num_cores: 4
		}, OccupationInterval {
			start: 24, num_cores: 3
		}, OccupationInterval {
			start: 25, num_cores: 2
		}, OccupationInterval {
			start: 42, num_cores: 0
		}], timeline.intervals);

		// Certainly occupies time 21 to 23
		assert!(!timeline.insert(Job::release_to_deadline(2, 21, 2, 23)));
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 15, num_cores: 1
		}, OccupationInterval {
			start: 20, num_cores: 3
		}, OccupationInterval {
			start: 21, num_cores: 5
		}, OccupationInterval {
			start: 23, num_cores: 4
		}, OccupationInterval {
			start: 24, num_cores: 3
		}, OccupationInterval {
			start: 25, num_cores: 2
		}, OccupationInterval {
			start: 42, num_cores: 0
		}], timeline.intervals);

		// Also certainly occupies time 21 to 23
		assert!(!timeline.insert(Job::release_to_deadline(3, 20, 3, 24)));
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 15, num_cores: 1
		}, OccupationInterval {
			start: 20, num_cores: 3
		}, OccupationInterval {
			start: 21, num_cores: 6
		}, OccupationInterval {
			start: 23, num_cores: 4
		}, OccupationInterval {
			start: 24, num_cores: 3
		}, OccupationInterval {
			start: 25, num_cores: 2
		}, OccupationInterval {
			start: 42, num_cores: 0
		}], timeline.intervals);

		assert_eq!(None, timeline.find_interruption(0, 21));
		assert_eq!(Some(3), timeline.find_interruption(0, 22));
		assert_eq!(Some(3), timeline.find_interruption(0, 100));
		assert_eq!(Some(3), timeline.find_interruption(22, 100));
		assert_eq!(None, timeline.find_interruption(23, 100));
	}

	#[test]
	fn test_intervals_without_overlap() {
		let mut timeline = OccupationTimeline::new(1);
		assert!(!timeline.insert(Job::release_to_deadline(0, 10, 15, 30)));
		assert!(!timeline.insert(Job::release_to_deadline(0, 30, 15, 50)));
		assert!(!timeline.insert(Job::release_to_deadline(0, 50, 15, 70)));

		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 15, num_cores: 1
		}, OccupationInterval {
			start: 25, num_cores: 0
		}, OccupationInterval {
			start: 35, num_cores: 1
		}, OccupationInterval {
			start: 45, num_cores: 0
		}, OccupationInterval {
			start: 55, num_cores: 1
		}, OccupationInterval {
			start: 65, num_cores: 0
		}], timeline.intervals);

		assert_eq!(None, timeline.find_interruption(0, 15));
		assert_eq!(Some(1), timeline.find_interruption(0, 16));
		for start in vec![10, 15, 20] {
			assert_eq!(Some(1), timeline.find_interruption(start, 20));
		}
		assert_eq!(Some(1), timeline.find_interruption(24, 35));
		assert_eq!(None, timeline.find_interruption(25, 35));
		assert_eq!(Some(3), timeline.find_interruption(25, 36));
		assert_eq!(Some(3), timeline.find_interruption(25, 100));
		assert_eq!(Some(3), timeline.find_interruption(44, 100));
		assert_eq!(Some(5), timeline.find_interruption(45, 100));
	}

	#[test]
	fn test_jobs_without_certain_execution() {
		let mut timeline = OccupationTimeline::new(1);
		assert!(!timeline.insert(Job::release_to_deadline(0, 0, 10, 30)));
		assert!(!timeline.insert(Job::release_to_deadline(0, 0, 15, 30)));
		assert!(!timeline.insert(Job::release_to_deadline(0, 50, 20, 90)));

		assert_eq!(vec![OccupationInterval { start: 0, num_cores: 0 }], timeline.intervals);
		assert_eq!(None, timeline.find_interruption(0, 12345));
	}

	#[test]
	fn test_stacking_intervals() {
		let mut timeline = OccupationTimeline::new(100);

		let job = Job::release_to_deadline(0, 30, 20, 50);
		for _ in 0..100 {
			assert!(!timeline.insert(job));
		}

		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 30, num_cores: 100
		}, OccupationInterval {
			start: 50, num_cores: 0
		}], timeline.intervals);

		// Next insertion causes deadline miss since there are 'only' 100 cores
		assert!(timeline.insert(job));

		assert_eq!(None, timeline.find_interruption(0, 30));
		assert_eq!(Some(1), timeline.find_interruption(0, 31));
		assert_eq!(Some(1), timeline.find_interruption(49, 100));
		assert_eq!(None, timeline.find_interruption(50, 100));
	}

	#[test]
	fn test_overwriting_insert() {
		let mut timeline = OccupationTimeline::new(2);
		assert!(!timeline.insert(Job::release_to_deadline(0, 0, 60, 100)));
		assert!(!timeline.insert(Job::release_to_deadline(1, 10, 10, 20)));
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 10, num_cores: 1
		}, OccupationInterval {
			start: 20, num_cores: 0
		}, OccupationInterval {
			start: 40, num_cores: 1
		}, OccupationInterval {
			start: 60, num_cores: 0
		}], timeline.intervals);
		assert!(!timeline.insert(Job::release_to_deadline(2, 15, 85, 100)));
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 10, num_cores: 1
		}, OccupationInterval {
			start: 15, num_cores: 2
		}, OccupationInterval {
			start: 20, num_cores: 1
		}, OccupationInterval {
			start: 40, num_cores: 2
		}, OccupationInterval {
			start: 60, num_cores: 1
		}, OccupationInterval {
			start: 100, num_cores: 0
		}], timeline.intervals);
	}

	#[test]
	fn overwriting_regression_test() {
		let mut timeline = OccupationTimeline::new(2);
		timeline.insert(Job::release_to_deadline(0, 5, 10, 20));
		timeline.insert(Job::release_to_deadline(1, 15, 6, 21));
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 10, num_cores: 1
		}, OccupationInterval {
			start: 21, num_cores: 0
		}], timeline.intervals);

		let mut timeline2 = timeline.clone();
		let mut timeline3 = timeline.clone();
		timeline.insert(Job::release_to_deadline(0, 5, 5, 10));
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 5, num_cores: 1
		}, OccupationInterval {
			start: 21, num_cores: 0
		}], timeline.intervals);

		timeline2.insert(Job::release_to_deadline(0, 4, 5, 10));
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 5, num_cores: 1
		}, OccupationInterval {
			start: 9, num_cores: 0
		}, OccupationInterval {
			start: 10, num_cores: 1
		}, OccupationInterval {
			start: 21, num_cores: 0
		}], timeline2.intervals);

		timeline3.insert(Job::release_to_deadline(0, 10, 12, 33));
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 10, num_cores: 1
		}, OccupationInterval {
			start: 22, num_cores: 0
		}], timeline3.intervals);
	}

	#[test]
	fn test_insert_fill_gap() {
		let mut timeline = OccupationTimeline::new(1);
		timeline.insert(Job::release_to_deadline(0, 5, 10, 15));
		timeline.insert(Job::release_to_deadline(0, 20, 10, 30));

		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 5, num_cores: 1
		}, OccupationInterval {
			start: 15, num_cores: 0
		}, OccupationInterval {
			start: 20, num_cores: 1
		}, OccupationInterval {
			start: 30, num_cores: 0
		}], timeline.intervals);

		timeline.insert(Job::release_to_deadline(0, 15, 5, 20));
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 5, num_cores: 1
		}, OccupationInterval {
			start: 30, num_cores: 0
		}], timeline.intervals);
	}

	#[test]
	fn test_simple_feasible_refinement() {
		let mut timeline = OccupationTimeline::new(1);
		let mut long_job = Job::release_to_deadline(0, 5, 10, 20);
		let mut early_job = Job::release_to_deadline(1, 0, 5, 20);
		let mut late_job = Job::release_to_deadline(2, 5, 6, 21);
		timeline.insert(long_job);
		timeline.insert(early_job);
		timeline.insert(late_job);

		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 10, num_cores: 1
		}, OccupationInterval {
			start: 15, num_cores: 0
		}], timeline.intervals);

		assert_eq!(RefineResult::Unchanged, timeline.refine(&mut long_job));
		assert_eq!(RefineResult::Unchanged, timeline.refine(&mut early_job));

		assert_eq!(RefineResult::ModifiedJobAndIntervals, timeline.refine(&mut late_job));
		assert_eq!(15, late_job.earliest_start);
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 10, num_cores: 1
		}, OccupationInterval {
			start: 21, num_cores: 0
		}], timeline.intervals);
		assert_eq!(RefineResult::Unchanged, timeline.refine(&mut late_job));

		assert_eq!(RefineResult::ModifiedJob, timeline.refine(&mut early_job));
		assert_eq!(5, early_job.latest_start);
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 10, num_cores: 1
		}, OccupationInterval {
			start: 21, num_cores: 0
		}], timeline.intervals);
		assert_eq!(RefineResult::Unchanged, timeline.refine(&mut early_job));

		assert_eq!(RefineResult::ModifiedJobAndIntervals, timeline.refine(&mut long_job));
		assert_eq!(5, long_job.earliest_start);
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 5, num_cores: 1
		}, OccupationInterval {
			start: 21, num_cores: 0
		}], timeline.intervals);
		assert_eq!(RefineResult::Unchanged, timeline.refine(&mut long_job));

		assert_eq!(RefineResult::ModifiedJobAndIntervals, timeline.refine(&mut early_job));
		assert_eq!(0, early_job.latest_start);
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 1
		}, OccupationInterval {
			start: 21, num_cores: 0
		}], timeline.intervals);
		assert_eq!(RefineResult::Unchanged, timeline.refine(&mut early_job));
		assert_eq!(RefineResult::Unchanged, timeline.refine(&mut long_job));
		assert_eq!(RefineResult::Unchanged, timeline.refine(&mut late_job));
	}

	#[test]
	fn test_refinement_shift_to_right1() {
		let mut timeline = OccupationTimeline::new(1);
		let mut long_job = Job::release_to_deadline(0, 5, 10, 20);
		let mut early_job = Job::release_to_deadline(1, 4, 6, 20);
		timeline.insert(long_job);
		timeline.insert(early_job);

		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 10, num_cores: 1
		}, OccupationInterval {
			start: 15, num_cores: 0
		}], timeline.intervals);

		assert_eq!(RefineResult::Unchanged, timeline.refine(&mut long_job));
		assert_eq!(RefineResult::ModifiedJobAndIntervals, timeline.refine(&mut early_job));
		assert_eq!(4, early_job.latest_start);
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 4, num_cores: 1
		}, OccupationInterval {
			start: 15, num_cores: 0
		}], timeline.intervals);
		assert_eq!(RefineResult::Unchanged, timeline.refine(&mut early_job));

		assert_eq!(RefineResult::ModifiedJobAndIntervals, timeline.refine(&mut long_job));
		assert_eq!(10, long_job.earliest_start);
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 4, num_cores: 1
		}, OccupationInterval {
			start: 20, num_cores: 0
		}], timeline.intervals);
	}

	#[test]
	fn test_refinement_shift_to_right2() {
		let mut timeline = OccupationTimeline::new(1);
		let mut long_job = Job::release_to_deadline(0, 5, 10, 20);
		let mut early_job = Job::release_to_deadline(1, 3, 6, 20);
		timeline.insert(long_job);
		timeline.insert(early_job);

		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 10, num_cores: 1
		}, OccupationInterval {
			start: 15, num_cores: 0
		}], timeline.intervals);

		assert_eq!(RefineResult::Unchanged, timeline.refine(&mut long_job));
		assert_eq!(RefineResult::ModifiedJobAndIntervals, timeline.refine(&mut early_job));
		assert_eq!(4, early_job.latest_start);
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 4, num_cores: 1
		}, OccupationInterval {
			start: 9, num_cores: 0
		}, OccupationInterval {
			start: 10, num_cores: 1
		}, OccupationInterval {
			start: 15, num_cores: 0
		}], timeline.intervals);
		assert_eq!(RefineResult::Unchanged, timeline.refine(&mut early_job));

		assert_eq!(RefineResult::ModifiedJobAndIntervals, timeline.refine(&mut long_job));
		assert_eq!(9, long_job.earliest_start);
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 4, num_cores: 1
		}, OccupationInterval {
			start: 9, num_cores: 0
		}, OccupationInterval {
			start: 10, num_cores: 1
		}, OccupationInterval {
			start: 19, num_cores: 0
		}], timeline.intervals);
	}

	#[test]
	fn test_refinement_shift_to_left2() {
		let mut timeline = OccupationTimeline::new(1);
		let mut long_job = Job::release_to_deadline(0, 5, 10, 20);
		let mut late_job = Job::release_to_deadline(1, 5, 6, 22);
		timeline.insert(long_job);
		timeline.insert(late_job);

		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 10, num_cores: 1
		}, OccupationInterval {
			start: 15, num_cores: 0
		}], timeline.intervals);

		assert_eq!(RefineResult::Unchanged, timeline.refine(&mut long_job));
		assert_eq!(RefineResult::ModifiedJobAndIntervals, timeline.refine(&mut late_job));
		assert_eq!(15, late_job.earliest_start);
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 10, num_cores: 1
		}, OccupationInterval {
			start: 15, num_cores: 0
		}, OccupationInterval {
			start: 16, num_cores: 1
		}, OccupationInterval {
			start: 21, num_cores: 0
		}], timeline.intervals);
		assert_eq!(RefineResult::Unchanged, timeline.refine(&mut late_job));

		assert_eq!(RefineResult::ModifiedJobAndIntervals, timeline.refine(&mut long_job));
		assert_eq!(6, long_job.latest_start);
		assert_eq!(vec![OccupationInterval {
			start: 0, num_cores: 0
		}, OccupationInterval {
			start: 6, num_cores: 1
		}, OccupationInterval {
			start: 15, num_cores: 0
		}, OccupationInterval {
			start: 16, num_cores: 1
		}, OccupationInterval {
			start: 21, num_cores: 0
		}], timeline.intervals);
	}

	#[test]
	fn test_simple_feasible_strengthening() {
		let mut problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 5, 10, 20),
				Job::release_to_deadline(1, 0, 5, 20),
				Job::release_to_deadline(2, 5, 6, 21)
			],
			constraints: vec![],
			num_cores: 1
		};
		assert_eq!(OccupationStrengthenResult::Modified, strengthen_bounds_using_core_occupation(&mut problem));
		assert_eq!(0, problem.jobs[1].earliest_start);
		assert_eq!(0, problem.jobs[1].latest_start);
		assert_eq!(5, problem.jobs[0].earliest_start);
		assert_eq!(5, problem.jobs[0].latest_start);
		assert_eq!(15, problem.jobs[2].earliest_start);
		assert_eq!(15, problem.jobs[2].latest_start);
	}

	#[test]
	fn test_simple_infeasible_strengthening() {
		let mut problem = Problem {
			jobs: vec![
				Job::release_to_deadline(0, 0, 8, 15),
				Job::release_to_deadline(1, 7, 1, 8),
			],
			constraints: vec![],
			num_cores: 1
		};
		assert_eq!(OccupationStrengthenResult::Infeasible, strengthen_bounds_using_core_occupation(&mut problem));
	}

	fn periodic_infeasible_problem() -> Problem {
		Problem {
			jobs: vec![
				Job::release_to_deadline(0, 0, 11, 45),
				Job::release_to_deadline(1, 10, 1, 11),
				Job::release_to_deadline(2, 20, 1, 21),
				Job::release_to_deadline(3, 30, 1, 31),
				Job::release_to_deadline(4, 40, 1, 41),
			],
			constraints: vec![],
			num_cores: 1
		}
	}

	#[test]
	fn test_periodic_infeasible_strengthening() {
		let mut problem1 = periodic_infeasible_problem();
		let mut problem2 = problem1.clone();
		let mut problem3 = problem1.clone();
		assert_eq!(OccupationStrengthenResult::Infeasible, strengthen_bounds_using_core_occupation(&mut problem1));

		problem2.jobs[0] = Job::release_to_deadline(0, 0, 10, 45);
		assert_eq!(OccupationStrengthenResult::Modified, strengthen_bounds_using_core_occupation(&mut problem2));
		assert_eq!(0, problem2.jobs[0].earliest_start);
		assert_eq!(0, problem2.jobs[0].latest_start);

		problem3.jobs[0] = Job::release_to_deadline(0, 0, 9, 45);
		assert_eq!(OccupationStrengthenResult::Modified, strengthen_bounds_using_core_occupation(&mut problem3));
		assert_eq!(0, problem3.jobs[0].earliest_start);
		assert_eq!(31, problem3.jobs[0].latest_start);
	}

	#[test]
	fn periodic_regression_test() {
		let mut problem = periodic_infeasible_problem();

		let mut timeline = OccupationTimeline::new(problem.num_cores);
		for job in &problem.jobs {
			timeline.insert(*job);
		}

		assert_eq!(RefineResult::Infeasible, timeline.refine(&mut problem.jobs[0]));
	}
}
