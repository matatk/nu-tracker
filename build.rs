use std::{env, fs, path::Path};

use regex::Regex;

const REPOS_JSON_FILE: &str = "repos.json";
const REPOS_CONSTANTS_FILE: &str = "repos_constants.rs";

fn main() {
	// FIXME: DRY with config.rs
	let re = Regex::new(r#""version": ?(\d+)"#).unwrap();
	let json_string =
		fs::read_to_string(REPOS_JSON_FILE).expect("should be able to read {REPOS_JSON_FILE}");
	let repos_version = re
		.captures(&json_string)
		.expect("regex should match")
		.get(1)
		.expect("version should be found in JSON file")
		.as_str();

	let output = format!(
		r#"const FILE_NAME: &str = "{REPOS_JSON_FILE}";
const DEFAULT_REPOS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/{REPOS_JSON_FILE}"));
const CURRENT_VERSION: &str = "{repos_version}";"#
	);

	fs::write(
		Path::new(&env::var("OUT_DIR").unwrap()).join(REPOS_CONSTANTS_FILE),
		output,
	)
	.expect("should be able to write {REPOS_CONSTANTS_FILE}");

	println!("cargo:rerun-if-changed={REPOS_JSON_FILE}");
}
