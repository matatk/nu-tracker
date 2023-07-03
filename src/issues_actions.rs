use std::io::{self, Write};
use std::{println, process::Command, str};

use chrono::NaiveDate;
use regex::Regex;

use crate::assignee_query::AssigneeQuery;
use crate::config::{WgOrTfRepos, WorkingGroupInfo};
use crate::flatten_assignees::flatten_assignees;
use crate::make_table::make_table;
use crate::returned_issue::ReturnedIssue;
use crate::showing::showing;

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
	repos: &Vec<&str>,
	assignee: AssigneeQuery,
	actions: &bool,
	closed: &bool,
	verbose: &bool,
) {
	let mut cmd = Command::new("gh");
	add_base_args(&mut cmd, repos, &assignee, closed);
	if !*actions {
		cmd.args(vec!["--", "-label:action"]);
	}

	if *verbose {
		println!("Issues: running: {cmd:?}");
	}
	cmd.status().expect("'gh' should run");
}

/// Query for action issues in given repos; make a custom report, sorted by due date.
// TODO: DRY with specs, comments?
pub fn actions(repos: &Vec<&str>, assignee: AssigneeQuery, closed: &bool, verbose: &bool) {
	let mut cmd = Command::new("gh");
	add_base_args(&mut cmd, repos, &assignee, closed);
	cmd.args(["--label", "action"])
		.args(["--json", &ReturnedIssue::FIELD_NAMES_AS_ARRAY.join(",")]);

	if *verbose {
		println!("Actions: running: {cmd:?}");
	}
	let output = cmd.output().expect("'gh' should run");

	if output.status.success() {
		let out = str::from_utf8(&output.stdout).expect("got non-utf8 data from 'gh'");
		let actions: Vec<ReturnedIssue> = serde_json::from_str(out).unwrap();

		if actions.is_empty() {
			println!("No actions found");
			return;
		} else {
			println!("{} actions\n", showing(actions.len()))
		}

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
	} else {
		io::stdout().write_all(&output.stdout).unwrap();
		io::stderr().write_all(&output.stderr).unwrap();
		panic!("'gh' did not run successfully")
	}
}

fn add_base_args(
	command: &mut Command,
	repos: &Vec<&str>,
	assignee: &AssigneeQuery,
	closed: &bool,
) {
	let closed_args: Vec<&str> = if *closed {
		vec![]
	} else {
		vec!["--state", "open"]
	};

	command.args(["search", "issues"]).args(closed_args);

	for repo in repos {
		command.args(vec!["--repo", repo]);
	}

	assignee.gh_args(command);
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
