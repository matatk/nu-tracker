use std::{
	collections::{HashMap, HashSet},
	fmt, println,
	str::{self, FromStr},
};

use regex::Regex;

use crate::assignee_query::AssigneeQuery;
use crate::flatten_assignees::flatten_assignees;
use crate::make_table::make_table;
use crate::query::Query;
use crate::returned_issue::ReturnedIssueHeavy;
use crate::status::{LabelStringVec, Status};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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
			} else {
				the_status.is(&name)
			}
		}

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

/// Query for issue comment requests; output a custom report.
pub fn comments(
	repo: &str,
	status: LabelStringVec,
	not_status: LabelStringVec,
	spec: Option<String>,
	assignee: AssigneeQuery,
	show_source_issue: bool,
	verbose: bool,
	web: bool,
) {
	let mut query = Query::new("Comments", verbose);

	if let Some(ref spec) = spec {
		query.label(format!("s:{}", spec));
	}

	query
		.labels(status)
		.not_labels(not_status)
		.repo(repo)
		.assignee(assignee);

	if web {
		query.run_direct(true);
		return;
	}

	let issues: Vec<ReturnedIssueHeavy> = query.run(
		"comment review requests",
		ReturnedIssueHeavy::FIELD_NAMES_AS_ARRAY.to_vec(),
	);

	// TODO: more functional?
	let mut rows: Vec<Vec<String>> = vec![];
	let mut invalid_reqs: Vec<Vec<String>> = vec![];
	let mut source_labels: HashSet<SourceLabel> = HashSet::new();

	for issue in issues {
		let request = CommentReviewRequest::from(issue);

		if spec.is_none() {
			if let Some(group) = &request.source_label {
				source_labels.insert(group.clone());
			}
		}

		if show_source_issue {
			rows.push(request.to_vec_string())
		} else {
			let with_source = request.to_vec_string();
			let without_source = &with_source[0..with_source.len() - 1];
			rows.push(without_source.to_vec())
		}

		if !request.status.is_valid() {
			invalid_reqs.push(vec![
				request.tracking_number.to_string(),
				request.title,
				format!("{}", request.status),
			])
		}
	}

	if !invalid_reqs.is_empty() {
		println!(
			"Requests with invalid statuses due to conflicting labels:\n\n{}\n",
			make_table(vec!["ID", "TITLE", "INVALID STATUS"], invalid_reqs, None)
		);
	}

	if !source_labels.is_empty() {
		let mut source_groups = source_labels
			.iter()
			.map(|s| format!("{s}"))
			.collect::<Vec<_>>();
		source_groups.sort();
		println!("Source groups: {}\n", source_groups.join(", "));
	}

	let mut max_widths = HashMap::new();
	// FIXME: don't do either of these limitations if we don't need to.
	max_widths.insert(2, 15); // SPEC
	max_widths.insert(4, 15); // TRACKERS

	let table = if show_source_issue {
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
