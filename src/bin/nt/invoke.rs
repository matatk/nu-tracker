// TODO: DRY review_number? Also request_number?
use std::{error::Error, str::FromStr};

use clap::{Args, Parser, Subcommand};

use ntlib::{CharterLabels, CommentLabels, ReportFormat};

/// Nu Tracker: Track W3C actions and horizontal review requests
#[derive(Parser)]
#[command(author, version)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Command,
	/// Verbose mode (prints out the 'gh' command line etc.)
	#[arg(short, long, global = true)]
	pub verbose: bool,
	/// Operate from the perspective of WG (overrides config file)
	#[arg(short = 'g', long, value_name = "WG")]
	pub working_group: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
	/// Query issues or actions; use 'gh' to display results table
	Issues {
		#[clap(flatten)]
		shared: IssueActionArgs,
		/// Include actions (issues with the label 'action')
		#[arg(short, long)]
		actions: bool,
	},
	/// Query actions; display results, by due date, in a custom table
	Actions {
		#[clap(flatten)]
		shared: IssueActionArgs,
	},
	/// List requests for comments on other groups' issues
	Comments {
		#[clap(flatten)]
		status: StatusArgs<CommentLabels>,
		/// Filter by spec, or spec group (e.g. 'open-ui')
		#[arg(short = 'p', long)]
		spec: Option<String>,
		#[clap(flatten)]
		assignees: AssigneeArgs,
		/// Show the source issue column in the table
		#[arg(short = 'i', long)]
		show_source: bool,
		/// Request number (only) to open in the browser (e.g. '42')
		request_number: Option<u32>,
		#[clap(flatten)]
		rf: ReportFormatArg,
	},
	/// List review requests by due date, or open a specific request
	Specs {
		#[clap(flatten)]
		assignees: AssigneeArgs,
		#[clap(flatten)]
		rf: ReportFormatArg,
		/// Review number (only) to open in the browser (e.g. '42')
		review_number: Option<u32>,
	},
	/// List charter review requests, or open a specific request
	Charters {
		#[clap(flatten)]
		status: StatusArgs<CharterLabels>,
		#[clap(flatten)]
		rf: ReportFormatArg,
		/// Review number (only) to open in the browser (e.g. '42')
		review_number: Option<u32>,
	},
	/// Open a specific GitHub issue in your browser
	Browse {
		/// Issue to open (e.g. 'w3c/apa#42')
		issue_locator: String,
	},
	/// Manage settings
	Config {
		#[command(subcommand)]
		command: ConfigCommand,
	},
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
	/// Show the configuration directory path (without creating it)
	ShowDir,
	/// Get or set the default working group
	WorkingGroup {
		/// Operate from the perspective of WG (defaults to 'apa')
		#[arg(value_name = "WG")]
		working_group: Option<String>,
	},
}

#[derive(Args)]
pub struct RepoArgs {
	#[clap(flatten)]
	pub sources: RepoSourceArgs,
	/// Include main WG/TF repos only
	#[arg(short, long)]
	pub main: bool,
}

#[derive(Args)]
#[group(required = true)]
pub struct RepoSourceArgs {
	/// Include WG repos
	#[arg(short)]
	pub wg: bool,
	/// Include TFs' repos (all TFs if no arguments given)
	#[arg(short, num_args = 0..)]
	pub tf: Option<Vec<String>>,
}

#[derive(Args)]
#[group(multiple = false)]
pub struct AssigneeArgs {
	/// Only those assigned to USER (use '@me' for yourself)
	#[arg(short = 'u', long, value_name = "USER")]
	pub assignee: Option<String>,
	/// Only those without assignees
	#[arg(short = 'U', long)]
	pub no_assignee: bool,
}

#[derive(Args)]
pub struct IssueActionArgs {
	#[clap(flatten)]
	pub repos: RepoArgs,
	#[clap(flatten)]
	pub assignees: AssigneeArgs,
	#[clap(flatten)]
	pub rf: ReportFormatArg,
	/// Only those with all of the given labels
	#[arg(short, long, value_name = "LABEL", num_args = 1..)]
	pub label: Vec<String>,
	/// Include closed ones
	#[arg(short, long)]
	pub closed: bool,
}

#[derive(Args)]
pub struct StatusArgs<T: FromStr + Send + Sync + Clone + 'static>
where
	T::Err: Error + Send + Sync + 'static,
{
	/// List known status flags, and their corresponding labels
	#[arg(short = 'f', long)]
	pub status_flags: bool,
	/// Query issues with these status labels, by flag letter(s) (e.g. 'TAP')
	#[arg(short, long, value_parser = T::from_str)]
	pub status: Option<T>,
	/// Query issues without these status labels, by flag letter(s) (e.g. 'TAP')
	#[arg(short = 'S', long, value_name = "STATUS", value_parser = T::from_str)]
	pub not_status: Option<T>,
}

#[derive(Args)]
pub struct ReportFormatArg {
	#[arg(short, long, num_args = 1.., default_values_t = vec![ReportFormat::Table], value_enum)]
	pub report_formats: Vec<ReportFormat>,
}
