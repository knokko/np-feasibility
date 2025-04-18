mod bounds;
mod cli;
mod problem;
mod parser;

use clap::Parser;
use cli::Args;
use parser::parse_problem;

fn main() {
	let args = Args::parse();
	let problem = parse_problem(
		&args.jobs_file, args.precedence_file.as_deref(), args.num_cores
	);
	println!("Found {} jobs and {} constraints using {} cores", problem.jobs.len(), problem.constraints.len(), problem.num_cores);
}
