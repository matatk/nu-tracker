// TODO: DRY review_number? Also request_number?
use std::{error::Error, path::PathBuf, str::FromStr};

use clap::{Args, Parser, Subcommand, ValueEnum};

use ntlib::{CharterLabels, CommentField, CommentLabels, DesignField, DesignLabels, ReportFormat};

/// Nu Tracker: Track W3C actions and horizontal review requests
#[derive(Parser)]
#[command(author, version)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Command,
	/// Verbose mode (prints out the 'gh' command line etc.)
	#[arg(short, long, global = true)]
	pub verbose: bool,
	/// Operate from the perspective of group (overrides config file)
	#[arg(long = "as", value_name = "GROUP")] // NOTE: main.rs
	pub as_group: Option<String>,
	/// Load repository info from a custom JSON file
	///
	/// You can get the current known repos in JSON format by using the `nt config repos-info` command.
	// NOTE: Synch this message with name below.
	#[arg(long, value_name = "FILE")]
	pub repos_file: Option<PathBuf>,
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
		shared: CommentDesignArgs<CommentLabels, CommentField>,
		#[clap(flatten)]
		origin: OriginArgs,
	},
	/// List requests for comments on other groups' designs
	Designs {
		#[clap(flatten)]
		shared: CommentDesignArgs<DesignLabels, DesignField>,
	},
	/// List review requests by due date, or open a specific request
	Specs {
		#[clap(flatten)]
		assignees: AssigneeArgs,
		#[clap(flatten)]
		report: ReportFormatsArg,
		/// Review number (only) to open in the browser (e.g. '42')
		review_number: Option<u32>,
	},
	/// List charter review requests, or open a specific request
	Charters {
		#[clap(flatten)]
		status: StatusArgs<CharterLabels>,
		#[clap(flatten)]
		report: ReportFormatsArg,
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
	/// Get or set the default group
	Group {
		/// Operate from the perspective of group (defaults to 'apa')
		#[arg()]
		group: Option<String>,
	},
	/// Get or set the default columns for the comments table
	CommentColumns {
		/// Make columns for these fields (in the given order)
		#[arg(value_name = "FIELD")]
		cs: Option<Vec<CommentField>>,
	},
	/// Get or set the default columns for the designs table
	DesignColumns {
		/// Make columns for these fields (in the given order)
		#[arg(value_name = "FIELD")]
		cs: Option<Vec<DesignField>>,
	},
	/// Print out the default repository info in JSON format
	// NOTE: Synch this name with the docstring above.
	ReposInfo,
}

#[derive(Args)]
pub struct RepoArgs {
	#[clap(flatten)]
	pub sources: RepoSourceArgs,
	/// Include main group/TF repos only
	#[arg(short, long)]
	pub main: bool,
}

#[derive(Args)]
#[group(required = true)]
pub struct RepoSourceArgs {
	/// Include the group's repos
	#[arg(short = 'g')]
	pub include_group: bool,
	/// Include TFs' repos (all TFs if no arguments given)
	#[arg(short = 't', value_name = "TF", num_args = 0..)]
	pub include_tfs: Option<Vec<String>>,
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
#[group(multiple = false)]
pub struct OriginArgs {
	/// Only issues originating from our group (show all issues if unset)
	// TODO: i18n process URL?
	#[arg(short = 'o', long)]
	pub our: bool,
	/// Only issues originating from other groups (show all issues if unset)
	#[arg(short = 'O', long)]
	pub other: bool,
}

#[derive(Args)]
pub struct IssueActionArgs {
	#[clap(flatten)]
	pub repos: RepoArgs,
	#[clap(flatten)]
	pub assignees: AssigneeArgs,
	#[clap(flatten)]
	pub report: ReportFormatsArg,
	/// Only those with all of the given labels
	#[arg(short, long, value_name = "LABEL", num_args = 1..)]
	pub label: Vec<String>,
	/// Include closed ones
	#[arg(short, long)]
	pub closed: bool,
}

#[derive(Args)]
pub struct CommentDesignArgs<
	T: FromStr + Send + Sync + Clone + 'static,
	F: ValueEnum + Send + Sync + Clone + 'static,
> where
	T::Err: Error + Send + Sync + 'static,
{
	#[clap(flatten)]
	pub status: StatusArgs<T>,
	/// Filter by spec, or spec group (e.g. 'open-ui')
	// FIXME: is this correct in the modern usage?
	// FIXME: missing equivalent option for group? OR use this for both? Multiple args?
	#[arg(short = 'p', long)]
	pub spec: Option<String>,
	#[clap(flatten)]
	pub assignees: AssigneeArgs,
	/// Show the source issue column in the table
	#[arg(short = 'i', long)]
	pub show_source: bool,
	/// Request number (only) to open in the browser (e.g. '42')
	pub request_number: Option<u32>,
	#[clap(flatten)]
	pub report: ReportFormatsArg,
	/// Columns to include in the table (overrides config file)
	#[arg(short, long, value_name = "FIELD", num_args = 1.., value_enum)]
	pub columns: Option<Vec<F>>, // NOTE: main.rs
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
pub struct ReportFormatsArg {
	#[arg(short = 'r', long = "report", value_name = "FORMAT", num_args = 1.., default_values_t = vec![ReportFormat::Table], value_enum)]
	pub formats: Vec<ReportFormat>,
}
