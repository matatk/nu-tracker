use std::{
	collections::{HashMap, HashSet},
	error::Error,
	fmt, println,
	str::FromStr,
};

use crate::returned_issue::ReturnedIssueANTBRLA;
use crate::status_labels::CommentLabels;
use crate::ToVecStringWithFields;
use crate::{assignee_query::AssigneeQuery, fetch_sort_print_handler, ReportFormat};
use crate::{flatten_assignees::flatten_assignees, query::Query};
use crate::{generate_table::generate_table, status_labels::CommentStatus};

use super::{make_fields_and_request, make_print_table, make_source_label};

make_source_label!(Spec: prefix: "s");
make_source_label!(Group:
	prefix: "wg" "cg" "ig" "bg";
	whole: "whatwg"
);

make_fields_and_request!(
	Comment,
	"Comment review request fields",
	[
		assignees String | Assignees "Assigned users", 15;
			|me: &CommentReviewRequest| me.assignees.clone(),
		group Option<GroupLabel> | Group "The group the request is from/relates to", 11;
			|me: &CommentReviewRequest| {
				if let Some(group) = &me.group {
					group.to_string()
				} else {
					String::from("???")
				}
			},
		id u32 | Id "The tracking issue's number";
			|me: &CommentReviewRequest| me.id.to_string(),
		our bool | Our "Whether the issue comes from our group";
			|me: &CommentReviewRequest| {
				if me.our {
					String::from("Yes")
				} else {
					String::from(" - ")
				}
			},
		source String | Source "The source issue";
			|me: &CommentReviewRequest| me.source.clone(),
		spec Option<SpecLabel> | Spec "The spec the request relates to", 15;
			|me: &CommentReviewRequest| {
				if let Some(spec) = &me.spec {
					spec.to_string()
				} else {
					String::from("???")
				}
			},
		status CommentStatus | Status "The status of the request";
			|me: &CommentReviewRequest| format!("{}", me.status),
		title String | Title "The request's title";
			|me: &CommentReviewRequest| me.title.clone()
	],
	|issue: ReturnedIssueANTBRLA| {
		let mut group = None;
		let mut spec = None;
		let mut status: CommentStatus = CommentStatus::new();

		for label in issue.labels {
			let name = label.name.to_string();

			// TODO: More functional please.
			// TODO: Check for having already inserted?
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
			our: issue.author.to_string() != "w3cbot",
		}
	}
);

/// Query for issue comment requests; output a custom report.
pub fn comments(
	repo: &str,
	status: CommentLabels,
	not_status: CommentLabels,
	spec: Option<String>,
	assignee: AssigneeQuery,
	show_source_issue: bool,
	report_formats: &[ReportFormat],
	fields: &[CommentField],
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
		ReportFormat::Table => Box::new(|requests| print_table(spec.clone(), fields, show_source_issue, requests)),
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
