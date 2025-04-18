use clap::Parser;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = APP_NAME)]
#[command(version = VERSION)]
#[command(author = AUTHOR)]
#[command(about = "Non-preemptive feasibility test/static schedule generator", long_about = None)]
pub struct Args {
	/// The CSV file containing the jobs
	#[arg(short, long)]
	pub jobs_file: String,

	/// The CSV file containing the (precedence) constraints
	#[arg(short, long)]
	pub precedence_file: Option<String>,

	/// The number of jobs that the target system can run in parallel
	#[arg(short, long)]
	pub num_cores: u32,
}
