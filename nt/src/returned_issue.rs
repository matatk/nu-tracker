// TODO: Test that only required fields are requested
use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub trait ReturnedIssue {
	const GITHUB_FIELD_NAMES: &'static [&'static str];
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Assignee {
	pub id: String,
	pub is_bot: bool,
	pub login: String,
	pub r#type: String,
	pub url: String,
}

impl Display for Assignee {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.login)
	}
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Label {
	pub id: String,
	pub color: String,
	pub description: String,
	pub name: String,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
	pub name: String,
	pub name_with_owner: String,
}

#[cfg(test)]
mod tests {
	use std::assert_eq;

	use nt_macros::make_returned_issue;

	use super::ReturnedIssue;

	#[test]
	fn function() {
		struct _TestRequest {
			hail: String,
			details: String,
			answer: u8,
		}

		#[make_returned_issue]
		fn _test_function(issue: ReturnedTestIssue) -> _TestRequest {
			_TestRequest {
				hail: issue.title,
				details: issue.body,
				answer: 42,
			}
		}

		assert_eq!(ReturnedTestIssue::GITHUB_FIELD_NAMES, &["title", "body"]);
	}

	#[test]
	fn implementation() {
		struct _TestRequest {
			hail: String,
			details: String,
			people: Vec<String>,
			answer: u8,
		}

		#[make_returned_issue]
		impl _TestRequest {
			fn _from(issue: ReturnedTestIssue) -> Self {
				Self {
					hail: issue.title,
					details: issue.body,
					people: issue
						.assignees
						.iter()
						.map(|assignee| assignee.to_string())
						.collect(),
					answer: 42,
				}
			}
		}

		assert_eq!(
			ReturnedTestIssue::GITHUB_FIELD_NAMES,
			&["title", "body", "assignees"]
		);
	}
}
