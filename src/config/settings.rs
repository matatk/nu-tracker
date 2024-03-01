use std::{fs, path::PathBuf};

use etcetera::base_strategy::{choose_base_strategy, BaseStrategy};
use serde::{Deserialize, Serialize};

use super::{deserialise, ConfigError, Meta};

/// Holds user settings
#[derive(Serialize, Deserialize)]
pub struct Settings {
	meta: Meta,
	conf: UserSettings,
	#[serde(skip)]
	modified: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserSettings {
	working_group: String,
}

impl Settings {
	const APP_DIR: &'static str = "nu-tracker";
	const FILE_NAME: &'static str = "settings.json";
	const CURRENT_VERSION: u16 = 1;

	/// Load the user's settings from a file, or create a default settings struct (to be saved later)
	pub fn load_or_init() -> Result<Self, ConfigError> {
		let path = Self::settings_file_path();
		if path.exists() {
			deserialise(fs::read_to_string(&path)?, Some(path))
		} else {
			println!("The default group is set to 'apa' - this can be changed using the `nt config` sub-command.");
			Ok(Settings {
				meta: Meta {
					version: Self::CURRENT_VERSION,
				},
				conf: UserSettings {
					working_group: String::from("apa"),
				},
				modified: true,
			})
		}
	}

	/// Serialise the current settings struct (if it has been modified or created on this run)
	// NOTE: Assumes that the dir and file exist, because this will be called after get_settings()
	pub fn save(&self, verbose: bool) -> Result<(), ConfigError> {
		let path = Self::settings_file_path();
		if verbose {
			if self.modified {
				println!("Saving settings ({path:?})")
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
	pub fn wg(&self) -> String {
		self.conf.working_group.clone()
	}

	/// Set the group (but don't save)
	// NOTE: Doesn't save - that function needs to be called before program exit
	pub fn set_wg(&mut self, wg: String) {
		if self.conf.working_group == wg {
			println!("Default WG is already '{}'", &self.conf.working_group)
		} else {
			self.conf.working_group = wg.to_string();
			println!("Default WG is now '{}'", &self.conf.working_group);
			self.modified = true
		}
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
