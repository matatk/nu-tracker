use std::{error::Error, fmt, println, str};

use chrono::NaiveDate;
use regex::Regex;

use crate::assignee_query::AssigneeQuery;
use crate::config::{WgOrTfRepos, WorkingGroupInfo};
use crate::flatten_assignees::flatten_assignees;
use crate::make_table::make_table;
use crate::query::Query;
use crate::returned_issue::ReturnedIssue;
use crate::{fetch_sort_print_handler, ReportFormat};

#[derive(Debug)]
pub enum GetReposError {
	NoReposSelected,
	NoSuchTf {
		task_force: String,
		group_task_forces: Vec<String>,
	},
}

impl Error for GetReposError {}

impl fmt::Display for GetReposError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
            GetReposError::NoReposSelected => write!(f, "No repos selected"),
            GetReposError::NoSuchTf { task_force, group_task_forces } => write!(f, "No TF called '{}'—you may want to pass the TF option last on the command line. Known TFs for this WG are: {}", task_force, group_task_forces.iter().map(|tf| format!("'{tf}'")).collect::<Vec<String>>().join(", "))
        }
	}
}

struct Action {
	issue: ReturnedIssue,
	due: Option<NaiveDate>,
}

impl Action {
	// TODO: Make trait?
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
			self.issue.title.to_string(),
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
			ReportFormat::Table => todo!(),
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

	let transmogrify = |issue: ReturnedIssue| {
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
	let table = make_table(
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

pub fn get_repos<'a>(
	repos: &'a WorkingGroupInfo,
	main: &bool,
	wg: &bool,
	tf: &'a Option<Vec<String>>,
) -> Result<Vec<&'a str>, GetReposError> {
	let mut query_repos: Vec<&str> = Vec::new();

	if *wg {
		add_repos_for_team(&mut query_repos, main, &repos.working_group)
	}

	if let Some(task_forces) = tf {
		if task_forces.is_empty() {
			for team_repos in repos.task_forces.values() {
				add_repos_for_team(&mut query_repos, main, team_repos)
			}
		} else {
			for task_force in task_forces {
				if let Some(team_repos) = repos.task_forces.get(task_force) {
					add_repos_for_team(&mut query_repos, main, team_repos)
				} else {
					return Err(GetReposError::NoSuchTf {
						task_force: task_force.clone(),
						group_task_forces: repos.task_forces.keys().cloned().collect(),
					});
				}
			}
		}
	}

	if query_repos.is_empty() {
		// NOTE: This should not happen because the CLI UI has at least one of these arguments as
		//       required (however, there could be a UI layer bug :-)).
		return Err(GetReposError::NoReposSelected);
	}

	Ok(query_repos)
}

fn add_repos_for_team<'a>(dest: &mut Vec<&'a str>, main: &bool, team_repos: &'a WgOrTfRepos) {
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
