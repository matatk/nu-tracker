use std::str::FromStr;

use clap::Parser;

use ntlib::{
	actions, comments, config, flags_labels_conflicts, get_repos, issues, specs, AssigneeQuery,
	LabelStringVec, Locator,
};

mod invoke;

use crate::invoke::{Cli, Command, ConfigCommand, IssueActionArgs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let cli = Cli::parse();

	config::ensure_dir()?;
	let repositories = config::Repos::load_or_init()?;
	let settings = config::Settings::load_or_init()?;

	let group_name = ascertain_group_name(
		&cli.working_group,
		|| settings.wg().to_string(),
		&repositories.known_wg_names(),
	);

	if cli.verbose {
		println!("Operating from the perspective of the '{}' WG", group_name)
	}

	let all_wgs_repos = repositories.wgs_repos();
	let wg_repos = all_wgs_repos
		.get(&group_name)
		.expect("should be able to get WorkingGroupInfo");

	match cli.command {
		Command::Issues {
			shared: IssueActionArgs {
				repos,
				assignees,
				label,
				closed,
			},
			actions,
		} => issues(
			&get_repos(wg_repos, &repos.main, &repos.sources.wg, &repos.sources.tf),
			AssigneeQuery::new(assignees.assignee, assignees.no_assignee),
			&label,
			&closed,
			&cli.verbose,
			&actions,
		),

		Command::Actions {
			shared: IssueActionArgs {
				repos,
				assignees,
				label,
				closed,
			},
		} => actions(
			&get_repos(wg_repos, &repos.main, &repos.sources.wg, &repos.sources.tf),
			AssigneeQuery::new(assignees.assignee, assignees.no_assignee),
			&label,
			&closed,
			&cli.verbose,
		),

		Command::Specs {
			assignees,
			review_number,
		} => {
			// FIXME: this is already checked in comments() and specs() -- somehow enforce that only Some() variants are passed in?
			if wg_repos.horizontal_review.is_some() {
				comments_or_specs(
					|| {
						specs(
							&group_name,
							wg_repos,
							AssigneeQuery::new(assignees.assignee.clone(), assignees.no_assignee),
							&cli.verbose,
						)
					},
					&review_number,
					&wg_repos.horizontal_review.as_ref().unwrap().specs,
				)
			} else {
				println!("{group_name} is not a horizontal review group")
			}
		}

		Command::Comments {
			status_flags,
			status,
			spec,
			assignees,
			show_source,
			request_number,
		} => {
			if status_flags {
				println!("{}", flags_labels_conflicts());
				return Ok(());
			}

			// FIXME: this is already checked in comments() and specs() -- somehow enforce that only Some() variants are passed in?
			if wg_repos.horizontal_review.is_some() {
				comments_or_specs(
					|| {
						comments(
							&group_name,
							wg_repos,
							// TODO: implement Default (needs cleanup)
							// TODO: clean up generally
							status
								.as_ref()
								.unwrap_or(LabelStringVec::from_str("").as_ref().unwrap()),
							&spec,
							AssigneeQuery::new(assignees.assignee.clone(), assignees.no_assignee),
							&show_source,
							&cli.verbose,
						)
					},
					&request_number,
					&wg_repos.horizontal_review.as_ref().unwrap().comments,
				)
			} else {
				println!("{group_name} is not a horizontal review group")
			}
		}

		Command::Browse { issue_locator } => open_locator(&issue_locator),

		Command::Config { command } => match command {
			ConfigCommand::ShowDir => println!("{}", config::config_dir().to_string_lossy()),

			ConfigCommand::WorkingGroup { working_group } => {
				let mut settings = settings;
				match working_group {
					Some(wg) => settings.set_wg(wg, &repositories.known_wg_names()),
					None => {
						println!("Default WG is: '{}'", settings.wg());
						println!(
							"You can override this temporarily via the --working-group/-g option."
						)
					}
				}
			}
		},
	}

	Ok(())
}

fn comments_or_specs<F: Fn()>(handler: F, open_number: &Option<u32>, org_and_repo: &str) {
	if let Some(targ) = open_number {
		let locator = format!("{org_and_repo}#{targ}");
		open_locator(locator.as_str())
	} else {
		handler()
	}
}

fn open_locator(issue_locator: &str) {
	if let Ok(locator) = Locator::from_str(issue_locator) {
		println!("Opening: {}", locator.url());
		if let Err(err) = open::that(locator.url()) {
			println!("Error: {err}")
		}
	} else {
		println!("Invalid issue locator: {issue_locator}")
	}
}

fn ascertain_group_name(
	parameter: &Option<String>,
	fallback: impl Fn() -> String,
	valid_wgs: &[&String],
) -> String {
	let group_name = match parameter {
		Some(group) => group.clone(), // TODO: why does this contain a &String?
		None => fallback(),
	};

	if !valid_wgs.contains(&&group_name) {
		let qualifier = match parameter {
			Some(_) => "given on command line",
			None => "specified in settings file",
		};
		println!("Unknown WG name {qualifier}: '{group_name}' - using 'apa' for this run.");
		return String::from("apa");
	}

	group_name
}
