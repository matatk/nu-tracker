use std::{
	collections::{HashMap, HashSet},
	error::Error,
	fmt::{self, Display},
	println,
	str::FromStr,
};

use paste::paste;
use regex::Regex;

use crate::flatten_assignees::flatten_assignees;
use crate::make_table::make_table;
use crate::query::Query;
use crate::returned_issue::ReturnedIssueANTBRLA;
use crate::status_labels::{CommentLabels, CommentStatus};
use crate::{assignee_query::AssigneeQuery, fetch_sort_print_handler, ReportFormat};

// FIXME: support whatwg (on its own)
// FIXME: make it optional at print time whether we include the prefix? (not for s:* but for wg:*)
macro_rules! make_source_label {
	($name:ident: $($prefix:expr)+) => {
		paste! {
			#[derive(Debug, PartialEq, Eq, Hash, Clone)]
			struct [<$name Label>](String);

			#[derive(Debug, PartialEq)]
			pub struct [<$name LabelError>];

			impl FromStr for [<$name Label>] {
				type Err = [<$name LabelError>];

				/// Create a SourceLabel from a text string
				fn from_str(label_str: &str) -> Result<[<$name Label>], [<$name LabelError>]> {
					match label_str.split_once(':') {
						Some((prefix, group)) => {
							$(
								if prefix == $prefix {
									return Ok([<$name Label>](group.into()))
								}
							)+
							Err([<$name LabelError>])
						}
						None => Err([<$name LabelError>]),
					}
				}
			}

			impl fmt::Display for [<$name Label>] {
				fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
					write!(f, "{}", self.0)
				}
			}
		}
	};
}

make_source_label!(Spec: "s");
make_source_label!(Group: "wg" "cg" "ig" "bg");

struct CommentReviewRequest {
	group_label: Option<GroupLabel>,
	spec_label: Option<SpecLabel>,
	status: CommentStatus,
	source_issue: String, // TODO: Make it a Locator? Doesn't seem needed.
	title: String,
	tracking_assignees: String,
	tracking_number: u32,
	raised_by_us: bool,
}

impl CommentReviewRequest {
	fn from(issue: ReturnedIssueANTBRLA) -> CommentReviewRequest {
		let mut group_label = None;
		let mut spec_label = None;
		let mut the_status: CommentStatus = CommentStatus::new();

		for label in issue.labels {
			let name = label.name.to_string();

			// TODO: More functional please.
			// TODO: Check for having already inserted?
			if let Ok(gl) = GroupLabel::from_str(&name) {
				group_label = Some(gl)
			} else if let Ok(sl) = SpecLabel::from_str(&name) {
				spec_label = Some(sl)
			} else if group_label.is_none() && spec_label.is_none() {
				the_status.is(&name)
			}
		}

		CommentReviewRequest {
			group_label,
			spec_label,
			status: the_status,
			source_issue: get_source_issue_locator(&issue.body),
			title: issue.title,
			tracking_assignees: flatten_assignees(&issue.assignees),
			tracking_number: issue.number,
			raised_by_us: issue.author.to_string() != "w3cbot",
		}
	}

	fn to_vec_string(&self) -> Vec<String> {
		vec![
			self.tracking_number.to_string(),
			self.title.to_string(),
			if let Some(group) = &self.group_label {
				group.to_string()
			} else {
				String::from("???")
			},
			if let Some(spec) = &self.spec_label {
				spec.to_string()
			} else {
				String::from("???")
			},
			self.status.to_string(),
			self.tracking_assignees.to_string(),
			if self.raised_by_us {
				String::from("X")
			} else {
				String::from("-")
			},
			self.source_issue.to_string(),
		]
	}
}

/// Query for issue comment requests; output a custom report.
pub fn comments(
	repo: &str,
	status: CommentLabels,
	not_status: CommentLabels,
	spec: Option<String>,
	assignee: AssigneeQuery,
	show_source_issue: bool,
	report_formats: &[ReportFormat],
	verbose: bool,
) -> Result<(), Box<dyn Error>> {
	let mut query = Query::new("Comments", verbose);

	if let Some(ref spec) = spec {
		query.label(format!("s:{}", spec));
	}

	query
		.labels(status)
		.not_labels(not_status)
		.repo(repo)
		.assignee(&assignee);

	let transmogrify = |issue: ReturnedIssueANTBRLA| Some(CommentReviewRequest::from(issue));

	fetch_sort_print_handler!("comments", query, transmogrify, report_formats, [{
		ReportFormat::Table => Box::new(|requests| print_table(spec.clone(), show_source_issue, requests)),
		ReportFormat::Agenda => todo!(),
		ReportFormat::Meeting => Box::new(|requests| print_meeting(repo, requests)),
	}]);
	Ok(())
}

fn print_table(spec: Option<String>, show_source_issue: bool, requests: &[CommentReviewRequest]) {
	// TODO: more functional?
	let mut rows: Vec<Vec<String>> = vec![];
	let mut invalid_reqs: Vec<Vec<String>> = vec![];
	let mut group_labels: HashSet<GroupLabel> = HashSet::new();
	let mut spec_labels: HashSet<SpecLabel> = HashSet::new();

	for request in requests {
		if spec.is_none() {
			if let Some(label) = &request.spec_label {
				spec_labels.insert(label.clone());
			}
		}

		if let Some(label) = &request.group_label {
			group_labels.insert(label.clone());
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
				request.title.clone(),
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

	fn list_domains<T: Display>(pretty: &str, labels: HashSet<T>) {
		if !labels.is_empty() {
			let mut domains = labels.iter().map(|s| format!("{s}")).collect::<Vec<_>>();
			domains.sort();
			println!("{pretty}: {}\n", domains.join(", "));
		}
	}

	list_domains("Groups", group_labels);
	list_domains("Specs", spec_labels);

	let mut max_widths = HashMap::new();
	// FIXME: Don't do these limitations if we don't need to.
	max_widths.insert(2, 11); // GROUP
	max_widths.insert(3, 15); // SPEC
	max_widths.insert(5, 15); // TRACKERS

	let table = if show_source_issue {
		make_table(
			vec![
				"ID", "TITLE", "GROUP", "SPEC", "STATUS", "TRACKERS", "O", "ISSUE",
			],
			rows,
			Some(max_widths),
		)
	} else {
		make_table(
			vec!["ID", "TITLE", "GROUP", "SPEC", "STATUS", "TRACKERS", "O"],
			rows,
			Some(max_widths),
		)
	};
	println!("{table}")
}

// FIXME: source issue isn't a link - can we ToString a Repository struct?
// TODO: include an option to print out the status too?
fn print_meeting(repo: &str, requests: &[CommentReviewRequest]) {
	println!("gb, off\n");
	for request in requests {
		println!(
			"subtopic: {}\nsource: {}\ntracking: https://github.com/{}/issues/{}\n",
			request.title, request.source_issue, repo, request.tracking_number
		)
	}
	println!("gb, on")
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
mod tests_spec_label {
	use std::assert_eq;

	use super::*;

	// FIXME: test for status labels being invalid

	#[test]
	fn valid_source() {
		let result = SpecLabel::from_str("s:html").unwrap();
		assert_eq!(result, SpecLabel(String::from("html")))
	}

	#[test]
	fn valid_source_multiple() {
		let result = GroupLabel::from_str("wg:apa").unwrap();
		assert_eq!(result, GroupLabel(String::from("apa")))
	}

	#[test]
	fn invalid_source() {
		let result = SpecLabel::from_str("noop:html");
		assert_eq!(result, Err(SpecLabelError))
	}
}
