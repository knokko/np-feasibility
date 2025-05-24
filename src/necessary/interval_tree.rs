use crate::problem::Time;
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
pub struct JobInterval {
	pub job: usize,
	pub start: Time,
	pub end: Time,
}

pub struct IntervalTree {
	split_time: Time,
	middle: Vec<JobInterval>,

	before: Option<Rc<Self>>,
	after: Option<Rc<Self>>,

	stack: Vec<Rc<Self>>,
}

impl IntervalTree {
	pub fn new() -> Self {
		Self {
			split_time: 0,
			middle: Vec::new(),

			before: None,
			after: None,

			stack: Vec::new(),
		}
	}

	pub fn insert(&mut self, interval: JobInterval) {
		debug_assert!(self.before.is_none());
		debug_assert!(self.after.is_none());
		self.middle.push(interval);
	}

	pub fn split(&mut self) {
		debug_assert!(self.before.is_none());
		debug_assert!(self.after.is_none());
		if self.middle.len() < 50 {
			return;
		}

		let mut before = Self::new();
		let mut after = Self::new();
		self.middle.sort_by_key(|i| i.start + i.end);
		let split_interval = &self.middle[self.middle.len() / 2];
		self.split_time = (split_interval.start + split_interval.end) / 2;

		self.middle.retain(|i| {
			if i.end <= self.split_time {
				before.insert(*i);
				false
			} else if i.start >= self.split_time {
				after.insert(*i);
				false
			} else {
				true
			}
		});

		before.split();
		after.split();
		self.before = Some(Rc::new(before));
		self.after = Some(Rc::new(after));
	}

	pub fn query(&mut self, interval: JobInterval, output: &mut Vec<JobInterval>) {
		debug_assert_eq!(0, self.stack.len());

		if let Some(before) = &self.before {
			if interval.start < self.split_time {
				self.stack.push(Rc::clone(before));
			}
		}

		if let Some(after) = &self.after {
			if interval.end > self.split_time {
				self.stack.push(Rc::clone(after));
			}
		}

		for candidate in &self.middle {
			if candidate.start < interval.end && candidate.end > interval.start {
				output.push(*candidate);
			}
		}

		while let Some(current_node) = self.stack.pop() {
			if let Some(before) = &current_node.before {
				if interval.start < current_node.split_time {
					self.stack.push(Rc::clone(before));
				}
			}
			if let Some(after) = &current_node.after {
				if interval.end > current_node.split_time {
					self.stack.push(Rc::clone(after));
				}
			}
			for candidate in &current_node.middle {
				if candidate.start < interval.end && candidate.end > interval.start {
					output.push(*candidate);
				}
			}
		}
		self.stack.clear();
	}
}
