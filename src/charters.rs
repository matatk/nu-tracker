// TODO: DRY with comments?
use std::{error::Error, println, str};

use crate::make_table::make_table;
use crate::query::Query;
use crate::returned_issue::ReturnedIssueHeavy;
use crate::status_labels::{CharterLabels, CharterStatus}; // FIXME: don't need to request repo, which is done as part of this

struct CharterReviewRequest {
	title: String,
	tracking_number: u32,
	status: CharterStatus,
}

impl CharterReviewRequest {
	fn from(issue: ReturnedIssueHeavy) -> Self {
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

	fn to_vec_string(&self) -> Vec<String> {
		vec![
			self.tracking_number.to_string(),
			self.title.to_string(),
			self.status.to_string(),
		]
	}
}

/// Query for FIXME; output a custom report.
pub fn charters(
	repo: &str,
	status: CharterLabels,
	not_status: CharterLabels,
	agenda: bool,
	web: bool,
	verbose: bool,
) -> Result<(), Box<dyn Error>> {
	let mut query = Query::new("Charters", verbose);

	query
		.labels(["charter", "Horizontal review requested"])
		.labels(status)
		.not_labels(not_status)
		.repo(repo);

	if web {
		query.run_direct(true);
		return Ok(());
	}

	let requests: Vec<CharterReviewRequest> = query
		.run(
			"charter review requests",
			ReturnedIssueHeavy::FIELD_NAMES_AS_ARRAY.to_vec(),
		)?
		.into_iter()
		.map(CharterReviewRequest::from)
		.collect();

	if agenda {
		print_agenda(repo, &requests);
	} else {
		print_table(&requests);
	}
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

fn print_agenda(repo: &str, requests: &[CharterReviewRequest]) {
	println!("gb, off\n");
	for request in requests {
		println!(
			"subtopic: {}\nhttps://github.com/{}/issues/{}\n",
			request.title, repo, request.tracking_number
		)
	}
	println!("gb, on")
}
