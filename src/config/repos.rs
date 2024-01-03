use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{config_dir, get_or_create, InitialContent, Meta};

include!(concat!(env!("OUT_DIR"), "/repos_constants.rs"));

/// Holds all information on WGs, the TFs they contain, and the repos belonging to all of them.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repos {
	meta: Meta,
	repos: HashMap<String, WorkingGroupInfo>,
}

impl Repos {
	/// Load an existing repos JSON file, or create and save one, and then return it
	pub fn load_or_init() -> Result<Self, super::ConfigError> {
		get_or_create(
			FILE_NAME,
			"repos",
			InitialContent::Static(DEFAULT_REPOS.to_string()),
			CURRENT_VERSION,
			"NOTE",
			Some(format!("You may want to contribute any additions you made, and then delete your {:?} file (a new version will be written on the next run).", config_dir()?.join(FILE_NAME)))
		)
	}

	pub fn wgs_repos(&self) -> &HashMap<String, WorkingGroupInfo> {
		&self.repos
	}

	pub fn known_wg_names(&self) -> Vec<&String> {
		self.repos.keys().collect()
	}

	pub fn is_known_wg(&self, name: &str) -> bool {
		let valid_names = &self.known_wg_names();
		valid_names.contains(&&name.to_string())
	}
}

/// Contains horizontal review (if applicable), and repo info for a WG, as well as repo info for
/// the WG's contained TFs.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkingGroupInfo {
	pub horizontal_review: Option<HorizontalReview>,
	pub working_group: WgOrTfRepos,
	pub task_forces: HashMap<String, WgOrTfRepos>,
}

/// Provides URLs for the horizontal review repos for a WG
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HorizontalReview {
	pub specs: String,
	pub comments: String,
}

/// Provides URLs for the main and other repos for a WG or TF
#[derive(Serialize, Deserialize)]
pub struct WgOrTfRepos {
	pub main: String,
	pub others: Option<Vec<String>>,
}
