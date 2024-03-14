use std::{
	collections::{HashMap, HashSet},
	error::Error,
	fmt, println,
	str::FromStr,
};

use crate::query::Query;
use crate::returned_issue::ReturnedIssueANTBRLA;
use crate::status_labels::{DesignLabels, DesignStatus};
use crate::ToVecStringWithFields;
use crate::{assignee_query::AssigneeQuery, fetch_sort_print_handler, ReportFormat};
use crate::{flatten_assignees::flatten_assignees, make_table::make_table};

use super::{make_fields_and_request, make_source_label};

make_source_label!(Spec:
	prefix: "s";
	prefixs: "Topic"
);
make_source_label!(Group:
	prefix: "wg" "cg" "ig" "bg";
	prefixs: "Venue";
	whole: "whatwg"
);

make_fields_and_request!(
	Design,
	"Design review request fields",
	[
		assignees String | Assignees "Assigned users", 15;
			|me: &DesignReviewRequest| me.assignees.clone(),
		group Option<GroupLabel> | Group "The group the request is from/relates to", 11;
			|me: &DesignReviewRequest| {
				if let Some(group) = &me.group {
					group.to_string()
				} else {
					String::from("???")
				}
			},
		id u32 | Id "The tracking issue's number";
			|me: &DesignReviewRequest| me.id.to_string(),
		source String | Source "The source issue";
			|me: &DesignReviewRequest| me.source.clone(),
		spec Option<SpecLabel> | Spec "The spec the request relates to", 15;
			|me: &DesignReviewRequest| {
				if let Some(spec) = &me.spec {
					spec.to_string()
				} else {
					String::from("???")
				}
			},
		status DesignStatus | Status "The status of the request";
			|me: &DesignReviewRequest| format!("{}", me.status),
		title String | Title "The request's title";
			|me: &DesignReviewRequest| me.title.clone()
	],
	|issue: ReturnedIssueANTBRLA| {
		let mut group = None;
		let mut spec = None;
		let mut status: DesignStatus = DesignStatus::new();

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
		}
	}
);

/// Query for design review requests; output a custom report.
pub fn designs(
	repo: &str,
	status: DesignLabels,
	not_status: DesignLabels,
	spec: Option<String>,
	assignee: AssigneeQuery,
	show_source_issue: bool,
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

	let transmogrify = |issue: ReturnedIssueANTBRLA| Some(DesignReviewRequest::from(issue));

	fetch_sort_print_handler!("designs", query, transmogrify, report_formats, [{
		ReportFormat::Table => Box::new(|requests| print_table(spec.clone(), fields, show_source_issue, requests)),
		ReportFormat::Agenda => todo!(),
		ReportFormat::Meeting => Box::new(|requests| print_meeting(repo, requests)),
	}]);
	Ok(())
}

fn print_table(
	spec: Option<String>,
	fields: &[DesignField],
	show_source_issue: bool,
	requests: &[DesignReviewRequest],
) {
	// TODO: more functional?
	let mut rows = vec![];
	let mut invalid_reqs = vec![];
	let mut group_labels: HashSet<GroupLabel> = HashSet::new();
	let mut spec_labels: HashSet<SpecLabel> = HashSet::new();

	let mut headers = Vec::from(fields);

	if show_source_issue && !fields.contains(&DesignField::Source) {
		headers.push(DesignField::Source);
	}

	for request in requests {
		if spec.is_none() {
			if let Some(label) = &request.spec {
				spec_labels.insert(label.clone());
			}
		}

		if let Some(label) = &request.group {
			group_labels.insert(label.clone());
		}

		// FIXME: shouldn't need to clone
		rows.push(request.to_vec_string(&headers));

		if !request.status.is_valid() {
			invalid_reqs.push(vec![
				request.id.to_string(),
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

	fn list_domains<T: fmt::Display>(pretty: &str, labels: HashSet<T>) {
		if !labels.is_empty() {
			let mut domains = labels.iter().map(|s| format!("{s}")).collect::<Vec<_>>();
			domains.sort();
			println!("{pretty}: {}\n", domains.join(", "));
		}
	}

	list_domains("Groups", group_labels);
	list_domains("Specs", spec_labels);

	let mut max_widths = HashMap::new();
	for (i, header) in headers.iter().enumerate() {
		if let Some(max_width) = DesignReviewRequest::max_field_width(header) {
			max_widths.insert(i, max_width);
		}
	}

	let table = make_table(
		headers.iter().map(|h| h.as_ref().to_uppercase()).collect(),
		rows,
		Some(max_widths),
	);
	println!("{table}")
}

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
