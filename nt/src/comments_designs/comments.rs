use std::{
	collections::{HashMap, HashSet},
	error::Error,
	fmt, println,
};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use strum_macros::AsRefStr;

use nt_macros::make_returned_issue;

use crate::assignee_query::AssigneeQuery;
use crate::fetch_sort_print_handler;
use crate::flatten_assignees::flatten_assignees;
use crate::origin_query::OriginQuery;
use crate::query::Query;
use crate::returned_issue::ReturnedIssue;
use crate::status_labels::{CommentLabel, CommentStatus};
use crate::{ReportFormat, ToVecStringWithFields};

use super::{make_print_table, make_source_label, TRY_TITLE_COLUMN_WIDTH};

make_source_label!(Spec: prefix: "s");
make_source_label!(Group:
	prefix: "wg" "cg" "ig" "bg";
	whole: "whatwg"
);

/// Comment review request fields
#[derive(AsRefStr, Clone, Debug, Deserialize, PartialEq, Serialize, ValueEnum)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CommentField {
	/// Assigned users
	Assignees,
	/// The group the request is from/relates to
	Group,
	/// The tracking issue's number
	Id,
	/// Whether the issue comes from our group
	Our,
	/// The source issue
	Source,
	/// The spec the request relates to
	Spec,
	/// The status of the request
	Status,
	/// The request's title
	Title,
}

struct CommentReviewRequest {
	assignees: String,
	group: Option<GroupLabel>,
	id: u32,
	our: bool,
	source: String,
	spec: Option<SpecLabel>,
	status: CommentStatus,
	title: String,
}

#[make_returned_issue]
impl CommentReviewRequest {
	fn from(issue: CommentReturnedIssue) -> Self {
		let mut group = None;
		let mut spec = None;
		let mut status = CommentStatus::new();
		for label in issue.labels {
			let name = label.name.to_string();
			if let Ok(gl) = GroupLabel::try_from(&label) {
				group = Some(gl)
			} else if let Ok(sl) = SpecLabel::try_from(&label) {
				spec = Some(sl)
			} else if group.is_none() && spec.is_none() {
				status.is(&name, label.color.into())
			}
		}

		Self {
			group,
			spec,
			status,
			source: super::get_source_issue_locator(&issue.body),
			title: issue.title,
			assignees: flatten_assignees(&issue.assignees),
			id: issue.number,
			our: issue.author.to_string() != "w3cbot",
		}
	}

	fn max_field_width(field: &CommentField) -> Option<u16> {
		match field {
			CommentField::Assignees => Some(15),
			CommentField::Group => Some(11),
			CommentField::Spec => Some(15),
			_ => None,
		}
	}
}

impl ToVecStringWithFields for CommentReviewRequest {
	type Field = CommentField;

	fn to_vec_string(&self, fields: &[CommentField]) -> Vec<String> {
		let mut out: Vec<String> = Vec::new();

		for field in fields {
			out.push(match field {
				CommentField::Assignees => self.assignees.clone(),
				CommentField::Group => {
					if let Some(group) = &self.group {
						group.to_string()
					} else {
						String::from("???")
					}
				}
				CommentField::Id => self.id.to_string(),
				CommentField::Our => {
					if self.our {
						String::from("Yes")
					} else {
						String::from(" - ")
					}
				}
				CommentField::Source => self.source.clone(),
				CommentField::Spec => {
					if let Some(spec) = &self.spec {
						spec.to_string()
					} else {
						String::from("???")
					}
				}
				CommentField::Status => format!("{}", self.status),
				CommentField::Title => self.title.clone(),
			})
		}

		out
	}
}

/// Query for issue comment requests; output a custom report.
pub fn comments(
	repo: &str,
	status: Vec<CommentLabel>,
	not_status: Vec<CommentLabel>,
	spec: Option<String>,
	assignee: AssigneeQuery,
	report_formats: &[ReportFormat],
	fields: &[CommentField],
	from: OriginQuery,
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
		.assignee(&assignee)
		.origin(&from);

	let transmogrify = |issue: CommentReturnedIssue| Some(CommentReviewRequest::from(issue));

	fetch_sort_print_handler!("comments", query, transmogrify, report_formats, [{
		ReportFormat::Table => Box::new(|requests| print_table(spec.clone(), fields, requests)),
		ReportFormat::Agenda => todo!(),
		ReportFormat::Meeting => Box::new(|requests| print_meeting(repo, requests)),
	}]);
	Ok(())
}

make_print_table!(Comment);

// FIXME: source issue isn't a link - can we ToString a Repository struct?
// TODO: include an option to print out the status too?
fn print_meeting(repo: &str, requests: &[CommentReviewRequest]) {
	println!("gb, off\n");
	for request in requests {
		println!(
			"subtopic: {}\nsource: {}\ntracking: https://github.com/{}/issues/{}\n",
			request.title, request.source, repo, request.id
		)
	}
	println!("gb, on")
}
