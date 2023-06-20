use std::{env, fs, path::Path};

use regex::Regex;

const JSON_VERSION_PATTERN: &str = r#""version": ?(\d+)"#;

fn main() {
	create_repos_include();
	create_config_include();
}

fn create_repos_include() {
	const REPOS_JSON_FILE: &str = "repos.json";

	let re = Regex::new(JSON_VERSION_PATTERN).unwrap();
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

	save("repos_constants.rs", output, Some(REPOS_JSON_FILE));
}

fn create_config_include() {
	let output = format!(r##"const JSON_VERSION_PATTERN: &str = r#"{JSON_VERSION_PATTERN}"#;"##);
	save("config_constants.rs", output, None);
}

fn save(output_file_name: &str, data: String, source_file: Option<&str>) {
	fs::write(
		Path::new(&env::var("OUT_DIR").unwrap()).join(output_file_name),
		data,
	)
	.expect("should be able to write {file_name}");

	match source_file {
		Some(source) => println!("cargo:rerun-if-changed={source}"),
		None => println!("cargo:rerun-if-changed=build.rs"),
	}
}
