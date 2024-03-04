use std::{collections::HashMap, error::Error, fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{deserialise, ConfigError, Meta};

#[cfg(not(windows))]
macro_rules! sep {
	() => {
		"/"
	};
}

#[cfg(windows)]
macro_rules! sep {
	() => {
		r#"\"#
	};
}

#[derive(Error, Debug)]
pub enum ReposError {
	#[error("Unknown group name '{group_name}'. Please consider contributing info for this group. Known group names:\n{}", valid_groups.join("\n"))]
	InvalidGroup {
		group_name: String,
		/// Needs to be sorted
		valid_groups: Vec<String>,
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
	/// Load a custom repositories file, or use the default one
	pub fn load_or_init(file: &Option<PathBuf>, verbose: &bool) -> Result<Self, ConfigError> {
		let json_string = if let Some(ref path) = file {
			if *verbose {
				println!("Loading repos info from {path:?}")
			}
			fs::read_to_string(path)?
		} else {
			// TODO: Build a Rust literal from the JSON file at compile time?
			include_str!(concat!(
				".",
				sep!(),
				"..",
				sep!(),
				"..",
				sep!(),
				"repos.json"
			))
			.to_string()
		};
		deserialise(json_string, file)
	}

	/// Return a pretty serialised version of the default repositories info
	pub fn stringify(&self) -> Result<String, Box<dyn Error>> {
		// FIXME: Use ConfigError::Json
		Ok(serde_json::to_string_pretty(&self)?)
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
	/// HR repos
	pub horizontal_review: Option<HorizontalReview>,
	/// The group's repos
	pub group: MainAndOtherRepos,
	/// The group's TF's, and their repos
	pub task_forces: Option<HashMap<String, MainAndOtherRepos>>,
}

/// Provides URLs for the horizontal review repos for a group
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HorizontalReview {
	pub specs: String,
	pub comments: String,
}

/// Provides URLs for the main and other repos for a group or TF
#[derive(Serialize, Deserialize)]
pub struct MainAndOtherRepos {
	/// The repo where actions will be created
	pub main: String,
	/// Other repos this group or TF has
	pub others: Option<Vec<String>>,
}
