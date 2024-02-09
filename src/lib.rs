//! Provides functions to query GitHub for issues, actions, and horizontal spec review and issue
//! comment requests, according to W3C conventions.
//!
//! The `gh` command is used to actually make the queries. The output from `gh` is eitehr printed
//! verbatim (in the case of issues), or obtained in JSON format, and processed extensively to add
//! more helpful information to it, to help WG and TF chairs keep track of things.
//!
//! For info on how to use the tool based on this library, refer to [the Nu Tracker README on GitHub](https://github.com/matatk/nu-tracker/blob/main/README.md).
use std::error::Error;

use clap::ValueEnum;
use serde::Deserialize;
use struct_field_names_as_array::FieldNamesAsArray;

pub use charters::charters;
pub use comments::comments;
pub use issues_actions::{actions, get_repos, issues};
pub use locator::Locator;
use query::Query;
pub use specs::specs;

mod assignee_query;
mod charters;
mod comments;
pub mod config;
mod flatten_assignees;
mod issues_actions;
mod locator;
mod make_table;
mod query;
mod returned_issue;
mod showing;
mod specs;
mod status_labels;

pub use assignee_query::AssigneeQuery;
pub use status_labels::{
	CharterLabels, CharterStatusValidator, CommentLabels, CommentStatus, CommentStatusValidator,
	LabelInfo, ParseFlagError,
};

#[derive(Clone, Copy, ValueEnum)]
pub enum ReportFormat {
	/// Print via GitHub CLI
	#[clap(hide(true))]
	Gh,
	/// Tabular
	Table,
	/// Subtopics and links, for pasting into IRC during a call
	Meeting,
	/// List, suitable for use in call announcements
	Agenda,
	/// Open in a browser
	Web,
}

fn fetch<
	const N: usize,
	ReturnedIssueType: FieldNamesAsArray<N> + for<'a> Deserialize<'a>,
	DomainType,
	Transform: Fn(ReturnedIssueType) -> DomainType,
	GetSortKey: Fn(&DomainType) -> DomainKeyType,
	DomainKeyType: Ord,
>(
	name: &str,
	query: &mut Query,
	transform: Transform,
	get_sort_key: Option<GetSortKey>,
) -> Result<Vec<DomainType>, Box<dyn Error>> {
	let mut requests: Vec<DomainType> = query
		.run(name, ReturnedIssueType::FIELD_NAMES_AS_ARRAY.to_vec())?
		.into_iter()
		.map(transform)
		.collect();
	if let Some(gsk) = get_sort_key {
		requests.sort_by_key(gsk)
	}
	Ok(requests)
}
