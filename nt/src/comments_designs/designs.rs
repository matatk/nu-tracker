use std::{
	collections::{HashMap, HashSet},
	error::Error,
	fmt, println,
	str::FromStr,
};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use strum_macros::AsRefStr;

use nt_macros::make_returned_issue;

use crate::assignee_query::AssigneeQuery;
use crate::flatten_assignees::flatten_assignees;
use crate::generate_table::generate_table;
use crate::query::Query;
use crate::returned_issue::ReturnedIssue;
use crate::status_labels::{Conflicts, DesignLabel, DesignStatus, Status};
use crate::{fetch_sort_print_handler, ReportFormat, ToVecStringWithFields};

use super::{make_print_table, make_source_label};

make_source_label!(Spec:
	prefix: "s";
	prefixs: "Topic"
);
make_source_label!(Group:
	prefix: "wg" "cg" "ig" "bg";
	prefixs: "Venue";
	whole: "whatwg"
);

/// Design review request fields
#[derive(AsRefStr, Clone, Debug, Deserialize, PartialEq, Serialize, ValueEnum)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DesignField {
	/// Assigned users
	Assignees,
	/// The group the request is from/relates to
	Group,
	/// The tracking issue's number
	Id,
	/// The source issue
	Source,
	/// The spec the request relates to
	Spec,
	/// The status of the request
	Status,
	/// The request's title
	Title,
}

struct DesignReviewRequest {
	assignees: String,
	group: Option<GroupLabel>,
	id: u32,
	source: String,
	spec: Option<SpecLabel>,
	status: DesignStatus,
	title: String,
}

#[make_returned_issue]
impl DesignReviewRequest {
	fn from(issue: DesignReturnedIssue) -> Self {
		let mut group = None;
		let mut spec = None;
		let mut status: DesignStatus = DesignStatus::new();
		for label in issue.labels {
			let name = label.name.to_string();
			if let Ok(gl) = GroupLabel::from_str(&name) {
				group = Some(gl)
			} else if let Ok(sl) = SpecLabel::from_str(&name) {
				spec = Some(sl)
			} else if group.is_none() && spec.is_none() {
				status.is(&name)
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
		}
	}

	fn max_field_width(field: &DesignField) -> Option<u16> {
		match field {
			DesignField::Assignees => Some(15),
			DesignField::Group => Some(11),
			DesignField::Spec => Some(15),
			_ => None,
		}
	}
}

impl ToVecStringWithFields for DesignReviewRequest {
	type Field = DesignField;

	fn to_vec_string(&self, fields: &[DesignField]) -> Vec<String> {
		let mut out: Vec<String> = Vec::new();

		for field in fields {
			out.push(match field {
				DesignField::Assignees => self.assignees.clone(),
				DesignField::Group => {
					if let Some(group) = &self.group {
						group.to_string()
					} else {
						String::from("???")
					}
				}
				DesignField::Id => self.id.to_string(),
				DesignField::Source => self.source.clone(),
				DesignField::Spec => {
					if let Some(spec) = &self.spec {
						spec.to_string()
					} else {
						String::from("???")
					}
				}
				DesignField::Status => format!("{}", self.status),
				DesignField::Title => self.title.clone(),
			})
		}

		out
	}
}

/// Query for design review requests; output a custom report.
pub fn designs(
	repo: &str,
	status: Vec<DesignLabel>,
	not_status: Vec<DesignLabel>,
	spec: Option<String>,
	assignee: AssigneeQuery,
	report_formats: &[ReportFormat],
	fields: &[DesignField],
	verbose: bool,
) -> Result<(), Box<dyn Error>> {
	let mut query = Query::new("Designs", verbose);

	if let Some(ref spec) = spec {
		query.label(format!("s:{}", spec));
	}

	query
		.labels(status)
		.not_labels(not_status)
		.repo(repo)
		.assignee(&assignee);

	let transmogrify = |issue: DesignReturnedIssue| Some(DesignReviewRequest::from(issue));

	fetch_sort_print_handler!("designs", query, transmogrify, report_formats, [{
		ReportFormat::Table => Box::new(|requests| print_table(spec.clone(), fields, requests)),
		ReportFormat::Agenda => todo!(),
		ReportFormat::Meeting => Box::new(|requests| print_meeting(repo, requests)),
	}]);
	Ok(())
}

make_print_table!(Design);

// FIXME: source issue isn't a link - can we ToString a Repository struct?
// TODO: include an option to print out the status too?
fn print_meeting(repo: &str, requests: &[DesignReviewRequest]) {
	println!("gb, off\n");
	for request in requests {
		println!(
			"subtopic: {}\nsource: {}\ntracking: https://github.com/{}/issues/{}\n",
			request.title, request.source, repo, request.id
		)
	}
	println!("gb, on")
}
