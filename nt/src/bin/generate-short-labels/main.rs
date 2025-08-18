use std::{env, error::Error, fmt::Display, fs, path::PathBuf, process::Command, str};

use serde::{Deserialize, Serialize};
use thiserror::Error;

mod context;

#[derive(PartialEq)]
enum Action {
	Help,
	DetailedList,
	LightList,
	WriteFiles,
}

// FIXME: remove need for Clone
// NOTE: Needed for sorting only: Ord, Eq, PartialOrd, PartialEq
#[derive(Serialize, Deserialize, Debug, Clone, Ord, Eq, PartialEq, PartialOrd)]
struct Label {
	name: String,
	description: String,
}

impl Display for Label {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.name)
	}
}

struct Labels(Vec<Label>);

impl Labels {
	const fn len(&self) -> usize {
		self.0.len()
	}
}

impl IntoIterator for Labels {
	type Item = Label;

	type IntoIter = <Vec<Label> as IntoIterator>::IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl Display for Labels {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			self.0
				.iter()
				.map(std::string::ToString::to_string)
				.collect::<Vec<_>>()
				.join("\n")
		)
	}
}

#[derive(Debug, PartialEq, Serialize)]
enum SourceLabel {
	GroupLabel {
		prefix: Option<String>,
		name: String,
		description: String,
	},
	SpecLabel {
		prefix: String,
		name: String,
		description: String,
	},
}

#[derive(Error, Debug)]
#[error("couldn't convert")]
struct SourceLabelError {}

impl TryFrom<Label> for SourceLabel {
	type Error = SourceLabelError;

	fn try_from(value: Label) -> Result<Self, Self::Error> {
		match value.name.split_once(':') {
			Some((head, tail)) => match head {
				"Venue" | "venue" | "wg" | "cg" | "ig" => Ok(Self::GroupLabel {
					prefix: Some(head.into()),
					name: tail.into(),
					description: value.description,
				}),
				"s" => Ok(Self::SpecLabel {
					prefix: head.into(),
					name: tail.into(),
					description: value.description,
				}),
				"Topic" | "Provenance" => Ok(Self::SpecLabel {
					prefix: head.into(),
					name: tail.trim_start().into(),
					description: value.description,
				}),
				_ => Err(Self::Error {}),
			},
			None => {
				if value.name == "ietf" || value.name == "whatwg" {
					Ok(Self::GroupLabel {
						prefix: None,
						name: value.name,
						description: value.description,
					})
				} else {
					Err(Self::Error {})
				}
			}
		}
	}
}

impl Display for SourceLabel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::GroupLabel {
				prefix,
				name,
				description,
			} => {
				if let Some(prefix) = prefix {
					write!(f, "{prefix}:{name}")
				} else {
					write!(f, "{name}")
				}
			}
			Self::SpecLabel {
				prefix,
				name,
				description,
			} => write!(f, "{prefix}:{name}"),
		}
	}
}

#[derive(Debug, PartialEq, Serialize)]
struct StatusLabel {
	prefix: Option<String>,
	name: String,
	description: String,
}

impl From<Label> for StatusLabel {
	fn from(value: Label) -> Self {
		match value.name.split_once(": ") {
			Some((head, tail)) => Self {
				prefix: Some(head.into()),
				name: tail.into(),
				description: value.description,
			},
			None => Self {
				prefix: None,
				name: value.name,
				description: value.description,
			},
		}
	}
}

impl Display for StatusLabel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Some(ref prefix) = self.prefix {
			write!(f, "{prefix}:{}", self.name)
		} else {
			write!(f, "{}", self.name)
		}
	}
}

#[derive(Serialize)]
struct ShortLabels {
	source: Vec<(String, SourceLabel)>,
	status: Vec<(String, StatusLabel)>,
}

fn main() {
	if let Err(error) = run() {
		println!("Error: {error}");
	}
}

fn run() -> Result<(), Box<dyn Error>> {
	let repos = context::load_or_init_repos_info(None, true)?;

	let action = match env::args().nth(1) {
		Some(thingy) => match thingy.as_str() {
			"list" | "l" => Action::LightList,
			"detaileds" | "d" => Action::DetailedList,
			"write" | "w" => Action::WriteFiles,
			_ => Action::Help,
		},
		None => Action::Help,
	};

	if action == Action::Help {
		println!("Usage: generate-short-labels <ACTION>");
		println!("Actions:");
		println!("l, list     List GitHub labels for each repo");
		println!("d, details  Verbose label-parsing for each repo");
		println!("w, write    Save verbose parsong JSON for each repo");
		return Ok(());
	}

	for group in repos.known_group_names() {
		let repos_for_group = repos.for_group(&group)?;
		for repo_opt in [repos_for_group.hr_comments(), repos_for_group.hr_designs()] {
			if let Some(repo) = repo_opt {
				println!("{group}: {repo}");

				let labels_file = labels_file_path(&repo);

				if action == Action::WriteFiles && labels_file.exists() {
					println!("{labels_file:?} already exists - skipping");
					continue;
				}

				let labels = get_repo_labels(repo)?;

				if action == Action::DetailedList || action == Action::WriteFiles {
					let categorised_labels = categorise_labels(labels);
					let labels_string = serde_json::to_string_pretty(&categorised_labels)?;

					if action == Action::DetailedList {
						println!("{labels_string}")
					} else {
						fs::write(labels_file, labels_string)?;
					}
				} else {
					println!("{} labels\n{}\n", labels.len(), labels);
				}
			}
		}
	}

	Ok(())
}

fn labels_file_path(repo: &str) -> PathBuf {
	let file_name = format!("{}.json", repo_to_filename(repo));
	let mut path = PathBuf::new();
	path.push(file_name);
	path
}

fn repo_to_filename(repo: &str) -> String {
	repo.replace('/', "-")
}

fn get_repo_labels(repo: &str) -> Result<Labels, Box<dyn Error>> {
	let mut cmd = Command::new("gh");
	cmd.args([
		"label",
		"list",
		"-R",
		repo,
		"-L",
		"999",
		"--json",
		"name,description",
	]);

	let mut labels: Vec<Label> = serde_json::from_str(
		str::from_utf8(&cmd.output().expect("gh should run").stdout)
			.expect("gh should produce utf-8 output"),
	)?;

	labels.sort();

	Ok(Labels(labels))
}

fn categorise_labels(labels: Labels) -> ShortLabels {
	let mut source_labels = Vec::new();
	let mut other_labels = Vec::new();

	for label in labels {
		if let Ok(source_label) = SourceLabel::try_from(label.clone()) {
			source_labels.push(source_label);
		} else {
			other_labels.push(StatusLabel::from(label));
		}
	}

	let mut shorts = ShortLabels {
		source: vec![],
		status: vec![],
	};

	for srclbl in source_labels {
		shorts.source.push((srclbl.to_string(), srclbl));
	}

	for statlbl in other_labels {
		shorts.status.push((statlbl.to_string(), statlbl));
	}

	shorts
}

#[cfg(test)]
mod tests_repo_to_filename {
	use super::*;

	#[test]
	fn slash() {
		assert_eq!(
			repo_to_filename("w3c/a11y-review"),
			String::from("w3c-a11y-review")
		);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn group_label_cg_open_ui() {
		let label = Label {
			name: String::from("cg:open-ui"),
			description: String::from("A nice CG"),
		};
		let source_label = SourceLabel::try_from(label).unwrap();
		assert_eq!(
			SourceLabel::GroupLabel {
				prefix: Some(String::from("cg")),
				name: String::from("open-ui"),
				description: String::from("A nice CG")
			},
			source_label
		);
	}

	#[test]
	fn group_label_ietf() {
		let label = Label {
			name: String::from("ietf"),
			description: String::new(),
		};
		let source_label = SourceLabel::try_from(label).unwrap();
		assert_eq!(
			SourceLabel::GroupLabel {
				prefix: None,
				name: String::from("ietf"),
				description: String::new()
			},
			source_label
		);
	}

	#[test]
	fn spec_label_without_space() {
		let label = Label {
			name: String::from("s:css"),
			description: String::new(),
		};
		let source_label = SourceLabel::try_from(label).unwrap();
		assert_eq!(
			SourceLabel::SpecLabel {
				prefix: String::from("s"),
				name: String::from("css"),
				description: String::new()
			},
			source_label
		);
	}

	#[test]
	fn spec_label_with_space() {
		let label = Label {
			name: String::from("Topic: accessibility"),
			description: String::new(),
		};
		let source_label = SourceLabel::try_from(label).unwrap();
		assert_eq!(
			SourceLabel::SpecLabel {
				prefix: String::from("Topic"),
				name: String::from("accessibility"),
				description: String::new()
			},
			source_label
		);
	}

	#[test]
	fn status_label_simple() {
		let label = Label {
			name: String::from("advice-requested"),
			description: String::new(),
		};
		let status_label = StatusLabel::from(label);
		assert_eq!(
			StatusLabel {
				prefix: None,
				name: String::from("advice-requested"),
				description: String::new()
			},
			status_label
		);
	}

	#[test]
	fn status_label_with_prefix() {
		let label = Label {
			name: String::from("Progress: Untriaged"),
			description: String::new(),
		};
		let status_label = StatusLabel::from(label);
		assert_eq!(
			StatusLabel {
				prefix: Some(String::from("Progress")),
				name: String::from("Untriaged"),
				description: String::new()
			},
			status_label
		);
	}
}
