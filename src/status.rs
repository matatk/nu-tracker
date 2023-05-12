use std::fmt;

#[macro_use]
mod label_and_map;
mod string_list;

pub use label_and_map::{FlagLabelMap, StatusLabel};
pub use string_list::{LabelStringList, ParseFlagError};

#[derive(Clone)]
pub struct Status(Vec<StatusLabel>);

impl Status {
	pub fn new() -> Self {
		Self(vec![])
	}

	pub fn add(&mut self, label: StatusLabel) {
		self.0.push(label);
	}
}

impl Default for Status {
	fn default() -> Self {
		Self::new()
	}
}

// TODO: allow choice of long/short - means this approach isn't the right one?
impl fmt::Display for Status {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut strings: Vec<String> = vec![];
		for label in &self.0 {
			strings.push(format!("{label}"));
		}
		write!(f, "{}", strings.join(" "))?;
		Ok(())
	}
}
