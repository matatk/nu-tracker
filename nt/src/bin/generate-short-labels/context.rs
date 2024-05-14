// FIXME: Sorta DRY?
use std::{error::Error, fs, path::PathBuf};

use ntlib::repos::AllGroupRepos;

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

pub fn load_or_init_repos_info(
	file: Option<PathBuf>,
	verbose: bool,
) -> Result<AllGroupRepos, Box<dyn Error>> {
	if let Some(ref path) = file {
		if verbose {
			println!("Loading repos info from {path:?}");
		}
		Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
	} else {
		// TODO: Build a Rust literal from the JSON file at compile time?
		Ok(serde_json::from_str(include_str!(concat!(
			".",
			sep!(),
			"..",
			sep!(),
			"..",
			sep!(),
			"..",
			sep!(),
			"repos.json"
		)))?)
	}
}
