use std::fs::read_dir;
use std::process::Command;

fn main() {
	let root_directory = read_dir("/home/knokko/np-feasibility-problems/infeasible-problems/").unwrap();
	let mut solved_infeasible = 0;
	let mut total_infeasible = 0;
	for (_, category) in root_directory.enumerate() {
		let category_directory = category.unwrap();
		let raw_category_name = category_directory.file_name();
		let category_name = raw_category_name.to_str().unwrap();
		let num_cores = get_num_cores(category_name);

		for (_, raw_file) in category_directory.path().read_dir().unwrap().enumerate() {
			let file = raw_file.unwrap();
			let raw_file_name = file.file_name();
			let file_name = raw_file_name.to_str().unwrap();
			if !file_name.ends_with("jobs.csv") {
				continue;
			}

			let constraint_file = file.path().to_str().unwrap().replace("jobs.csv", "constraints.csv");

			println!("File is {} and category is {}", file_name, category_name);
			let output = Command::new("../target/release/np-feasibility")
				.arg("--jobs-file").arg(file.path())
				.arg("--precedence-file").arg(constraint_file)
				.arg("--num-cores").arg(format!("{}", num_cores))
				.output().unwrap();
			if !output.status.success() {
				panic!("Failed to run np-feasibility {}", String::from_utf8(output.stderr).unwrap());
			}
			let certainly_infeasible = String::from_utf8(output.stdout).unwrap().contains("INFEASIBLE");
			if certainly_infeasible {
				solved_infeasible += 1;
			}
			total_infeasible += 1;
		}
	}

	println!("Identified {}/{} certainly infeasible problems", solved_infeasible, total_infeasible);
}

fn get_num_cores(category_name: &str) -> usize {
	let end_index = category_name.find("cores").unwrap();
	let mut start_index = end_index - 1;
	while start_index > 0 && category_name.as_bytes()[start_index].is_ascii_digit() {
		start_index -= 1;
	}
	let num_cores = &category_name[start_index + 1..end_index];
	num_cores.parse().unwrap()
}
