#![warn(missing_docs)]
//! Provides functions to query GitHub for issues, actions, and horizontal spec review and issue
//! comment requests, according to W3C conventions.
//!
//! The `gh` command is used to actually make the queries. The output from `gh` is either printed
//! verbatim (in the case of issues), or obtained in JSON format, and processed extensively to add
//! more helpful information to it, to help group and TF chairs keep track of things.
//!
//! For info on how to use the tool based on this library, refer to [the Nu Tracker README on GitHub](https://github.com/matatk/nu-tracker/blob/main/README.md).
use std::error::Error;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

mod assignee_query;
mod charters;
mod comments_designs;
mod flatten_assignees;
mod generate_table;
mod issues_actions;
mod locator;
mod origin_query;
mod query;
pub mod repos;
mod returned_issue;
mod showing;
mod specs;
mod status_labels;

pub use assignee_query::AssigneeQuery;
pub use charters::charters;
pub use comments_designs::{comments, designs, CommentField, DesignField, DisplayableVec};
pub use issues_actions::{actions, issues, select_repos, SelectReposError};
pub use locator::Locator;
pub use origin_query::OriginQuery;
use query::Query;
use returned_issue::ReturnedIssue;
pub use specs::specs;
pub use status_labels::{CharterLabel, CommentLabel, DesignLabel, StatusLabel};

/// Represents the part of serialised files that indicates their version
#[derive(Serialize, Deserialize)]
pub struct Meta {
	/// File format version number
	version: u16,
}

impl Meta {
	/// Instantiate a 'meta' section
	#[must_use] pub fn new(version: u16) -> Self {
		Self { version }
	}
}

/// Converting something (e.g. an action, or spec review request) to `Vec<String>` (including all fields)
pub trait ToVecString {
	/// Convert completely
	fn to_vec_string(&self) -> Vec<String>;
}

/// Converting specific fields of something (e.g. a comment/design review request) to a `Vec<String>`
pub trait ToVecStringWithFields {
	/// A bit of info to be included (e.g. an enum variant specifying a field of the request)
	type Field;

	/// Convert specific fields
	fn to_vec_string(&self, fields: &[Self::Field]) -> Vec<String>;
}

/// How the results should be shown
#[derive(Clone, ValueEnum)]
pub enum ReportFormat {
	/// Print via GitHub CLI
	#[clap(hide(true))]
	Gh,
	/// Type out in totally tabular text
	Table,
	/// Subtopics and links, for pasting into IRC during a call
	Meeting,
	/// List, suitable for use in call announcements
	Agenda,
	/// Open the search query in GitHub in a browser
	Web,
}

fn fetch<ReturnedIssueType, DomainType, Transform: Fn(ReturnedIssueType) -> Option<DomainType>>(
	name: &str,
	query: &mut Query,
	transform: Transform,
) -> Result<Vec<DomainType>, Box<dyn Error>>
where
	ReturnedIssueType: ReturnedIssue + for<'a> Deserialize<'a>,
{
	Ok(query.run(name)?.into_iter().flat_map(transform).collect())
}

macro_rules! fetch_sort_print {
	($name:expr, $cell:ident, $query:ident, $transmogrify:ident, $get_sort_key:ident, $printer:ident) => {
		$cell.get_or_try_init(|| {
			crate::fetch($name, &mut $query, $transmogrify).map(|mut items| {
				items.sort_by_key($get_sort_key);
				items
			})
		})?;
		$printer($cell.get().unwrap())
	};
	($name:expr, $cell:ident, $query:ident, $transmogrify:ident, $printer:ident) => {
		$cell.get_or_try_init(|| crate::fetch($name, &mut $query, $transmogrify))?;
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
		// TODO: Use std instead
		let cell = ::once_cell::sync::OnceCell::new();
		for format in $report_formats {
			if matches!(format, ReportFormat::Web | ReportFormat::Gh) {
				$query.run_gh(matches!(format, ReportFormat::Web))
			} else {
				let printer: Box<dyn Fn(_)> = crate::simple_match!(format, $printers);
				crate::fetch_sort_print!($name, cell, $query, $transmogrify, $($get_sort_key,)? printer);
			}
		}
	};
}

pub(crate) use {fetch_sort_print, fetch_sort_print_handler, simple_match};
