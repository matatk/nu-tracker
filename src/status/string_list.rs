use std::{fmt, str::FromStr};

use super::FlagLabelMap;

#[derive(Clone)]
pub struct LabelStringList(InternalList);
type InternalList = Vec<String>;

impl FromStr for LabelStringList {
	type Err = ParseFlagError;

	fn from_str(status: &str) -> Result<LabelStringList, ParseFlagError> {
		let map = FlagLabelMap::new();

		let full_status_names: Result<Vec<&String>, ParseFlagError> = status
			.chars()
			.map(|s| {
				Ok(map
					.get(&s.to_string())
					.ok_or(format!("unknown status label flag '{s}'"))?)
			})
			.collect();

		match full_status_names {
			Ok(names) => {
				// TODO: Can we eliminate this copying?
				let owned_names = names.iter().map(|n| n.to_string()).collect();
				Ok(LabelStringList(owned_names))
			}
			Err(error) => Err(error),
		}
	}
}

impl fmt::Display for LabelStringList {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self.0)?;
		Ok(())
	}
}

impl IntoIterator for LabelStringList {
	type Item = <InternalList as IntoIterator>::Item;
	type IntoIter = <InternalList as IntoIterator>::IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

#[derive(Debug, PartialEq)]
pub struct ParseFlagError(String);

impl ParseFlagError {
	pub fn new(message: String) -> Self {
		ParseFlagError(message)
	}
}

impl fmt::Display for ParseFlagError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.0)?;
		Ok(())
	}
}

impl std::error::Error for ParseFlagError {}

impl From<String> for ParseFlagError {
	fn from(message: String) -> Self {
		ParseFlagError(message)
	}
}
