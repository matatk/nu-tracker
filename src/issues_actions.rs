use std::error::Error;

use chrono::NaiveDate;
use regex::Regex;
use thiserror::Error;

use crate::assignee_query::AssigneeQuery;
use crate::config::{GroupRepos, MainAndOtherRepos};
use crate::flatten_assignees::flatten_assignees;
use crate::generate_table::generate_table;
use crate::query::Query;
use crate::returned_issue::ReturnedIssueANTBR;
use crate::{fetch_sort_print_handler, ReportFormat, ToVecString};

#[derive(Error, Debug)]
pub enum GetReposError {
	#[error("No repos selected")]
	NoneSelected,
	#[error("This group has no task forces")]
	NoTaskForces,
	// TODO: Include group name in here
	#[error("Unknown TF '{}'. Please consider contributing an update to the info for this TF's group. Known TFs for this group are: {}", .task_force, .group_task_forces.iter().map(|tf| format!("'{tf}'")).collect::<Vec<String>>().join(", "))]
	UnknownTaskForce {
		task_force: String,
		group_task_forces: Vec<String>,
	},
}

struct Action {
	issue: ReturnedIssueANTBR,
	due: Option<NaiveDate>,
}

impl ToVecString for Action {
	fn to_vec_string(&self) -> Vec<String> {
		vec![
			match self.due {
				Some(date) => format!("{date}"),
				None => String::from("(no date)"),
			},
			format!(
				"{}#{}",
				self.issue.repository.name_with_owner, self.issue.number
			),
			self.issue.title.clone(),
			flatten_assignees(&self.issue.assignees),
		]
	}
}

/// Query for issues in given repos; have `gh` print the output.
pub fn issues(
	repos: Vec<&str>,
	assignee: AssigneeQuery,
	labels: Vec<String>,
	closed: bool,
	actions: bool,
	report_formats: &[ReportFormat],
	verbose: bool,
) -> Result<(), Box<dyn Error>> {
	let mut include_actions = actions;
	let mut query = Query::new("Issues", verbose);

	for label in labels {
		query.label(&label); // TODO: idiomatic?
		if label == "action" {
			include_actions = true;
		}
	}

	if !include_actions {
		query.not_label("action");
	}

	query
		.repos(repos)
		.include_closed(closed)
		.assignee(&assignee);

	for format in report_formats {
		match format {
			ReportFormat::Gh => query.run_gh(false),
			ReportFormat::Table => query.run_gh(false),
			ReportFormat::Meeting => todo!(),
			ReportFormat::Agenda => todo!(),
			ReportFormat::Web => query.run_gh(true),
		}
	}
	Ok(())
}

// FIXME: if including closed, show status column
/// Query for action issues in given repos; make a custom report, sorted by due date.
pub fn actions(
	repos: Vec<&str>,
	assignee: AssigneeQuery,
	labels: Vec<String>,
	closed: bool,
	report_formats: &[ReportFormat],
	verbose: bool,
) -> Result<(), Box<dyn Error>> {
	let mut query = Query::new("Actions", verbose);
	query
		.repos(repos)
		.assignee(&assignee)
		.labels(labels)
		.label("action")
		.include_closed(closed);

	let transmogrify = |issue: ReturnedIssueANTBR| {
		Some(Action {
			issue: issue.clone(),
			due: get_due(&issue.body),
		})
	};
	let key = |action: &Action| action.due;

	fetch_sort_print_handler!("actions", query, transmogrify, report_formats, key, [{
		ReportFormat::Table => Box::new(print_table),
		ReportFormat::Agenda => todo!(),
		ReportFormat::Meeting =>  Box::new(print_meeting),
	}]);
	Ok(())
}

// TODO: DRY with specs?
fn print_table(actions: &[Action]) {
	let table = generate_table(
		vec!["DUE", "LOCATOR", "TITLE", "ASSIGNEES"],
		actions.iter().map(|a| a.to_vec_string()).collect(),
		None,
	);
	println!("{table}")
}

fn print_meeting(actions: &[Action]) {
	println!("gb, off\n");
	for action in actions {
		println!(
			"subtopic: {}\nhttps://github.com/{}/issues/{}\nDue: {}\nAssignees: {}\n",
			action.issue.title,
			action.issue.repository.name_with_owner,
			action.issue.number,
			action.due.unwrap_or_default(),
			flatten_assignees(&action.issue.assignees),
		)
	}
	println!("gb, on")
}

/// Find the relevant repos for this search
///
/// Based on the current group and the scope the user wishes to apply to the search.
pub fn get_repos<'a>(
	group_repos: &'a GroupRepos,
	main_only: &bool,
	include_group: &bool,
	include_tfs: &'a Option<Vec<String>>,
) -> Result<Vec<&'a str>, GetReposError> {
	let mut query_repos: Vec<&str> = Vec::new();

	if *include_group {
		add_repos(&mut query_repos, main_only, &group_repos.group)
	}

	if let Some(tfs) = include_tfs {
		if let Some(group_tfs) = &group_repos.task_forces {
			if tfs.is_empty() {
				for tf_repos in group_tfs.values() {
					add_repos(&mut query_repos, main_only, tf_repos)
				}
			} else {
				for task_force in tfs {
					if let Some(team_repos) = group_tfs.get(task_force) {
						add_repos(&mut query_repos, main_only, team_repos)
					} else {
						return Err(GetReposError::UnknownTaskForce {
							task_force: task_force.clone(),
							group_task_forces: group_tfs.keys().cloned().collect(),
						});
					}
				}
			}
		} else {
			return Err(GetReposError::NoTaskForces);
		}
	}

	if query_repos.is_empty() {
		// NOTE: This should not happen because the CLI UI has at least one of these arguments as
		//       required (however, there could be a UI layer bug :-)).
		return Err(GetReposError::NoneSelected);
	}

	Ok(query_repos)
}

fn add_repos<'a>(dest: &mut Vec<&'a str>, main: &bool, team_repos: &'a MainAndOtherRepos) {
	dest.push(&team_repos.main);
	if !main {
		// TODO: chain
		if let Some(others) = &team_repos.others {
			for other in others {
				dest.push(other);
			}
		}
	}
}

// Info on date formats: https://github.com/w3c/GHURLBot/issues/5
fn get_due(text: &str) -> Option<NaiveDate> {
	let cur = Regex::new(
		r"^(?i:due):[[:space:]]+(\d{4}-\d{2}-\d{2})(?:[[:space:]]+\(.+\))?.?[[:space:]]*$",
	)
	.unwrap();
	let pre = Regex::new(r"^due  ?(\d\d? [[:alpha:]]{3} \d{4})$").unwrap();

	for line in text.lines() {
		if let Some(caps) = cur.captures(line) {
			let date_text = caps.get(1).unwrap().as_str();
			return NaiveDate::parse_from_str(date_text, "%Y-%m-%d").ok();
		} else if let Some(caps) = pre.captures(line) {
			let date_text = caps.get(1).unwrap().as_str();
			return NaiveDate::parse_from_str(date_text, "%d %b %Y").ok();
		}
	}

	None
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn no_crash_if_no_dates() {
		assert_eq!(get_due("Invalid request"), None);
	}

	#[test]
	fn simple() {
		assert_eq!(
			get_due("Due: 2027-05-23"),
			Some(NaiveDate::from_ymd_opt(2027, 5, 23).unwrap())
		);
	}

	#[test]
	fn simple_case_space_dot() {
		assert_eq!(
			get_due("dUe:  2027-05-23."),
			Some(NaiveDate::from_ymd_opt(2027, 5, 23).unwrap())
		);
	}

	#[test]
	fn simple_comment_dot_space() {
		assert_eq!(
			get_due("Due: 2027-05-23 (Saturday the 42nd of Septembruary).  "),
			Some(NaiveDate::from_ymd_opt(2027, 5, 23).unwrap())
		);
	}

	#[test]
	fn multiple_lines_last() {
		assert_eq!(
			get_due("Description of the action\n\nDue: 2027-05-23"),
			Some(NaiveDate::from_ymd_opt(2027, 5, 23).unwrap())
		);
	}

	#[test]
	fn multiple_lines_first_takes_precedence() {
		assert_eq!(
			get_due("Due: 2027-05-24\n\nHere's some more info...\n\nDue: 2027-05-23"),
			Some(NaiveDate::from_ymd_opt(2027, 5, 24).unwrap())
		);
	}

	#[test]
	fn date_may_not_be_on_first_line() {
		assert_eq!(
			get_due(
				r#"Opened by matatk via IRC channel #apa on irc.w3.org

Due: 2023-12-06 (Wednesday  6 December)

Background: our meta-issue on this horizontal review query: w3c/a11y-review#138

[As discussed during today's call](https://www.w3.org/2023/11/22-apa-minutes#t06), there are potentially three ways we may engage with the APG team (TBD following an initial review of their issues):

1. Requesting the APG *dialog patterns to mirror the apparent emerging consensus that the browser chrome should be reachable in the focus order.

2. Requesting the APG to use `inert` (separate issue, but worth making a link betwixt them?)

3. Asking the APG what the policy is on "widely supported" and when updates may be made to reflect widely supported techniques. (Seems that such an issue will have been discussed; we'll need to find it.)"#
			),
			Some(NaiveDate::from_ymd_opt(2023, 12, 6).unwrap())
		);
	}
}

#[cfg(test)]
mod tests_legacy_format {
	use super::*;

	#[test]
	fn no_padding() {
		assert_eq!(
			get_due("due 23 May 2027"),
			Some(NaiveDate::from_ymd_opt(2027, 5, 23).unwrap())
		);
	}

	#[test]
	fn with_padding() {
		assert_eq!(
			get_due("due  4 Jun 2028"),
			Some(NaiveDate::from_ymd_opt(2028, 6, 4).unwrap())
		);
	}

	#[test]
	fn multiple_lines() {
		assert_eq!(
			get_due("due 23 May 2027\n\nHere's some more info..."),
			Some(NaiveDate::from_ymd_opt(2027, 5, 23).unwrap())
		);
	}
}
