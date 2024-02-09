use std::{error::Error, str::FromStr};

use clap::Parser;

use invoke::{ReportFormatArg, StatusArgs};
use ntlib::{
	actions, charters, comments, config, get_repos, issues, specs, AssigneeQuery,
	CharterStatusValidator, CommentStatusValidator, LabelInfo, Locator,
};

mod invoke;

use crate::invoke::{Cli, Command, ConfigCommand, IssueActionArgs};

fn main() {
	if let Err(error) = run() {
		println!("Error: {error}");
	}
}

fn run() -> Result<(), Box<dyn Error>> {
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
			shared:
				IssueActionArgs {
					repos,
					assignees,
					label,
					closed,
					rf: ReportFormatArg { report_formats },
				},
			actions,
		} => issues(
			get_repos(wg_repos, &repos.main, &repos.sources.wg, &repos.sources.tf)?,
			AssigneeQuery::new(assignees.assignee, assignees.no_assignee),
			label,
			closed,
			actions,
			&report_formats,
			cli.verbose,
		)?,

		Command::Actions {
			shared:
				IssueActionArgs {
					repos,
					assignees,
					label,
					closed,
					rf: ReportFormatArg { report_formats },
				},
		} => actions(
			get_repos(wg_repos, &repos.main, &repos.sources.wg, &repos.sources.tf)?,
			AssigneeQuery::new(assignees.assignee, assignees.no_assignee),
			label,
			closed,
			&report_formats,
			cli.verbose,
		)?,

		Command::Comments {
			status: StatusArgs {
				status_flags,
				mut status,
				mut not_status,
			},
			mut spec,
			assignees,
			show_source,
			request_number,
			rf: ReportFormatArg { report_formats },
		} => {
			if status_flags {
				println!("{}", CommentStatusValidator::flags_labels_conflicts());
				return Ok(());
			}

			comments_or_specs(
				&group_name,
				wg_repos
					.horizontal_review
					.as_ref()
					.map(|hr| hr.comments.as_str()),
				|repo| {
					comments(
						repo,
						status.take().unwrap_or_default(),
						not_status.take().unwrap_or_default(),
						spec.take(),
						AssigneeQuery::new(assignees.assignee.clone(), assignees.no_assignee),
						show_source,
						&report_formats,
						cli.verbose,
					)
				},
				request_number,
			)?
		}

		Command::Specs {
			assignees,
			review_number,
			rf: ReportFormatArg { report_formats },
		} => comments_or_specs(
			&group_name,
			wg_repos
				.horizontal_review
				.as_ref()
				.map(|hr| hr.specs.as_str()),
			|repo| {
				specs(
					repo,
					AssigneeQuery::new(assignees.assignee.clone(), assignees.no_assignee),
					&report_formats,
					cli.verbose,
				)
			},
			review_number,
		)?,

		Command::Charters {
			status: StatusArgs {
				status_flags,
				mut status,
				mut not_status,
			},
			review_number,
			rf: ReportFormatArg { report_formats },
		} => {
			if status_flags {
				println!("{}", CharterStatusValidator::flags_labels_conflicts());
				return Ok(());
			}

			let repo = "w3c/strategy";
			// FIXME: DRY
			if let Some(targ) = review_number {
				let locator = format!("{repo}#{targ}");
				open_locator(locator.as_str())
			} else {
				charters(
					repo,
					status.take().unwrap_or_default(),
					not_status.take().unwrap_or_default(),
					&report_formats,
					cli.verbose,
				)?
			}
		}

		Command::Browse { issue_locator } => open_locator(&issue_locator),

		Command::Config { command } => match command {
			ConfigCommand::ShowDir => println!("{}", config::config_dir()?.to_string_lossy()),

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

fn comments_or_specs<F: FnMut(&str) -> Result<(), Box<dyn Error>>>(
	group_name: &str,
	org_and_repo: Option<&str>,
	mut handler: F,
	open_number: Option<u32>,
) -> Result<(), Box<dyn Error>> {
	if let Some(repo) = org_and_repo {
		if let Some(targ) = open_number {
			let locator = format!("{repo}#{targ}");
			open_locator(locator.as_str())
		} else {
			handler(repo)?
		}
	} else {
		println!("{group_name} is not a horizontal review group")
	}
	Ok(())
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
