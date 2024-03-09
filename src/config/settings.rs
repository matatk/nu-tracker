use std::{fs, path::PathBuf};

use etcetera::base_strategy::{choose_base_strategy, BaseStrategy};
use serde::{Deserialize, Serialize};

use super::{deserialise, ConfigError, Meta};
use crate::{CommentField, DesignField, DisplayableCommentFieldVec};

/// Holds user settings
#[derive(Serialize, Deserialize)]
pub struct Settings {
	meta: Meta,
	conf: UserSettings,
	#[serde(skip)]
	modified: bool,
	#[serde(skip)]
	verbose: bool,
}

// FIXME: Structure properly
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserSettings {
	group: String,
	comment_fields: Vec<CommentField>,
	design_fields: Vec<DesignField>,
}

impl Drop for Settings {
	fn drop(&mut self) {
		if let Err(error) = self.save() {
			println!("Error encountered whilst saving settings: {error}");
		}
	}
}

impl Settings {
	const APP_DIR: &'static str = "nu-tracker";
	const FILE_NAME: &'static str = "settings.json";
	const CURRENT_VERSION: u16 = 1;

	/// Load the user's settings from a file, or create a default settings struct (to be saved later)
	pub fn load_or_init(verbose: bool) -> Result<Self, ConfigError> {
		let path = Self::settings_file_path();
		if path.exists() {
			deserialise(fs::read_to_string(&path)?, &Some(path))
		} else {
			// FIXME: This is UI; shouldn't be here.
			println!("The default group is set to 'apa' - this can be changed using the `nt config group` sub-command.");
			Ok(Settings {
				meta: Meta {
					version: Self::CURRENT_VERSION,
				},
				conf: UserSettings {
					group: String::from("apa"),
					comment_fields: vec![
						CommentField::Id,
						CommentField::Title,
						CommentField::Group,
						CommentField::Spec,
						CommentField::Status,
						CommentField::Assignees,
						CommentField::Our,
					],
					design_fields: vec![
						DesignField::Id,
						DesignField::Title,
						DesignField::Group,
						DesignField::Spec,
						DesignField::Status,
						DesignField::Assignees,
					],
				},
				modified: true, // NOTE: Must be true, or the UI message above will be displayed on each run, until a setting is customised.
				verbose,
			})
		}
	}

	/// Serialise the current settings struct (if it has been modified or created on this run)
	// NOTE: Assumes that the dir and file exist, because this will be called after get_settings()
	pub fn save(&self) -> Result<(), ConfigError> {
		let path = Self::settings_file_path();
		// TODO: This is UI
		if self.verbose {
			if self.modified {
				println!("Saving settings ('{}')", path.display())
			} else {
				println!("Not saving settings")
			}
		}
		if self.modified {
			Self::ensure_dir()?;
			std::fs::write(
				path,
				serde_json::to_string_pretty(&self).expect("should be able to serialise settings"),
			)?
		}
		Ok(())
	}

	/// Get the current group from the settings
	// TODO: Would be nice to make this &str but then have to figure out how to get &str from clap
	pub fn group(&self) -> String {
		self.conf.group.clone()
	}

	/// Set the group (but don't save)
	// NOTE: Doesn't save - that function needs to be called before program exit
	// TODO: Move the messages out of here; UI. Their counterparts are already in main.
	pub fn set_group(&mut self, group: String) {
		if self.conf.group == group {
			println!("Default group is already '{}'", &self.conf.group)
		} else {
			self.conf.group = group.to_string();
			println!("Default group is now '{}'", &self.conf.group);
			self.modified = true
		}
	}

	/// Get the order of fields/columns for the comments table
	pub fn comment_fields(&self) -> Vec<CommentField> {
		self.conf.comment_fields.clone()
	}

	/// Get the order of fields/columns for the designs table
	pub fn design_fields(&self) -> Vec<DesignField> {
		self.conf.design_fields.clone()
	}

	/// Set the order of fields/columns for the comments table
	// NOTE: Doesn't save - that function needs to be called before program exit
	// TODO: check for similarity before setting
	pub fn set_comment_fields(&mut self, fields: Vec<CommentField>) {
		self.conf.comment_fields = fields;
		println!(
			"Default comment fields are now: {}",
			// TODO: Remove the need for the clone
			DisplayableCommentFieldVec::from(self.conf.comment_fields.clone())
		);
		self.modified = true
	}

	/// Return the directory where the settings file will be written
	pub fn config_dir() -> PathBuf {
		choose_base_strategy()
			.unwrap()
			.config_dir()
			.join(Self::APP_DIR)
	}

	fn ensure_dir() -> Result<(), ConfigError> {
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
