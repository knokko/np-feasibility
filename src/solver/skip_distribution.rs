use rand::prelude::*;

pub trait SkipDistribution {
	fn next_to_skip(&mut self) -> u32;
}

pub struct ZeroSkipDistribution;

impl SkipDistribution for ZeroSkipDistribution {
	fn next_to_skip(&mut self) -> u32 {
		0
	}
}

pub struct ExponentialSkipDistribution {
	skip_chance: f32
}

impl ExponentialSkipDistribution {
	pub fn new(skip_chance: f32) -> Self {
		Self { skip_chance }
	}
}

impl SkipDistribution for ExponentialSkipDistribution {

	fn next_to_skip(&mut self) -> u32 {
		let mut result = 0;
		let mut rng = rand::rng();
		while rng.random_bool(self.skip_chance as f64) {
			result += 1;
		}
		result
	}
}
