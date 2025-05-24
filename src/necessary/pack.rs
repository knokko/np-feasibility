use crate::problem::Time;

pub fn is_certainly_unpackable(num_processors: u32, bin_size: Time, jobs: &mut Vec<Time>) -> bool {
	debug_assert!(num_processors >= 1);
	if jobs.is_empty() {
		return false;
	}

	let mut total = 0;
	for job in jobs.iter() {
		if *job > bin_size {
			return true;
		}
		total += *job;
	}

	if jobs.len() as u32 <= num_processors {
		return false;
	}
	if total > num_processors as Time * bin_size {
		return true;
	}
	if num_processors == 1 || jobs.len() <= 2 {
		return false;
	}

	jobs.sort();

	if jobs.len() == 3 {
		debug_assert_eq!(num_processors, 2);
		return jobs[0] + jobs[1] > bin_size;
	}
	debug_assert!(jobs.len() >= 4);

	let smallest2 = Time::min(jobs[2], jobs[0] + jobs[1]);
	let mut min_wasted_space = 0;
	for index in (1 .. jobs.len()).rev() {
		let duration = jobs[index];

		if duration + jobs[0] > bin_size {
			min_wasted_space += bin_size - duration;
			continue;
		}

		if index > 1 && duration + jobs[1] > bin_size {
			debug_assert!(duration + jobs[0] <= bin_size);
			min_wasted_space += bin_size - jobs[0] - duration;
			continue;
		}

		if index > 2 && duration + smallest2 > bin_size {
			debug_assert!(duration + jobs[1] <= bin_size);
			min_wasted_space += bin_size - jobs[1] - duration;
		}
	}

	total + min_wasted_space > num_processors as Time * bin_size
}

#[cfg(test)]
mod tests {
	use super::is_certainly_unpackable;

	#[test]
	fn test_without_jobs() {
		assert!(!is_certainly_unpackable(1, 10, &mut vec![]));
		assert!(!is_certainly_unpackable(2, 10, &mut vec![]));
		assert!(!is_certainly_unpackable(5, 10, &mut vec![]));

		assert!(!is_certainly_unpackable(1, 0, &mut vec![]));
		assert!(!is_certainly_unpackable(2, 0, &mut vec![]));
		assert!(!is_certainly_unpackable(5, 0, &mut vec![]));
	}

	#[test]
	fn test_with_1_job() {
		let job = &mut vec![100];
		assert!(is_certainly_unpackable(1, 99, job));
		assert!(is_certainly_unpackable(5, 99, job));

		assert!(!is_certainly_unpackable(1, 100, job));
		assert!(!is_certainly_unpackable(5, 100, job));
	}

	#[test]
	fn test_with_2_equally_long_jobs() {
		let mut jobs = vec![100, 100];
		assert!(is_certainly_unpackable(1, 99, &mut jobs));
		assert!(is_certainly_unpackable(2, 99, &mut jobs));
		assert!(is_certainly_unpackable(5, 99, &mut jobs));

		assert!(is_certainly_unpackable(1, 100, &mut jobs));
		assert!(!is_certainly_unpackable(2, 100, &mut jobs));
		assert!(!is_certainly_unpackable(5, 100, &mut jobs));

		assert!(is_certainly_unpackable(1, 197, &mut jobs));
		assert!(!is_certainly_unpackable(1, 200, &mut jobs));
		assert!(!is_certainly_unpackable(1, 300, &mut jobs));
	}

	#[test]
	fn test_with_2_jobs_of_different_length() {
		let mut jobs = vec![100, 50];
		assert!(is_certainly_unpackable(1, 99, &mut jobs));
		assert!(is_certainly_unpackable(2, 99, &mut jobs));
		assert!(is_certainly_unpackable(5, 99, &mut jobs));

		assert!(is_certainly_unpackable(1, 100, &mut jobs));
		assert!(!is_certainly_unpackable(2, 100, &mut jobs));
		assert!(!is_certainly_unpackable(5, 100, &mut jobs));

		assert!(is_certainly_unpackable(1, 149, &mut jobs));
		assert!(!is_certainly_unpackable(1, 150, &mut jobs));
		assert!(!is_certainly_unpackable(1, 197, &mut jobs));
	}

	#[test]
	fn test_with_3_equally_long_jobs() {
		let mut jobs = vec![100, 100, 100];
		assert!(is_certainly_unpackable(1, 99, &mut jobs));
		assert!(is_certainly_unpackable(3, 99, &mut jobs));
		assert!(is_certainly_unpackable(100, 99, &mut jobs));

		assert!(is_certainly_unpackable(1, 100, &mut jobs));
		assert!(is_certainly_unpackable(2, 100, &mut jobs));
		assert!(!is_certainly_unpackable(3, 100, &mut jobs));
		assert!(!is_certainly_unpackable(4, 100, &mut jobs));

		assert!(is_certainly_unpackable(1, 299, &mut jobs));
		assert!(!is_certainly_unpackable(1, 300, &mut jobs));
		assert!(!is_certainly_unpackable(1, 301, &mut jobs));

		assert!(is_certainly_unpackable(2, 199, &mut jobs));
		assert!(!is_certainly_unpackable(2, 200, &mut jobs));
		assert!(!is_certainly_unpackable(2, 299, &mut jobs));
	}

	#[test]
	fn test_with_3_jobs_of_different_length() {
		let mut jobs = vec![100, 50, 60];
		assert!(is_certainly_unpackable(1, 209, &mut jobs));
		assert!(is_certainly_unpackable(2, 109, &mut jobs));
		assert!(is_certainly_unpackable(5, 99, &mut jobs));

		assert!(is_certainly_unpackable(1, 110, &mut jobs));
		assert!(!is_certainly_unpackable(2, 110, &mut jobs));
		assert!(!is_certainly_unpackable(5, 110, &mut jobs));

		assert!(!is_certainly_unpackable(1, 210, &mut jobs));
		assert!(!is_certainly_unpackable(2, 210, &mut jobs));
		assert!(!is_certainly_unpackable(3, 100, &mut jobs));
	}

	#[test]
	fn test_with_4_equally_long_jobs() {
		let mut jobs = vec![100, 100, 100, 100];
		assert!(is_certainly_unpackable(1, 99, &mut jobs));
		assert!(is_certainly_unpackable(4, 99, &mut jobs));
		assert!(is_certainly_unpackable(100, 99, &mut jobs));

		assert!(is_certainly_unpackable(1, 100, &mut jobs));
		assert!(is_certainly_unpackable(3, 100, &mut jobs));
		assert!(!is_certainly_unpackable(4, 100, &mut jobs));
		assert!(!is_certainly_unpackable(123, 100, &mut jobs));

		assert!(is_certainly_unpackable(1, 399, &mut jobs));
		assert!(!is_certainly_unpackable(1, 400, &mut jobs));
		assert!(!is_certainly_unpackable(1, 401, &mut jobs));

		assert!(is_certainly_unpackable(2, 199, &mut jobs));
		assert!(!is_certainly_unpackable(2, 200, &mut jobs));
		assert!(!is_certainly_unpackable(2, 399, &mut jobs));
	}

	#[test]
	fn test_with_4_jobs_of_different_length() {
		let mut jobs = vec![100, 50, 80, 20];
		assert!(is_certainly_unpackable(1, 249, &mut jobs));
		assert!(is_certainly_unpackable(2, 129, &mut jobs));
		assert!(is_certainly_unpackable(4, 99, &mut jobs));
		assert!(is_certainly_unpackable(9, 99, &mut jobs));

		assert!(!is_certainly_unpackable(1, 250, &mut jobs));
		assert!(!is_certainly_unpackable(2, 130, &mut jobs));
		assert!(!is_certainly_unpackable(5, 130, &mut jobs));

		assert!(!is_certainly_unpackable(3, 100, &mut jobs));
	}
}
