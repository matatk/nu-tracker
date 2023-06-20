use std::{fmt, str::FromStr};

use super::{flags_labels_conflicts, label_for_flag};

type InternalList = Vec<String>;

#[derive(Clone)]
pub struct LabelStringVec(InternalList);

impl FromStr for LabelStringVec {
	type Err = ParseFlagError;

	fn from_str(status: &str) -> Result<LabelStringVec, ParseFlagError> {
		let full_status_names: Result<Vec<&str>, ParseFlagError> = status
			.chars()
			.map(|s| {
				Ok(label_for_flag(&s)
					.ok_or_else(|| format!("Valid flags:\n{}", flags_labels_conflicts()))?)
			})
			.collect();

		match full_status_names {
			Ok(names) => {
				// TODO: Can we eliminate this copying?
				let owned_names = names.iter().map(|n| n.to_string()).collect();
				Ok(LabelStringVec(owned_names))
			}
			Err(error) => Err(error),
		}
	}
}

impl fmt::Display for LabelStringVec {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self.0)?;
		Ok(())
	}
}

impl IntoIterator for LabelStringVec {
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
