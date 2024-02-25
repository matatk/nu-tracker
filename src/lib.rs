#![warn(missing_docs)]
//! Provides functions to query GitHub for issues, actions, and horizontal spec review and issue
//! comment requests, according to W3C conventions.
//!
//! The `gh` command is used to actually make the queries. The output from `gh` is either printed
//! verbatim (in the case of issues), or obtained in JSON format, and processed extensively to add
//! more helpful information to it, to help WG and TF chairs keep track of things.
//!
//! For info on how to use the tool based on this library, refer to [the Nu Tracker README on GitHub](https://github.com/matatk/nu-tracker/blob/main/README.md).
use std::error::Error;

use clap::ValueEnum;
use serde::Deserialize;
use struct_field_names_as_array::FieldNamesAsArray;

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
pub use charters::charters;
pub use comments::{comments, CommentField};
pub use issues_actions::{actions, get_repos, issues};
pub use locator::Locator;
use query::Query;
pub use specs::specs;
pub use status_labels::{
	CharterFromStrHelper, CharterLabels, CommentFromStrHelper, CommentLabels, StatusLabelInfo,
};

/// How the results should be shown
#[derive(Clone, ValueEnum)]
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
	Transform: Fn(ReturnedIssueType) -> Option<DomainType>,
>(
	name: &str,
	query: &mut Query,
	transform: Transform,
) -> Result<Vec<DomainType>, Box<dyn Error>> {
	Ok(query
		.run(name, ReturnedIssueType::FIELD_NAMES_AS_ARRAY.to_vec())?
		.into_iter()
		.flat_map(transform)
		.collect())
}

macro_rules! fetch_sort_print {
	($name:expr, $cell:ident, $query:ident, $transmogrify:ident, $get_sort_key:ident, $printer:ident) => {
		$cell.get_or_try_init(|| {
			fetch($name, &mut $query, $transmogrify).map(|mut items| {
				items.sort_by_key($get_sort_key);
				items
			})
		})?;
		$printer($cell.get().unwrap())
	};
	($name:expr, $cell:ident, $query:ident, $transmogrify:ident, $printer:ident) => {
		$cell.get_or_try_init(|| fetch($name, &mut $query, $transmogrify))?;
		$printer($cell.get().unwrap())
	};
}

macro_rules! simple_match {
   ($obj:expr, { $($matcher:pat => $result:expr),* $(,)? }) => {
       match $obj {
			$($matcher => $result),*,
			_ => unreachable!(),
       }
   }
}

macro_rules! fetch_sort_print_handler {
	($name:expr, $query:ident, $transmogrify:ident, $report_formats:ident, $($get_sort_key:ident,)? [ $printers:tt ]) => {
		use once_cell::sync::OnceCell;
		use crate::{fetch, fetch_sort_print, simple_match};

		let cell = OnceCell::new();
		for format in $report_formats {
			if matches!(format, ReportFormat::Web | ReportFormat::Gh) {
				$query.run_gh(matches!(format, ReportFormat::Web))
			} else {
				let printer: Box<dyn Fn(_)> = simple_match!(format, $printers);
				fetch_sort_print!($name, cell, $query, $transmogrify, $($get_sort_key,)? printer);
			}
		}
	};
}

pub(crate) use {fetch_sort_print, fetch_sort_print_handler, simple_match};
