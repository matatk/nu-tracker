use std::{println, str};

use chrono::NaiveDate;
use regex::Regex;

use crate::assignee_query::AssigneeQuery;
use crate::config::{WgOrTfRepos, WorkingGroupInfo};
use crate::flatten_assignees::flatten_assignees;
use crate::make_table::make_table;
use crate::query::Query;
use crate::returned_issue::ReturnedIssue;

struct DatedAction {
	action: ReturnedIssue,
	due: Option<NaiveDate>,
}

impl DatedAction {
	// TODO: Make trait?
	fn to_vec_string(&self) -> Vec<String> {
		vec![
			match self.due {
				Some(date) => format!("{date}"),
				None => String::from("(no date)"),
			},
			format!(
				"{}#{}",
				self.action.repository.name_with_owner, self.action.number
			),
			self.action.title.to_string(),
			flatten_assignees(&self.action.assignees),
		]
	}
}

/// Query for issues in given repos; have `gh` print the output.
pub fn issues(
	repos: Vec<&str>,
	assignee: AssigneeQuery,
	labels: Vec<String>,
	closed: bool,
	verbose: bool,
	actions: bool,
) {
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
		.assignee(assignee)
		.run_direct();
}

// FIXME: if including closed, show status column
/// Query for action issues in given repos; make a custom report, sorted by due date.
pub fn actions(
	repos: Vec<&str>,
	assignee: AssigneeQuery,
	labels: Vec<String>,
	closed: bool,
	verbose: bool,
) {
	let mut query = Query::new("Actions", verbose);
	let actions: Vec<ReturnedIssue> = query
		.repos(repos)
		.assignee(assignee)
		.labels(labels)
		.label("action")
		.include_closed(closed)
		.run("actions", ReturnedIssue::FIELD_NAMES_AS_ARRAY.to_vec());

	let mut dated_actions: Vec<DatedAction> = vec![];
	for action in actions {
		dated_actions.push(DatedAction {
			action: action.clone(), // TODO: idiomatic?
			due: get_due(&action.body),
		})
	}
	dated_actions.sort_by_key(|a| a.due);

	let mut rows: Vec<Vec<String>> = vec![];
	for dated in dated_actions {
		rows.push(dated.to_vec_string())
	}

	let table = make_table(vec!["DUE", "LOCATOR", "TITLE", "ASSIGNEES"], rows, None);
	println!("{table}")
}

pub fn get_repos<'a>(
	repos: &'a WorkingGroupInfo,
	main: &bool,
	wg: &bool,
	tf: &Option<Vec<String>>,
) -> Vec<&'a str> {
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
					panic!("No TF called '{}'—you may want to pass the TF option last on the command line. Known TFs for this WG are:\n{:?}", task_force, repos.task_forces.keys());
				}
			}
		}
	}

	if query_repos.is_empty() {
		panic!("No repos selected")
	}

	query_repos
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

fn get_due(line: &str) -> Option<NaiveDate> {
	let re = Regex::new(r"^due  ?(\d\d? [[:alpha:]]{3} \d{4})$").unwrap();
	let first_line = line.lines().next().unwrap();

	if let Some(caps) = re.captures(first_line) {
		let date_text = caps.get(1).unwrap().as_str();
		return NaiveDate::parse_from_str(date_text, "%d %b %Y").ok();
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
