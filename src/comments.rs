use std::io::{self, Write};
use std::{
	collections::HashMap,
	fmt, println,
	process::Command,
	str::{self, FromStr},
};

use regex::Regex;

use crate::config::WorkingGroupInfo;
use crate::flatten_assignees::flatten_assignees;
use crate::make_table::make_table;
use crate::returned_issue::ReturnedIssueHeavy;
use crate::showing::showing;
use crate::status::{LabelStringList, Status, StatusLabel};

#[derive(Debug, PartialEq)]
struct SourceLabel {
	group: String,
}

#[derive(Debug, PartialEq)]
pub struct SourceLabelError;

impl FromStr for SourceLabel {
	type Err = SourceLabelError;

	/// Create a SourceLabel from a text string
	fn from_str(label_str: &str) -> Result<SourceLabel, SourceLabelError> {
		match label_str.split_once(':') {
			Some((prefix, group)) => {
				if prefix == "s" {
					Ok(SourceLabel {
						group: String::from(group),
					})
				} else {
					Err(SourceLabelError)
				}
			}
			None => Err(SourceLabelError),
		}
	}
}

impl fmt::Display for SourceLabel {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.group)
	}
}

struct CommentReviewRequest {
	source_label: Option<SourceLabel>,
	status: Status,
	source_issue: String, // TODO: Make it a Locator? Doesn't seem needed.
	title: String,
	tracking_assignees: String,
	tracking_number: u32,
}

impl CommentReviewRequest {
	fn from(issue: ReturnedIssueHeavy) -> CommentReviewRequest {
		let mut the_source_label: Option<SourceLabel> = None;
		let mut the_status: Status = Status::new();

		for label in issue.labels {
			let name = label.name.to_string();

			if let Ok(source_label) = SourceLabel::from_str(&name) {
				the_source_label = Some(source_label)
			} else if let Ok(status_label) = StatusLabel::from_str(&name) {
				the_status.add(status_label)
			}
		}

		// TODO: Check valid Status states here

		CommentReviewRequest {
			source_label: the_source_label,
			status: the_status,
			source_issue: get_source_issue_locator(&issue.body),
			title: issue.title,
			tracking_assignees: flatten_assignees(&issue.assignees),
			tracking_number: issue.number,
		}
	}

	fn to_vec_string(&self) -> Vec<String> {
		vec![
			self.tracking_number.to_string(),
			self.title.to_string(),
			if let Some(group) = &self.source_label {
				group.to_string()
			} else {
				String::from("UNKNOWN")
			},
			self.status.to_string(),
			self.tracking_assignees.to_string(),
			self.source_issue.to_string(),
		]
	}
}

// FIXME: DRY with actions, specs?
/// Query for issue comment requests; output a custom report.
pub fn comments(
	group_name: &str,
	repos: &WorkingGroupInfo,
	status: &LabelStringList,
	source: &bool,
	verbose: &bool,
) {
	if repos.horizontal_review.is_none() {
		println!("Group '{group_name}' is not a horizontal review group.");
		return;
	}

	let mut cmd = Command::new("gh");
	cmd.args(["search", "issues"])
		.args([
			"--repo",
			&repos.horizontal_review.as_ref().unwrap().comments,
		])
		.args(["--state", "open"])
		.args([
			"--json",
			&ReturnedIssueHeavy::FIELD_NAMES_AS_ARRAY.join(","),
		]);

	for label in status.clone() {
		// TODO: remove the need for clone?
		cmd.args(["--label", &label.to_string()]); // TODO: neaten / idiomatic
	}

	if *verbose {
		println!("Comment review: running: {cmd:?}");
	}
	let output = cmd.output().expect("'gh' should run");

	if output.status.success() {
		let out = str::from_utf8(&output.stdout).expect("got non-utf8 data from 'gh'");
		let reviews: Vec<ReturnedIssueHeavy> = serde_json::from_str(out).unwrap();

		// DRY with specs
		if reviews.is_empty() {
			// TODO: Make this neater a la .join() for the vec
			println!("No comment review requests found");
			return;
		} else {
			println!(
				"{} open review requests in {}\n",
				showing(reviews.len()),
				repos.horizontal_review.as_ref().unwrap().comments // FIXME: DRY with above (here and in specs)
			)
		}

		// TODO: more functional?
		let mut rows: Vec<Vec<String>> = vec![];
		for request in reviews {
			if *source {
				rows.push(CommentReviewRequest::from(request).to_vec_string());
			} else {
				let with_source = CommentReviewRequest::from(request).to_vec_string();
				let without_source = &with_source[0..with_source.len() - 1];
				rows.push(without_source.to_vec())
			}
		}

		let mut max_widths = HashMap::new();
		// FIXME: don't do either of these limitations if we don't need to.
		max_widths.insert(2, 15); // SPEC
		max_widths.insert(4, 15); // TRACKERS

		let table = if *source {
			make_table(
				vec!["ID", "TITLE", "SPEC", "STATUS", "TRACKERS", "ISSUE"],
				rows,
				Some(max_widths),
			)
		} else {
			make_table(
				vec!["ID", "TITLE", "SPEC", "STATUS", "TRACKERS"],
				rows,
				Some(max_widths),
			)
		};
		println!("{table}")
	} else {
		io::stdout().write_all(&output.stdout).unwrap();
		io::stderr().write_all(&output.stderr).unwrap();
		panic!("'gh' did not run successfully")
	}
}

// TODO: change to return result, because not having the link is an error?
fn get_source_issue_locator(body: &str) -> String {
	let re = Regex::new(r"§ https://github.com/(.+)/(.+)/.+/(\d+)").unwrap();

	if let Some(caps) = re.captures(body) {
		let owner = caps.get(1).unwrap().as_str();
		let repo = caps.get(2).unwrap().as_str();
		let number = caps.get(3).unwrap().as_str();
		return format!("{}/{}#{}", owner, repo, number);
	}

	String::from("UNKNOWN!")
}

#[cfg(test)]
mod tests_get_locator {
	use super::*;

	#[test]
	fn no_crash_if_no_dates() {
		assert_eq!(
			get_source_issue_locator("Invalid request"),
			String::from("UNKNOWN!")
		);
	}

	#[test]
	fn multiple_lines() {
		assert_eq!(
			get_source_issue_locator(
				"**This is a tracker issue.** Only discuss things here if they are a11y group internal meta-discussions about the issue. **Contribute to the actual discussion at the following link:**

§ https://github.com/openui/open-ui/issues/530"
			),
			String::from("openui/open-ui#530")
		);
	}

	#[test]
	fn multiple_lines_pr() {
		assert_eq!(
			get_source_issue_locator(
                "**This is a tracker issue.** Only discuss things here if they are a11y group internal meta-discussions about the issue. **Contribute to the actual discussion at the following link:**

§ https://github.com/whatwg/html/pull/8352"
			),
			String::from("whatwg/html#8352")
		);
	}
}

#[cfg(test)]
mod tests_source_label {
	use std::assert_eq;

	use super::*;

	// FIXME: test for status labels being invalid

	#[test]
	fn valid_source() {
		let result = SourceLabel::from_str("s:html").unwrap();
		assert_eq!(
			result,
			SourceLabel {
				group: String::from("html")
			}
		)
	}

	#[test]
	fn invalid_source() {
		let result = SourceLabel::from_str("noop:html");
		assert_eq!(result, Err(SourceLabelError))
	}
}
