use std::{fmt, marker::PhantomData, str::FromStr};

use thiserror::Error;

use super::StatusLabelInfo;

type InternalList = Vec<String>;

#[derive(Clone)]
pub struct LabelStringVec<FromStrHelper: StatusLabelInfo> {
	list: InternalList,
	check: PhantomData<FromStrHelper>,
}

impl<FromStrHelper: StatusLabelInfo> LabelStringVec<FromStrHelper> {
	pub fn is_empty(&self) -> bool {
		self.list.is_empty()
	}
}

impl<FromStrHelper: StatusLabelInfo> Default for LabelStringVec<FromStrHelper> {
	fn default() -> Self {
		LabelStringVec {
			list: Vec::new(),
			check: PhantomData,
		}
	}
}

impl<FromStrHelper: StatusLabelInfo> FromStr for LabelStringVec<FromStrHelper> {
	type Err = ParseFlagError;

	fn from_str(abbreviated_labels: &str) -> Result<LabelStringVec<FromStrHelper>, ParseFlagError> {
		let full_status_names: Result<Vec<&str>, ParseFlagError> = abbreviated_labels
			.chars()
			.map(|l| {
				Ok(FromStrHelper::label_for(&l).ok_or_else(|| {
					format!("Valid flags:\n{}", FromStrHelper::flags_labels_conflicts())
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

impl<FromStrHelper: StatusLabelInfo> fmt::Display for LabelStringVec<FromStrHelper> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self.list)?;
		Ok(())
	}
}

impl<FromStrHelper: StatusLabelInfo> IntoIterator for LabelStringVec<FromStrHelper> {
	type Item = <InternalList as IntoIterator>::Item;
	type IntoIter = <InternalList as IntoIterator>::IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		self.list.into_iter()
	}
}

/// Indicates when an flag was given that doesn't correspond to any known status label for the type of issue at hand
#[derive(Error, Debug, PartialEq)]
#[error("{0}")]
pub struct ParseFlagError(String);

impl From<String> for ParseFlagError {
	fn from(message: String) -> Self {
		ParseFlagError(message)
	}
}
