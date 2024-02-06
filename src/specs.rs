// TODO: What to do if the given dest date is earlier than 21 days?
// TODO: What to do if the given start date doesn't match the issue title?
// FIXME: How to report ones that didn't parse?
use std::{error::Error, println, str};

use chrono::{Days, NaiveDate};
use regex::Regex;

use crate::assignee_query::AssigneeQuery;
use crate::flatten_assignees::flatten_assignees;
use crate::make_table::make_table;
use crate::query::Query;
use crate::returned_issue::ReturnedIssueLight;

const DEFAULT_REVIEW_TIME: u64 = 21;

#[derive(Debug, PartialEq)]
struct SpecTitleAndDueDate {
	spec: String,
	due: NaiveDate,
}

#[derive(Debug)]
struct SpecReviewRequest {
	spec: String,
	due: NaiveDate,
	number: u32,
	assignees: String,
}

impl SpecReviewRequest {
	// TODO: Make trait?
	fn to_vec_string(&self) -> Vec<String> {
		vec![
			format!("{}", self.due),
			self.number.to_string(),
			self.spec.to_string(),
			self.assignees.to_string(),
		]
	}
}

/// Query for spec review requests, output a custom report, sorted by due date.
pub fn specs(
	repo: &str,
	assignee: AssigneeQuery,
	agenda: bool,
	web: bool,
	verbose: bool,
) -> Result<(), Box<dyn Error>> {
	let mut start = Query::new("Specs", verbose);
	// TODO: Neaten?
	let query = start.repo(repo).assignee(assignee);

	if web {
		query.run_direct(true);
		return Ok(());
	}

	// TODO: why does this not need type annotation, and actions does?
	let mut requests: Vec<SpecReviewRequest> = query
		.run(
			"spec review requests",
			ReturnedIssueLight::FIELD_NAMES_AS_ARRAY.to_vec(),
		)?
		.into_iter()
		.filter_map(make_review_request)
		.collect();

	requests.sort_by_key(|r| r.due);

	if agenda {
		print_agenda(repo, &requests);
	} else {
		print_table(&requests);
	}
	Ok(())
}

// TODO: DRY with charters?
fn print_table(specs: &[SpecReviewRequest]) {
	let table = make_table(
		vec!["DUE", "ID", "SPEC", "ASSIGNEES"],
		specs.iter().map(|r| r.to_vec_string()).collect(),
		None,
	);
	println!("{table}");
}

// TODO: Include assignees? If so, make a type for assignees.
fn print_agenda(repo: &str, specs: &[SpecReviewRequest]) {
	println!("gb, off");
	for request in specs {
		println!(
			"subtopic: {}\nhttps://github.com/{}/issues/{}\nDue: {}\n",
			request.spec, repo, request.number, request.due,
		)
	}
	println!("gb, on");
}

// FIXME: Return a Result
fn make_review_request(
	ReturnedIssueLight {
		assignees,
		number,
		title,
	}: ReturnedIssueLight,
) -> Option<SpecReviewRequest> {
	if let Some(SpecTitleAndDueDate { spec, due }) = spec_and_due(title.as_str()) {
		return Some(SpecReviewRequest {
			spec,
			due,
			number,
			assignees: flatten_assignees(&assignees),
		});
	}

	println!("WARNING: Unable to identify due date for request #{number}: '{title}'",);
	None
}

fn spec_and_due(full_spec: &str) -> Option<SpecTitleAndDueDate> {
	const DATE_FORMAT: &str = "%Y-%m-%d";
	let two_dates = Regex::new(r"(\d{4})-(\d{2})-(\d{2}) .?> (\d{4})-(\d{2})-(\d{2})$").unwrap();
	let single_date = Regex::new(r"(\d{4})-(\d{2})-(\d{2})$").unwrap();

	if let Some(two_date_match) = two_dates.find(full_spec) {
		if let Some(due_str) = single_date.find(full_spec) {
			if let Ok(due) = NaiveDate::parse_from_str(due_str.as_str(), DATE_FORMAT) {
				return Some(SpecTitleAndDueDate {
					spec: full_spec[0..two_date_match.start()].trim_end().to_string(),
					due,
				});
			} else {
				None
			}
		} else {
			None
		}
	} else if let Some(filed) = single_date.find(full_spec) {
		if let Ok(filed_date) = NaiveDate::parse_from_str(filed.as_str(), DATE_FORMAT) {
			return Some(SpecTitleAndDueDate {
				spec: full_spec[0..filed.start()].trim_end().to_string(),
				due: filed_date + Days::new(DEFAULT_REVIEW_TIME),
			});
		} else {
			None
		}
	} else {
		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn no_crash_if_no_dates() {
		assert_eq!(spec_and_due("Invalid request"), None);
	}

	#[test]
	fn two_dates_full_arrow() {
		assert_eq!(
			spec_and_due("Verifiable Credential Data Integrity (and vc-di-eddsa and vc-di-ecdsa) 2023-05-27 -> 2023-07-31"), 
			Some(SpecTitleAndDueDate {
				spec: String::from("Verifiable Credential Data Integrity (and vc-di-eddsa and vc-di-ecdsa)"), 
				due: NaiveDate::from_ymd_opt(2023, 7, 31).unwrap()
			})
		);
	}

	#[test]
	fn two_dates_simple_arrow() {
		assert_eq!(
			spec_and_due("Digital Publishing WAI-ARIA Module 1.1 and Digital Publishing Accessibility API Mappings 1.1 2023-02-23 > 2023-04-01"), 
			Some(SpecTitleAndDueDate {
				spec: String::from("Digital Publishing WAI-ARIA Module 1.1 and Digital Publishing Accessibility API Mappings 1.1"),
				due: NaiveDate::from_ymd_opt(2023, 4, 1).unwrap()
			})
		);
	}

	#[test]
	fn one_date() {
		assert_eq!(
			spec_and_due("CSS View Transitions 2022-11-20"),
			Some(SpecTitleAndDueDate {
				spec: String::from("CSS View Transitions"),
				due: NaiveDate::from_ymd_opt(2022, 12, 11).unwrap()
			})
		);
	}

	#[test]
	fn two_dates_simple_arrow_extra_gap() {
		assert_eq!(
			spec_and_due("VISS 2 Core and Transport documents  2022-08-31 > 2022-09-30"),
			Some(SpecTitleAndDueDate {
				spec: String::from("VISS 2 Core and Transport documents"),
				due: NaiveDate::from_ymd_opt(2022, 9, 30).unwrap()
			})
		);
	}
}
