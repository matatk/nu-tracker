// TODO: DRY with comments?
use std::{error::Error, println, str};

use nt_macros::make_returned_issue;

use crate::generate_table::generate_table;
use crate::query::Query;
use crate::returned_issue::ReturnedIssue;
use crate::status_labels::{CharterLabel, CharterStatus, Status};
use crate::{fetch_sort_print_handler, ReportFormat, ToVecString};

struct CharterReviewRequest {
	title: String,
	tracking_number: u32,
	status: CharterStatus,
}

#[make_returned_issue]
impl CharterReviewRequest {
	// TODO: Make this (and likewise ones) trait-based too?
	fn from(issue: CharterReturnedIssue) -> Self
	where
		CharterReturnedIssue: ReturnedIssue,
	{
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
	status: Vec<CharterLabel>,
	not_status: Vec<CharterLabel>,
	report_formats: &[ReportFormat],
	verbose: bool,
) -> Result<(), Box<dyn Error>> {
	let mut query = Query::new("Charters", verbose);
	query
		.labels(["charter", "Horizontal review requested"])
		.labels(status)
		.not_labels(not_status)
		.repo(repo);

	let transmogrify = |issue: CharterReturnedIssue| Some(CharterReviewRequest::from(issue));

	fetch_sort_print_handler!("charters", query, transmogrify, report_formats, [{
		ReportFormat::Table => Box::new(print_table),
		ReportFormat::Agenda => todo!(),
		ReportFormat::Meeting => Box::new(|charters| print_meeting(repo, charters)),
	}]);
	Ok(())
}

fn print_table(requests: &[CharterReviewRequest]) {
	let table = generate_table(
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
