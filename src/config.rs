//! Loading, querying, modifying and saving known group and TF repos, and user settings.
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

mod repos;
mod settings;

pub use repos::{AllGroupRepos, GroupRepos, MainAndOtherRepos};
pub use settings::Settings;

/// Errors that can happen relating to config (repos, settings) info
#[derive(Error, Debug)]
pub enum ConfigError {
	/// An I/O error when trying to load/write a file, or create a directory
	#[error("IO: {0}")]
	IoError(#[from] std::io::Error),
	/// A deserialisation error
	#[error("JSON error in {source}: {details}\n\n{}", if let ConfigJsonErrorSource::File(_) = .source { "NOTE: This could be due to the file format changing. Migration is not currently implemented, but is on the roadmap. For now, you could try deleting the file and having Nu Tracker re-create it on next run. Sorry for the loss of any customisations you have made!"} else { "NOTE: This is a bug; please report it :-)." })]
	JsonError {
		/// Source of the error
		source: ConfigJsonErrorSource,
		/// More info (from serde)
		details: String,
	},
}

/// Where did a deserialisation error come from?
#[derive(Error, Debug)]
pub enum ConfigJsonErrorSource {
	// FIXME: Can't we check this at compile time? Then this isn't needed.
	/// The default data (i.e. for repos, which is done by including a repos.json file)
	#[error("internal data")]
	Internal,
	/// The file that the user provided (repos or settings)
	#[error("'{0}'")]
	File(PathBuf),
}

#[derive(Serialize, Deserialize)]
struct Meta {
	version: u16,
}

fn deserialise<T: for<'a> Deserialize<'a>>(
	json: String,
	file: &Option<PathBuf>,
) -> Result<T, ConfigError> {
	match serde_json::from_str(&json) {
		Ok(thing) => Ok(thing),
		Err(error) => Err(ConfigError::JsonError {
			source: match file {
				Some(path) => ConfigJsonErrorSource::File(path.clone()),
				None => ConfigJsonErrorSource::Internal,
			},
			details: error.to_string(),
		}),
	}
}
