use serde::{Deserialize, Serialize};

use super::{config_dir, get_input, get_or_create, InitialContent, Meta};

const FILE_NAME: &str = "settings.json";
const CURRENT_VERSION: u16 = 1;

/// Holds user settings
#[derive(Serialize, Deserialize)]
pub struct Settings {
	meta: Meta,
	conf: UserSettings,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserSettings {
	working_group: String,
}

impl Settings {
	/// Load an existing settings JSON file, or create and save one, with the user's input, and then return it
	pub fn load_or_init() -> Result<Self, super::ConfigError> {
		get_or_create(
			FILE_NAME,
			"settings",
			InitialContent::Creator(init),
			format!("{CURRENT_VERSION}").as_str(),
			"WARNING",
			None,
		)
	}

	// NOTE: Assumes that the dir and file exist, because this will be called after get_settings()
	pub fn save(&self) {
		std::fs::write(
			config_dir().join(FILE_NAME),
			serde_json::to_string_pretty(&self).expect("should be able to serialise settings"),
		)
		.unwrap_or_else(|_| panic!("should be able to write {FILE_NAME}"));
	}

	pub fn wg(&self) -> &String {
		&self.conf.working_group
	}

	// FIXME: Move UI back to main.rs
	// FIXME: return the mutated self?
	pub fn set_wg(&mut self, wg: String, valid_wgs: &[&String]) {
		if self.conf.working_group == wg {
			println!("Default WG is already '{}'", &self.conf.working_group)
		} else if valid_wgs.contains(&&wg) {
			self.conf.working_group = wg;
			self.save();
			println!("Default WG is now '{}'", &self.conf.working_group)
		} else {
			println!("Unknown WG name: '{wg}' - not changing setting");
			return;
		}
	}
}

fn init() -> String {
	let initial = Settings {
		meta: Meta {
			version: CURRENT_VERSION,
		},
		conf: UserSettings {
			working_group: get_group(),
		},
	};

	serde_json::to_string_pretty(&initial).expect("should be able to serialise Settings")
}

fn get_group() -> String {
	let mut answer: String;

	loop {
		answer = get_input("Default working group?");
		if !answer.is_empty() {
			// FIXME: check for valid group
			break;
		}
	}

	answer
}
