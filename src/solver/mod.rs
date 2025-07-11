use crate::problem::Problem;
use crate::solver::forced::ForcedJobsContext;
use crate::solver::precedence_tracker::PrecedenceTracker;

mod forced;
mod precedence_tracker;
mod attempt;
mod queue;
mod job_ordering;
mod skip_distribution;

struct FatProblem {
	problem: Problem, // TODO Maybe turn into reference
	forced: ForcedJobsContext,
	precedence: PrecedenceTracker,
}

impl FatProblem {
	fn new(problem: Problem) -> Self {
		Self {
			forced: ForcedJobsContext::new(&problem),
			precedence: PrecedenceTracker::new(&problem),
			problem
		}
	}
}
