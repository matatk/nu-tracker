// TODO: DRY with comments?
// FIXME: don't need to request repo, which is done as part of ReturnedIssueHeavy
use std::{error::Error, println, str};

use crate::make_table::make_table;
use crate::query::Query;
use crate::returned_issue::ReturnedIssueANTBRL;
use crate::status_labels::{CharterLabels, CharterStatus};
use crate::{fetch_sort_print_handler, ReportFormat, ToVecString};

struct CharterReviewRequest {
	title: String,
	tracking_number: u32,
	status: CharterStatus,
}

// TODO: Make this (and likewise ones) trait-based too?
impl CharterReviewRequest {
	fn from(issue: ReturnedIssueANTBRL) -> Self {
		let mut the_status: CharterStatus = CharterStatus::new();

		for label in issue.labels {
			let name = label.name.to_string();
			the_status.is(&name)
		}

		Self {
			title: issue.title,
			tracking_number: issue.number,
			status: the_status,
		}
	}
}

impl ToVecString for CharterReviewRequest {
	fn to_vec_string(&self) -> Vec<String> {
		vec![
			self.tracking_number.to_string(), // FIXME: into? (same for other to_vec_str()s)j
			self.title.clone(),
			self.status.to_string(),
		]
	}
}

/// Query for charter review requests (across all groups); output a custom report.
pub fn charters(
	repo: &str,
	status: CharterLabels,
	not_status: CharterLabels,
	report_formats: &[ReportFormat],
	verbose: bool,
) -> Result<(), Box<dyn Error>> {
	let mut query = Query::new("Charters", verbose);
	query
		.labels(["charter", "Horizontal review requested"])
		.labels(status)
		.not_labels(not_status)
		.repo(repo);

	let transmogrify = |issue: ReturnedIssueANTBRL| Some(CharterReviewRequest::from(issue));

	fetch_sort_print_handler!("charters", query, transmogrify, report_formats, [{
		ReportFormat::Table => Box::new(print_table),
		ReportFormat::Agenda => todo!(),
		ReportFormat::Meeting => Box::new(|charters| print_meeting(repo, charters)),
	}]);
	Ok(())
}

fn print_table(requests: &[CharterReviewRequest]) {
	let table = make_table(
		vec!["ID", "TITLE", "STATUS"],
		requests.iter().map(|r| r.to_vec_string()).collect(),
		None,
	);
	println!("{table}");
}

fn print_meeting(repo: &str, requests: &[CharterReviewRequest]) {
	println!("gb, off\n");
	for request in requests {
		println!(
			"subtopic: {}\nhttps://github.com/{}/issues/{}\n",
			request.title, repo, request.tracking_number
		)
	}
	println!("gb, on")
}
