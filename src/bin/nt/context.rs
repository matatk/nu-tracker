use std::{error::Error, fs, path::PathBuf};

use etcetera::base_strategy::{choose_base_strategy, BaseStrategy};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use ntlib::{
	repos::{AllGroupRepos, GroupRepos},
	CommentField, DesignField, DisplayableVec, Meta,
};

#[derive(Error, Debug)]
pub enum ContextError {
	#[error("IO: {0}")]
	IoError(#[from] std::io::Error),
	#[error("JSON error in {source}: {details}\n\n{}", if let ContextJsonErrorSource::File(_) = .source { "NOTE: This could be due to the file format changing. Migration is not currently implemented, but is on the roadmap. For now, you could try deleting the file and having Nu Tracker re-create it on next run. Sorry for the loss of any customisations you have made!"} else { "NOTE: This is a bug; please report it :-)." })]
	JsonError {
		source: ContextJsonErrorSource,
		details: String,
	},
	#[error(
		"No group name specified - this must be given either on the command line (via the `--as` option), or in the settings file (use `nt config group` to set it)."  // NOTE: SYNCH: invoke.rs
	)]
	MissingGroup,
}

#[derive(Error, Debug)]
pub enum ContextJsonErrorSource {
	#[error("internal data")]
	Internal,
	#[error("'{0}'")]
	File(PathBuf),
}

pub struct Context {
	cli_group: Option<String>,
	settings_file: SettingsFile,
	repos: AllGroupRepos,
}

impl Context {
	pub fn new(
		cli_group: Option<String>,
		custom_repos_file: Option<PathBuf>,
		verbose: bool,
	) -> Result<Self, Box<dyn Error>> {
		Ok(Self {
			cli_group,
			settings_file: SettingsFile::load_or_init(verbose)?,
			repos: load_or_init_repos_info(custom_repos_file, verbose)?, // TODO: Still too much UI?
		})
	}

	pub fn group_name(&self) -> Result<String, ContextError> {
		// FIXME: idiomaticness
		if let Some(group) = self.cli_group.clone() {
			// TODO: remove need for clone?
			Ok(group)
		} else if let Some(group) = self.settings_file.settings().group() {
			Ok(group)
		} else {
			Err(ContextError::MissingGroup)
		}
	}

	pub fn is_group_name_overridden(&self) -> bool {
		self.cli_group.is_some()
	}

	pub fn group_repos(&self) -> Result<&GroupRepos, Box<dyn Error>> {
		Ok(self.all_group_repos().for_group(&self.group_name()?)?)
	}

	pub fn all_group_repos(&self) -> &AllGroupRepos {
		&self.repos
	}

	pub fn settings(&self) -> &Settings {
		self.settings_file.settings()
	}

	pub fn settings_mut(&mut self) -> &mut Settings {
		self.settings_file.settings_mut()
	}

	pub fn config_dir() -> PathBuf {
		SettingsFile::config_dir()
	}
}

#[derive(Serialize, Deserialize)]
struct SettingsFile {
	meta: Meta,
	conf: Settings,
	#[serde(skip)]
	verbose: bool,
}

impl SettingsFile {
	const APP_DIR: &'static str = "nu-tracker";
	const FILE_NAME: &'static str = "settings.json";
	const CURRENT_VERSION: u16 = 1;

	pub fn load_or_init(verbose: bool) -> Result<Self, ContextError> {
		let path = Self::settings_file_path();
		if path.exists() {
			if verbose {
				println!("Loading settings file: {path:?}")
			}
			let mut elf = deserialise::<Self>(fs::read_to_string(&path)?, Some(path))?;
			elf.verbose = verbose;
			Ok(elf)
		} else {
			if verbose {
				println!("No settings file; using default settings where possible.")
			}
			Ok(Self {
				meta: Meta::new(Self::CURRENT_VERSION),
				conf: Settings::default(),
				verbose,
			})
		}
	}

	pub fn settings(&self) -> &Settings {
		&self.conf
	}

	pub fn settings_mut(&mut self) -> &mut Settings {
		&mut self.conf
	}

	// NOTE: Assumes that the dir and file exist, because this will be called after get_settings()
	pub fn save(&self) -> Result<(), ContextError> {
		let path = Self::settings_file_path();

		if self.conf.modified() {
			// FIXME: This is UI
			if self.verbose {
				println!("Saving settings ('{}')", path.display());
			}
			Self::ensure_dir()?;
			std::fs::write(
				path,
				serde_json::to_string_pretty(&self).expect("should be able to serialise settings"),
			)?;
		} else {
			// FIXME: This is UI
			if self.verbose {
				println!("Not saving settings.");
			}
		}

		Ok(())
	}

	pub fn config_dir() -> PathBuf {
		choose_base_strategy()
			.unwrap()
			.config_dir()
			.join(Self::APP_DIR)
	}

	fn ensure_dir() -> Result<(), ContextError> {
		let default = Self::config_dir();
		if default.exists() {
			Ok(())
		} else {
			Ok(std::fs::create_dir_all(default)?)
		}
	}

	fn settings_file_path() -> PathBuf {
		Self::config_dir().join(Self::FILE_NAME)
	}
}

impl Drop for SettingsFile {
	fn drop(&mut self) {
		if let Err(error) = self.save() {
			println!("Error encountered whilst saving settings: {error}");
		}
	}
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
	#[serde(skip_serializing_if = "Option::is_none")]
	group: Option<String>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	comment_columns: Vec<CommentField>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	design_columns: Vec<DesignField>,
	#[serde(skip)]
	modified: bool,
}

fn fallback_comment_columns() -> Vec<CommentField> {
	vec![
		CommentField::Id,
		CommentField::Title,
		CommentField::Group,
		CommentField::Spec,
		CommentField::Status,
		CommentField::Assignees,
		CommentField::Our,
	]
}

fn fallback_design_columns() -> Vec<DesignField> {
	vec![
		DesignField::Id,
		DesignField::Title,
		DesignField::Group,
		DesignField::Spec,
		DesignField::Status,
		DesignField::Assignees,
	]
}

impl Settings {
	// TODO: Would be nice to make this &str but then have to figure out how to get &str from clap
	pub fn group(&self) -> Option<String> {
		self.group.clone()
	}

	pub fn set_group(&mut self, group: String) {
		self.group = Some(group.to_string());
		self.modified = true;
	}

	pub fn comment_columns(&self) -> Vec<CommentField> {
		if self.comment_columns.is_empty() {
			fallback_comment_columns()
		} else {
			self.comment_columns.clone()
		}
	}

	// TODO: check for similarity before setting
	pub fn set_comment_columns(&mut self, fields: Vec<CommentField>) {
		self.comment_columns = fields;
		// FIXME: move UI to main.rs?
		println!(
			"Default comment fields are now: {}",
			// TODO: Remove the need for the clone
			DisplayableVec::from(self.comment_columns.clone())
		);
		self.modified = true;
	}

	pub fn design_columns(&self) -> Vec<DesignField> {
		if self.design_columns.is_empty() {
			fallback_design_columns()
		} else {
			self.design_columns.clone()
		}
	}

	// TODO: check for similarity before setting
	pub fn set_design_columns(&mut self, fields: Vec<DesignField>) {
		self.design_columns = fields;
		// FIXME: move UI to main.rs?
		println!(
			"Default design fields are now: {}",
			// TODO: Remove the need for the clone
			DisplayableVec::from(self.design_columns.clone())
		);
		self.modified = true;
	}

	pub fn modified(&self) -> bool {
		self.modified
	}
}

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

fn load_or_init_repos_info(
	file: Option<PathBuf>,
	verbose: bool,
) -> Result<AllGroupRepos, ContextError> {
	let json_string = if let Some(ref path) = file {
		if verbose {
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
			"..",
			sep!(),
			"repos.json"
		))
		.to_string()
	};
	deserialise(json_string, file)
}

fn deserialise<T: for<'a> Deserialize<'a>>(
	json: String,
	file: Option<PathBuf>,
) -> Result<T, ContextError> {
	match serde_json::from_str(&json) {
		Ok(thing) => Ok(thing),
		Err(error) => Err(ContextError::JsonError {
			source: match file {
				Some(path) => ContextJsonErrorSource::File(path.clone()),
				None => ContextJsonErrorSource::Internal,
			},
			details: error.to_string(),
		}),
	}
}

// TODO: Not sure really how useful these tests are for now, other than to
//       remind me what the expected behaviour is :-). However, they may be
//       useful as a/ basis for adding migration on top (eek).
#[cfg(test)]
mod tests {
	use ntlib::{CommentField, DesignField};

	use super::*;

	#[test]
	fn all_settings() {
		let fixture = r#"{
			"meta": {
				"version": 1
			},
			"conf": {
				"group": "apa42",
				"commentColumns": [ "id", "our" ],
				"designColumns": [ "title", "assignees" ]
			}
		}"#;
		let result = deserialise::<SettingsFile>(fixture.into(), None)
			.expect("fixture with all settings should parse");
		let settings = result.settings();

		assert_eq!(settings.group(), Some("apa42".into()));
		assert_eq!(
			settings.comment_columns(),
			vec![CommentField::Id, CommentField::Our]
		);
		assert_eq!(
			settings.design_columns(),
			vec![DesignField::Title, DesignField::Assignees]
		);
	}

	// NOTE: Not using the Default trait for 'fallback' values, so that defaults
	//       don't get needlessly serialised.
	#[test]
	fn missing_settings_use_useful_fallbacks() {
		let fixture = r#"{
			"meta": {
				"version": 1
			},
			"conf": {}
		}"#;
		let result = deserialise::<SettingsFile>(fixture.into(), None)
			.expect("fixture with no settings should parse");
		let settings = result.settings();

		assert_eq!(settings.group(), None);
		assert_eq!(settings.comment_columns(), fallback_comment_columns());
		assert_eq!(settings.design_columns(), fallback_design_columns());
	}
}
