use std::{fmt, marker::PhantomData, str::FromStr};

use super::LabelInfo;

type InternalList = Vec<String>;

#[derive(Clone)]
pub struct LabelStringVec<Validator: LabelInfo> {
	list: InternalList,
	check: PhantomData<Validator>,
}

impl<Validator: LabelInfo> LabelStringVec<Validator> {
	pub fn is_empty(&self) -> bool {
		self.list.is_empty()
	}
}

impl<Validator: LabelInfo> Default for LabelStringVec<Validator> {
	fn default() -> Self {
		LabelStringVec {
			list: Vec::new(),
			check: PhantomData,
		}
	}
}

impl<Validator: LabelInfo> FromStr for LabelStringVec<Validator> {
	type Err = ParseFlagError;

	fn from_str(abbreviated_labels: &str) -> Result<LabelStringVec<Validator>, ParseFlagError> {
		let full_status_names: Result<Vec<&str>, ParseFlagError> = abbreviated_labels
			.chars()
			.map(|l| {
				Ok(Validator::label_for(&l).ok_or_else(|| {
					format!("Valid flags:\n{}", Validator::flags_labels_conflicts())
				})?)
			})
			.collect();

		match full_status_names {
			Ok(names) => {
				// TODO: Can we eliminate this copying?
				let owned_names = names.iter().map(|n| n.to_string()).collect();
				Ok(LabelStringVec {
					list: owned_names,
					check: PhantomData,
				})
			}
			Err(error) => Err(error),
		}
	}
}

impl<Validator: LabelInfo> fmt::Display for LabelStringVec<Validator> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self.list)?;
		Ok(())
	}
}

impl<Validator: LabelInfo> IntoIterator for LabelStringVec<Validator> {
	type Item = <InternalList as IntoIterator>::Item;
	type IntoIter = <InternalList as IntoIterator>::IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		self.list.into_iter()
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
