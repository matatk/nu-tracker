// TODO: DRY gh invocation? / use a builder pattern?
// TODO: What to do if the given dest date is earlier than 21 days?
// TODO: What to do if the given start date doesn't match the issue title?
// FIXME: How to report ones that didn't parse?

use std::io::{self, Write};
use std::{println, process::Command, str};

use chrono::{Days, NaiveDate};
use regex::Regex;

use crate::assignee_query::AssigneeQuery;
use crate::config::WorkingGroupInfo;
use crate::flatten_assignees::flatten_assignees;
use crate::make_table::make_table;
use crate::returned_issue::ReturnedIssueLight;
use crate::showing::showing;

const DEFAULT_REVIEW_TIME: u64 = 21;

#[derive(Debug, PartialEq)]
struct SpecAndDue {
	spec: String,
	due: NaiveDate,
}

#[derive(Debug)]
struct ReviewRequest {
	spec: String,
	due: NaiveDate,
	number: u32,
	assignees: String,
}

impl ReviewRequest {
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

// TODO: DRY with actions, comments?
/// Query for spec review requests, output a custom report, sorted by due date.
pub fn specs(group_name: &str, repos: &WorkingGroupInfo, assignee: AssigneeQuery, verbose: &bool) {
	if repos.horizontal_review.is_none() {
		println!("Group '{group_name}' is not a horizontal review group.");
		return;
	}

	let mut cmd = Command::new("gh");
	cmd.args(["search", "issues"])
		.args(["--repo", &repos.horizontal_review.as_ref().unwrap().specs])
		.args(["--state", "open"])
		.args([
			"--json",
			&ReturnedIssueLight::FIELD_NAMES_AS_ARRAY.join(","),
		]);

	assignee.gh_args(&mut cmd);

	if *verbose {
		println!("Spec review: running: {cmd:?}");
	}
	let output = cmd.output().expect("'gh' should run");

	if output.status.success() {
		let out = str::from_utf8(&output.stdout).expect("got non-utf8 data from 'gh'");
		let reviews: Vec<ReturnedIssueLight> = serde_json::from_str(out).unwrap();

		// DRY with comments
		if reviews.is_empty() {
			// TODO: Make this neater a la .join() for the vec
			println!("No spec review requests found");
			return;
		} else {
			println!(
				"{} open review requests in {}\n",
				showing(reviews.len()),
				repos.horizontal_review.as_ref().unwrap().specs
			)
		}

		// TODO: idiomatic?
		let mut review_requests: Vec<ReviewRequest> = vec![];
		for issue_info in reviews {
			if let Some(review_request) = make_review_request(issue_info) {
				review_requests.push(review_request)
			} else {
				println!()
			}
		}
		review_requests.sort_by_key(|r| r.due);

		// TODO: more functional?
		let mut rows: Vec<Vec<String>> = vec![];
		for request in review_requests {
			rows.push(request.to_vec_string())
		}

		let table = make_table(vec!["DUE", "ID", "SPEC", "ASSIGNEES"], rows, None);
		println!("{table}")
	} else {
		io::stdout().write_all(&output.stdout).unwrap();
		io::stderr().write_all(&output.stderr).unwrap();
		panic!("'gh' did not run successfully")
	}
}

fn make_review_request(
	ReturnedIssueLight {
		assignees,
		number,
		title,
	}: ReturnedIssueLight,
) -> Option<ReviewRequest> {
	if let Some(SpecAndDue { spec, due }) = spec_and_due(title.as_str()) {
		return Some(ReviewRequest {
			spec,
			due,
			number,
			assignees: flatten_assignees(&assignees),
		});
	}

	println!("WARNING: Unable to identify due date for request #{number}: '{title}'",);
	None
}

fn spec_and_due(full_spec: &str) -> Option<SpecAndDue> {
	const DATE_FORMAT: &str = "%Y-%m-%d";
	let two_dates = Regex::new(r"(\d{4})-(\d{2})-(\d{2}) .?> (\d{4})-(\d{2})-(\d{2})$").unwrap();
	let single_date = Regex::new(r"(\d{4})-(\d{2})-(\d{2})$").unwrap();

	if let Some(two_date_match) = two_dates.find(full_spec) {
		if let Some(due_str) = single_date.find(full_spec) {
			if let Ok(due) = NaiveDate::parse_from_str(due_str.as_str(), DATE_FORMAT) {
				return Some(SpecAndDue {
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
			return Some(SpecAndDue {
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
			Some(SpecAndDue {
				spec: String::from("Verifiable Credential Data Integrity (and vc-di-eddsa and vc-di-ecdsa)"), 
				due: NaiveDate::from_ymd_opt(2023, 7, 31).unwrap()
			})
		);
	}

	#[test]
	fn two_dates_simple_arrow() {
		assert_eq!(
			spec_and_due("Digital Publishing WAI-ARIA Module 1.1 and Digital Publishing Accessibility API Mappings 1.1 2023-02-23 > 2023-04-01"), 
			Some(SpecAndDue {
				spec: String::from("Digital Publishing WAI-ARIA Module 1.1 and Digital Publishing Accessibility API Mappings 1.1"),
				due: NaiveDate::from_ymd_opt(2023, 4, 1).unwrap()
			})
		);
	}

	#[test]
	fn one_date() {
		assert_eq!(
			spec_and_due("CSS View Transitions 2022-11-20"),
			Some(SpecAndDue {
				spec: String::from("CSS View Transitions"),
				due: NaiveDate::from_ymd_opt(2022, 12, 11).unwrap()
			})
		);
	}

	#[test]
	fn two_dates_simple_arrow_extra_gap() {
		assert_eq!(
			spec_and_due("VISS 2 Core and Transport documents  2022-08-31 > 2022-09-30"),
			Some(SpecAndDue {
				spec: String::from("VISS 2 Core and Transport documents"),
				due: NaiveDate::from_ymd_opt(2022, 9, 30).unwrap()
			})
		);
	}
}
