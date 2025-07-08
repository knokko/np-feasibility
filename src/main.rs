mod bounds;
mod cli;
mod necessary;
mod parser;
mod permutation;
mod problem;
mod simulator;
mod solver;
mod sorted_job_iterator;

use bounds::*;
use clap::Parser;
use cli::Args;
use parser::parse_problem;
use permutation::ProblemPermutation;
use necessary::*;

fn main() {
	let args = Args::parse();
	let mut problem = parse_problem(
		&args.jobs_file, args.precedence_file.as_deref(), args.num_cores
	);
	println!("Found {} jobs and {} constraints using {} cores", problem.jobs.len(), problem.constraints.len(), problem.num_cores);

	let maybe_permutation = ProblemPermutation::possible(&mut problem);
	if let Some(permutation) = maybe_permutation {
		strengthen_bounds_using_constraints(&mut problem);
		debug_assert!(!strengthen_bounds_using_constraints(&mut problem));
		strengthen_bounds_using_core_occupation(&mut problem);
		permutation.transform_back(&mut problem);
		if problem.is_certainly_infeasible() || run_feasibility_load_test(&problem) || run_feasibility_interval_test(&problem) {
			println!("INFEASIBLE");
		} else {
			println!("This problem may or may not be feasible.");
		}
	} else {
		println!("This problem is cyclic! INFEASIBLE");
	}
}
