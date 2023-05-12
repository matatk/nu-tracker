use std::{fmt, str::FromStr};

/// Stores info required to locate a repo
///
/// This will usually be constructed via [`Locator::from_str`].
#[derive(Debug, PartialEq)]
pub struct Locator {
	owner: String,
	repo: String,
	issue: u32,
}

#[derive(Debug, PartialEq)]
pub struct LocatorError;

impl fmt::Display for Locator {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}/{}#{}", self.owner, self.repo, self.issue)
	}
}

impl FromStr for Locator {
	type Err = LocatorError;

	/// Create a Locator from a concise locator string, e.g. "w3c/apa#42"
	fn from_str(locator_str: &str) -> Result<Locator, LocatorError> {
		let slash_idx = locator_str.find('/');
		if slash_idx.is_none() {
			return Err(LocatorError);
		}
		let slash_idx = slash_idx.unwrap();

		let hash_idx = locator_str.find('#');
		if hash_idx.is_none() {
			return Err(LocatorError);
		}
		let hash_idx = hash_idx.unwrap();

		let owner = locator_str[0..slash_idx].to_string();
		if owner.is_empty() {
			return Err(LocatorError);
		}
		let repo = locator_str[slash_idx + 1..hash_idx].to_string();
		if repo.is_empty() {
			return Err(LocatorError);
		}

		let issue = locator_str[hash_idx + 1..].parse();
		if issue.is_err() {
			return Err(LocatorError);
		}
		let issue = issue.unwrap();

		if issue == 0 {
			return Err(LocatorError);
		}

		Ok(Locator { owner, repo, issue })
	}
}

impl Locator {
	/// Return the full HTTPS URL for the issue's page on GitHub.
	///
	/// **Note:** If this is actually a PR, GitHub will redirect the request.
	pub fn url(&self) -> String {
		format!(
			"https://github.com/{}/{}/issues/{}",
			self.owner, self.repo, self.issue
		)
	}
}

#[cfg(test)]
mod tests {
	use std::assert_eq;

	use super::*;

	#[test]
	fn valid() {
		let result = Locator::from_str("matatk/landmarks#1").unwrap();
		assert_eq!(
			result,
			Locator {
				owner: String::from("matatk"),
				repo: String::from("landmarks"),
				issue: 1,
			}
		)
	}

	#[test]
	fn no_slash() {
		let result = Locator::from_str("");
		assert_eq!(Err(LocatorError), result)
	}

	#[test]
	fn no_hash() {
		let result = Locator::from_str("/");
		assert_eq!(Err(LocatorError), result)
	}

	#[test]
	fn zero_length_owner() {
		let result = Locator::from_str("/#");
		assert_eq!(Err(LocatorError), result)
	}

	#[test]
	fn zero_length_repo() {
		let result = Locator::from_str("moo/#");
		assert_eq!(Err(LocatorError), result)
	}

	#[test]
	fn zero_length_issue() {
		let result = Locator::from_str("moo/moo#");
		assert_eq!(Err(LocatorError), result)
	}

	#[test]
	fn issue_is_zero() {
		let result = Locator::from_str("matatk/landmarks#0");
		assert_eq!(Err(LocatorError), result)
	}

	#[test]
	fn url() {
		let result = Locator::from_str("matatk/landmarks#1").unwrap().url();
		assert_eq!(
			result,
			String::from("https://github.com/matatk/landmarks/issues/1")
		)
	}
}
