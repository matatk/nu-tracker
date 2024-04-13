use std::{error::Error, str::FromStr};

use clap::Parser;

use ntlib::{
	actions, charters, comments, designs, issues, select_repos, specs, AssigneeQuery,
	CharterFromStrHelper, CommentFromStrHelper, DesignFromStrHelper, Locator, OriginQuery,
	StatusLabelInfo,
};

mod context;
mod invoke;

use crate::context::Context;
use crate::invoke::{Cli, Command, ConfigCommand};

fn main() {
	if let Err(error) = run() {
		println!("Error: {error}");
	}
}

fn run() -> Result<(), Box<dyn Error>> {
	let cli = Cli::parse();

	let mut ctx = Context::new(cli.as_group, cli.repos_file, cli.verbose)?;

	macro_rules! select_repos {
		($ctx:ident, $repos:expr) => {
			select_repos(
				$ctx.group_repos()?,
				&$repos.main,
				&$repos.sources.include_group,
				&$repos.sources.include_tfs,
			)?
		};
	}

	match cli.command {
		Command::Issues { shared, actions } => issues(
			select_repos!(ctx, shared.repos),
			AssigneeQuery::new(shared.assignees.assignee, shared.assignees.no_assignee),
			shared.label,
			shared.closed,
			actions,
			&shared.report.formats,
			cli.verbose,
		)?,

		// TODO: Allow user to give number on CLI to open that issue number in the group's
		// main repo? If we're going from only one TF's perspective, then do the same for
		// the TF?
		Command::Actions { shared } => actions(
			select_repos!(ctx, shared.repos),
			AssigneeQuery::new(shared.assignees.assignee, shared.assignees.no_assignee),
			shared.label,
			shared.closed,
			&shared.report.formats,
			cli.verbose,
		)?,

		Command::Comments {
			mut shared, // FIXME: not all things need to be mut but some do
			origin,
		} => {
			if shared.status.status_flags {
				println!("{}", CommentFromStrHelper::flags_labels_conflicts());
				return Ok(());
			}

			let columns = shared.columns.unwrap_or(ctx.settings().comment_columns());

			comments_or_specs(
				&ctx.group_name()?,
				ctx.group_repos()?.hr_comments(),
				|repo| {
					comments(
						repo,
						shared.status.status.take().unwrap_or_default(),
						shared.status.not_status.take().unwrap_or_default(),
						shared.spec.take(),
						AssigneeQuery::new(
							shared.assignees.assignee.clone(),
							shared.assignees.no_assignee,
						),
						&shared.report.formats,
						&columns,
						OriginQuery::new(origin.our, origin.other),
						cli.verbose,
					)
				},
				shared.request_number,
			)?
		}

		Command::Designs {
			mut shared, // FIXME: not all things need to be mut but some do
		} => {
			if shared.status.status_flags {
				println!("{}", DesignFromStrHelper::flags_labels_conflicts());
				return Ok(());
			}

			let columns = shared.columns.unwrap_or(ctx.settings().design_columns());

			comments_or_specs(
				&ctx.group_name()?,
				ctx.group_repos()?.hr_designs(),
				|repo| {
					designs(
						repo,
						shared.status.status.take().unwrap_or_default(),
						shared.status.not_status.take().unwrap_or_default(),
						shared.spec.take(),
						AssigneeQuery::new(
							shared.assignees.assignee.clone(),
							shared.assignees.no_assignee,
						),
						&shared.report.formats,
						&columns,
						cli.verbose,
					)
				},
				shared.request_number,
			)?
		}

		Command::Specs {
			assignees,
			review_number,
			report,
		} => comments_or_specs(
			&ctx.group_name()?,
			ctx.group_repos()?.hr_specs(),
			|repo| {
				specs(
					repo,
					AssigneeQuery::new(assignees.assignee.clone(), assignees.no_assignee),
					&report.formats,
					cli.verbose,
				)
			},
			review_number,
		)?,

		Command::Charters {
			mut status,
			review_number,
			report,
		} => {
			if status.status_flags {
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
					status.status.take().unwrap_or_default(),
					status.not_status.take().unwrap_or_default(),
					&report.formats,
					cli.verbose,
				)?
			}
		}

		Command::Browse { issue_locator } => open_locator(&issue_locator),

		Command::Config { command } => match command {
			ConfigCommand::ShowDir => {
				println!("{}", Context::config_dir().display())
			}

			ConfigCommand::Group { group } => match group {
				Some(g) => {
					ctx.all_group_repos().for_group(&g)?; // TODO: inelegant?
					ctx.settings_mut().set_group(g)
				}
				None => {
					match ctx.settings().group() {
						Some(set) => {
							println!("Default group from settings file is: '{set}'");
							if ctx.is_group_name_overridden() {
								println!("This has been overridden temporarily via the `--as` option to: '{}'", ctx.group_name().expect("when group name is overridden, group_name() should work"))
							// NOTE: SYNCH: invoke.rs
							} else {
								println!("You can override this temporarily via the `--as` option.") // NOTE: SYNCH: invoke.rs
							}
						}
						None => {
							let cli_group = ctx.group_name()?;
							println!("There's no settings file in use, or there's no default group specified there.");
							println!("Using group '{cli_group}' for this run, as given via the `--as` option.");
							// NOTE: SYNCH: invoke.rs
						}
					}
				}
			},

			ConfigCommand::CommentColumns { cs } => {
				config_comments_designs!(ctx, comment, "comments", cs);
			}

			ConfigCommand::DesignColumns { cs } => {
				config_comments_designs!(ctx, design, "designs", cs);
			}

			ConfigCommand::ReposInfo => {
				let repos_pretty = ctx.all_group_repos().stringify()?;
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

// FIXME: distinguish settings file vs command-line, like with group above?
macro_rules! config_comments_designs {
    ($ctx:ident, $name:ident, $pretty:expr, $fields:ident) => {
		::paste::paste! {
			match $fields {
				Some(actual_fields) => $ctx.settings_mut().[<set_ $name _columns>](actual_fields),
				None => {
					println!(
						concat!("Default ", $pretty, " table columns are: {}"),
						::ntlib::DisplayableVec::from($ctx.settings().[<$name _columns>]())
					);
					println!(concat!("You can override this temporarily via the --columns/-c option of the `", $pretty, "` sub-command."))
					// NOTE: SYNCH: invoke.rs
				}
			}
		};
    }
}

pub(crate) use config_comments_designs;
