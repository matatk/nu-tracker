//! Loading, querying, modifying and saving known WG and TF repos, and user settings.
use std::{
	error, fmt, fs,
	io::{self, Write},
	path::PathBuf,
};

use etcetera::base_strategy::{self, BaseStrategy};
use regex::Regex;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

mod repos;
mod settings;

pub use repos::{Repos, WgOrTfRepos, WorkingGroupInfo};
pub use settings::Settings;

const APP_DIR: &str = "nu-tracker";

pub enum ConfigError {
	DirNope,
	IoError(io::ErrorKind),
	JsonError { file_name: String, details: String },
	JsonMissingVersion(String),
}

impl error::Error for ConfigError {}

impl fmt::Debug for ConfigError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self)?;
		Ok(())
	}
}

impl fmt::Display for ConfigError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self {
			ConfigError::DirNope => write!(f, "Directory creation was cancelled.")?,
			ConfigError::IoError(kind) => write!(f, "IO: {kind}")?,
			ConfigError::JsonError { file_name, details } => {
				write!(f, "JSON: {file_name}: {details}")?
			}
			ConfigError::JsonMissingVersion(file_name) => {
				write!(f, "Can't find version number in {file_name}")?
			}
		}
		Ok(())
	}
}

impl From<io::Error> for ConfigError {
	fn from(error: io::Error) -> Self {
		ConfigError::IoError(error.kind())
	}
}

enum InitialContent {
	Static(String),
	Creator(fn() -> String),
}

#[derive(Serialize, Deserialize)]
struct Meta {
	version: u16,
}

/// Return the in-use config directory
pub fn config_dir() -> PathBuf {
	for dir in default_dirs() {
		if dir.exists() {
			return dir;
		}
	}

	panic!("should be able to find a config dir—ensure_dir() should've been called");
}

// TODO: How/if to test this?
// TODO: Simplify by returning Paths (which are like &PathBufs)?
/// Ensure that the config file directory exists
///
/// If no config file directory exists, this will create one, after asking the user for
/// confirmation.
pub fn ensure_dir() -> Result<(), ConfigError> {
	let candidates = default_dirs();

	for dir in &candidates {
		if dir.exists() {
			return Ok(());
		}
	}

	println!("Nu Tracker needs to create a config file directory.\n");

	if candidates.len() == 1 {
		println!("Config directory: {:?}\n", &candidates[0]);
		print!("The direcotry doesn't exist; create it?");
	} else {
		println!("Possible directories:");
		for (index, directory) in candidates.iter().enumerate() {
			println!("{index}: {directory:?}");
		}
		print!(
			"\nEnter the number for the path you'd like to create, \
                 or press ENTER/RETURN to exit."
		);
	}

	let targ: Option<&PathBuf> = if candidates.len() == 1 {
		if get_input(" [y/*]") == "y" {
			Some(&candidates[0])
		} else {
			None
		}
	} else {
		get_path_index(&candidates)
	};

	if let Some(dir) = targ {
		Ok(std::fs::create_dir_all(dir)?)
	} else {
		Err(ConfigError::DirNope)
	}
}

/// Return a list of the default config file directories for the current platform.
///
/// On MacOS, this returns the platform default one, plus the XDG one, which a number of *nix
/// tools use, so the user can choose which they want to use.
///
/// On other platforms this just returns a one-path Vec: the platform-default one.
///
/// After creation, [`config_dir()`] can be called to find the in-use path.
fn default_dirs() -> Vec<PathBuf> {
	let mut candidates = vec![dirs::config_dir()
		.expect("should be able to figure out platform config path")
		.join(APP_DIR)];

	#[cfg(target_os = "macos")]
	{
		let xdg = base_strategy::choose_base_strategy()
			.unwrap()
			.config_dir()
			.join(APP_DIR);

		candidates.push(xdg);
	}

	candidates
}

fn get_input(prompt: &str) -> String {
	let mut input = String::new();

	print!("{prompt} ");
	let _ = std::io::stdout().flush();

	std::io::stdin()
		.read_line(&mut input)
		.expect("should be able to read user's input");

	input.trim().to_string()
}

fn get_path_index(candidates: &[PathBuf]) -> Option<&PathBuf> {
	loop {
		let input = get_input("\nCreate path:");
		if !input.is_empty() {
			if let Ok(input) = input.trim().parse::<usize>() {
				let chosen = &candidates.get(input);
				if chosen.is_some() {
					break *chosen;
				} else {
					print!("Invalid path number.")
				}
			} else {
				print!("Please enter a path number, or press ENTER/RETURN to exit.")
			}
		} else {
			return None;
		}
	}
}

fn get_or_create<T>(
	file_name: &str,
	data_name: &str,
	init: InitialContent,
	current_version: &str,
	mismatch_severity: &str,
	mismatch_message: Option<String>,
) -> Result<T, ConfigError>
where
	T: DeserializeOwned,
{
	let file_path = config_dir().join(file_name);

	if !file_path.exists() {
		let content = match init {
			InitialContent::Static(content) => content,
			InitialContent::Creator(callable) => callable(),
		};

		std::fs::write(&file_path, content)?;
		println!("Saved default {data_name} file as: {file_path:?}");
	};

	let json_string = fs::read_to_string(file_path)?;
	check_version(
		current_version,
		file_name,
		&json_string,
		mismatch_severity,
		mismatch_message,
	)?;

	match serde_json::from_str::<T>(&json_string) {
		Ok(thing) => Ok(thing),
		Err(error) => Err(ConfigError::JsonError {
			file_name: file_name.to_string(),
			details: error.to_string(),
		}),
	}
}

// NOTE: Current version is str because we get it from the file directly.
fn check_version(
	current_version: &str,
	file_name: &str,
	json: &str,
	mismatch_severity: &str,
	mismatch_message: Option<String>,
) -> Result<(), ConfigError> {
	// FIXME: DRY with build.rs
	let re = Regex::new(r#""version": ?(\d+)"#).unwrap();

	if let Some(caps) = re.captures(json) {
		let json_version = caps.get(1).map_or("", |m| m.as_str());
		if json_version != current_version {
			print!(
				"{mismatch_severity}: Your version of {file_name} \
({json_version}) doesn't match the latest version ({current_version})."
			);
			if let Some(extra) = mismatch_message {
				println!(" {extra}\n");
			} else {
				println!("\n");
			}
		}
	} else {
		return Err(ConfigError::JsonMissingVersion(file_name.to_string()));
	}

	Ok(())
}
