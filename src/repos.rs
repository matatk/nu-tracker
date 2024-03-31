//! Structured info on groups' repositories
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::Meta;

/// Errors that can happen when querying repository info
#[derive(Error, Debug)]
pub enum ReposError {
	/// Unknown/invalid group name
	#[error("Unknown group name '{group_name}'. Please consider contributing info for this group (use `nt config repos-info` to get known groups' details in JSON format). Known group names are: {}", valid_groups.join(", "))]
	// NOTE: SYNCH: invoke.rs
	InvalidGroup {
		/// The name of the group
		group_name: String,
		/// Known/valid group names
		valid_groups: Vec<String>, // TODO: Needs to be sorted.
	},
}

/// Holds all information on groups, the TFs they contain, and the repos belonging to all of them.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllGroupRepos {
	meta: Meta,
	repos: HashMap<String, GroupRepos>,
}

impl AllGroupRepos {
	/// Return a pretty serialised version of the default repositories info
	pub fn stringify(&self) -> Result<String, serde_json::Error> {
		// TODO: Capture an error (could be with custom JSON file) and give a bit more help.
		serde_json::to_string_pretty(&self)
	}

	/// Return the repositories for a given group
	pub fn for_group(&self, group: &str) -> Result<&GroupRepos, ReposError> {
		self.repos.get(group).ok_or(ReposError::InvalidGroup {
			group_name: group.to_string(),
			valid_groups: self.known_group_names(),
		})
	}

	fn known_group_names(&self) -> Vec<String> {
		let mut names: Vec<String> = self.repos.keys().map(|n| n.to_string()).collect();
		names.sort();
		names
	}
}

/// Contains horizontal review (if applicable), and repo info for a group, as well as repo info for
/// the group's contained TFs.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupRepos {
	pub(crate) group: MainAndOtherRepos,
	horizontal_review: Option<HorizontalReview>,
	pub(crate) task_forces: Option<HashMap<String, MainAndOtherRepos>>,
}

impl GroupRepos {
	/// Return this group's horizontal comment review repo (if applicable)
	pub fn hr_comments(&self) -> Option<&str> {
		self.horizontal_review.as_ref()?.comments.as_deref()
	}

	/// Return this group's horizontal design review repo (if applicable)
	pub fn hr_designs(&self) -> Option<&str> {
		self.horizontal_review.as_ref()?.designs.as_deref()
	}

	/// Return this group's horizontal spec review repo (if applicable)
	pub fn hr_specs(&self) -> Option<&str> {
		self.horizontal_review.as_ref()?.specs.as_deref()
	}
}

/// Provides URLs for the horizontal review repos for a group
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HorizontalReview {
	comments: Option<String>,
	designs: Option<String>,
	specs: Option<String>,
}

/// Provides URLs for the main and other repos for a group or TF
#[derive(Serialize, Deserialize)]
pub struct MainAndOtherRepos {
	pub(crate) main: String,
	pub(crate) others: Option<Vec<String>>,
}
