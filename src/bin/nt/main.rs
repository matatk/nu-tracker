use std::{error::Error, str::FromStr};

use clap::Parser;

use invoke::{CommentDesignArgs, ReportFormatsArg, StatusArgs};
use ntlib::{
	actions, charters, comments,
	config::{AllGroupRepos, GroupRepos, Settings},
	designs, get_repos, issues, specs, AssigneeQuery, CharterFromStrHelper, CommentFromStrHelper,
	DesignFromStrHelper, DisplayableVec, Locator, OriginQuery, StatusLabelInfo,
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

	let repos = || -> Result<AllGroupRepos, Box<dyn Error>> {
		Ok(AllGroupRepos::load_or_init(&cli.repos_file, &cli.verbose)?)
	};

	let repos_and_settings = || -> Result<(AllGroupRepos, Settings), Box<dyn Error>> {
		Ok((repos()?, Settings::load_or_init(cli.verbose)?))
	};

	match cli.command {
		Command::Issues {
			shared:
				IssueActionArgs {
					repos,
					assignees,
					label,
					closed,
					report: ReportFormatsArg { formats },
				},
			actions,
		} => {
			let (repositories, mut settings) = repos_and_settings()?;
			issues(
				get_repos(
					group_and_repos(&repositories, &mut settings, cli.as_group, cli.verbose)?.1,
					&repos.main,
					&repos.sources.include_group,
					&repos.sources.include_tfs,
				)?,
				AssigneeQuery::new(assignees.assignee, assignees.no_assignee),
				label,
				closed,
				actions,
				&formats,
				cli.verbose,
			)?
		}

		// TODO: Allow user to give number on CLI to open that issue number in the group's
		// main repo? If we're going from only one TF's perspective, then do the same for
		// the TF?
		Command::Actions {
			shared:
				IssueActionArgs {
					repos,
					assignees,
					label,
					closed,
					report: ReportFormatsArg { formats },
				},
		} => {
			let (repositories, mut settings) = repos_and_settings()?;
			actions(
				get_repos(
					group_and_repos(&repositories, &mut settings, cli.as_group, cli.verbose)?.1,
					&repos.main,
					&repos.sources.include_group,
					&repos.sources.include_tfs,
				)?,
				AssigneeQuery::new(assignees.assignee, assignees.no_assignee),
				label,
				closed,
				&formats,
				cli.verbose,
			)?
		}

		Command::Comments {
			shared:
				CommentDesignArgs {
					status:
						StatusArgs {
							status_flags,
							mut status,
							mut not_status,
						},
					mut spec,
					assignees,
					request_number,
					report: ReportFormatsArg { formats },
					columns,
				},
			origin,
		} => {
			if status_flags {
				println!("{}", CommentFromStrHelper::flags_labels_conflicts());
				return Ok(());
			}

			let (repositories, mut settings) = repos_and_settings()?;
			let columns = columns.unwrap_or(settings.comment_columns());

			// FIXME: DRY with specs
			let (group_name, group_repos) =
				group_and_repos(&repositories, &mut settings, cli.as_group, cli.verbose)?;

			comments_or_specs(
				&group_name,
				group_repos.hr_comments(),
				|repo| {
					comments(
						repo,
						status.take().unwrap_or_default(),
						not_status.take().unwrap_or_default(),
						spec.take(),
						AssigneeQuery::new(assignees.assignee.clone(), assignees.no_assignee),
						&formats,
						&columns,
						OriginQuery::new(origin.our, origin.other),
						cli.verbose,
					)
				},
				request_number,
			)?
		}

		Command::Designs {
			shared:
				CommentDesignArgs {
					status:
						StatusArgs {
							status_flags,
							mut status,
							mut not_status,
						},
					mut spec,
					assignees,
					request_number,
					report: ReportFormatsArg { formats },
					columns,
				},
		} => {
			if status_flags {
				println!("{}", DesignFromStrHelper::flags_labels_conflicts());
				return Ok(());
			}

			let (repositories, mut settings) = repos_and_settings()?;
			let columns = columns.unwrap_or(settings.design_columns());

			// FIXME: DRY with specs
			let (group_name, group_repos) =
				group_and_repos(&repositories, &mut settings, cli.as_group, cli.verbose)?;

			comments_or_specs(
				&group_name,
				group_repos.hr_designs(),
				|repo| {
					designs(
						repo,
						status.take().unwrap_or_default(),
						not_status.take().unwrap_or_default(),
						spec.take(),
						AssigneeQuery::new(assignees.assignee.clone(), assignees.no_assignee),
						&formats,
						&columns,
						cli.verbose,
					)
				},
				request_number,
			)?
		}

		Command::Specs {
			assignees,
			review_number,
			report: ReportFormatsArg { formats },
		} => {
			let (repositories, mut settings) = repos_and_settings()?;

			// FIXME: DRY with comments
			let (group_name, group_repos) =
				group_and_repos(&repositories, &mut settings, cli.as_group, cli.verbose)?;

			comments_or_specs(
				&group_name,
				group_repos.hr_specs(),
				|repo| {
					specs(
						repo,
						AssigneeQuery::new(assignees.assignee.clone(), assignees.no_assignee),
						&formats,
						cli.verbose,
					)
				},
				review_number,
			)?
		}

		Command::Charters {
			status: StatusArgs {
				status_flags,
				mut status,
				mut not_status,
			},
			review_number,
			report: ReportFormatsArg { formats },
		} => {
			if status_flags {
				println!("{}", CharterFromStrHelper::flags_labels_conflicts());
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
					&formats,
					cli.verbose,
				)?
			}
		}

		Command::Browse { issue_locator } => open_locator(&issue_locator),

		Command::Config { command } => match command {
			ConfigCommand::ShowDir => {
				println!("{}", Settings::config_dir().display())
			}

			ConfigCommand::Group { group } => match group {
				Some(g) => {
					let (repositories, mut settings) = repos_and_settings()?;
					let _ = repositories.for_group(&g)?;
					settings.set_group(g)
				}
				None => {
					let settings = Settings::load_or_init(cli.verbose)?;
					println!("Default group is: '{}'", settings.group());
					println!("You can override this temporarily via the `--as` option.") // NOTE: invoke.rs
				}
			},

			ConfigCommand::CommentColumns { cs } => {
				config_comments_designs!(cli, comment, "comments", cs);
			}

			ConfigCommand::DesignColumns { cs } => {
				config_comments_designs!(cli, design, "designs", cs);
			}

			ConfigCommand::ReposInfo => {
				let repos_pretty = repos()?.stringify()?;
				println!("{repos_pretty}");
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
		return Err(format!("'{group_name}' doesn't do this kind of horizontal review").into());
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

fn group_and_repos<'a>(
	repositories: &'a AllGroupRepos,
	settings: &'a mut Settings,
	cli_group: Option<String>,
	verbose: bool,
) -> Result<(String, &'a GroupRepos), Box<dyn Error>> {
	let group_name = cli_group.unwrap_or(settings.group());
	let group_repos = repositories.for_group(&group_name)?;
	if verbose {
		println!("Operating from the perspective of group '{}'", group_name)
	}
	Ok((group_name, group_repos))
}

macro_rules! config_comments_designs {
    ($cli:ident, $name:ident, $pretty:expr, $fields:ident) => {
		::paste::paste! {
			let mut settings = Settings::load_or_init($cli.verbose)?;
			match $fields {
				Some(actual_fields) => settings.[<set_ $name _columns>](actual_fields),
				None => {
					println!(
						concat!("Default ", $pretty, " table columns are: {}"),
						DisplayableVec::from(settings.[<$name _columns>]())
					);
					println!(concat!("You can override this temporarily via the --columns/-c option of the `", $pretty, "` sub-command."))
					// NOTE: invoke.rs
				}
			}
		};
    }
}

pub(crate) use config_comments_designs;
