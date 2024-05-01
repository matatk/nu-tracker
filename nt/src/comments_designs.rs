mod comments;
mod designs;

use std::fmt;

use regex::Regex;

pub use comments::{comments, CommentField};
pub use designs::{designs, DesignField};

const TRY_TITLE_COLUMN_WIDTH: u16 = 50;

/// Wrapper around `Vec<AsRef<str>>` that implements [Display](std::fmt::Display)
///
/// This allows the definition of the CLI to be kept simpler, making it easy to use Clap's helpers like [clap::ValueEnum].
pub struct DisplayableVec<T>(Vec<T>);

impl<T> From<Vec<T>> for DisplayableVec<T> {
	fn from(value: Vec<T>) -> Self {
		Self(value)
	}
}

impl<T: AsRef<str>> fmt::Display for DisplayableVec<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}",
			self.0
				.iter()
				.map(|f| f.as_ref())
				.collect::<Vec<_>>()
				.join(", ")
		)
	}
}

// TODO: make it optional at print time whether we include the prefix?
//       s: - never show
//       Venue: - never show
//       Topic: - never show
//       cg/wg/ig/bg: - DO show
macro_rules! make_source_label {
	($name:ident: prefix: $($prefix:expr)+ $(; prefixs: $($prefixs:expr)+)? $(; whole: $whole:expr)?) => {
		::paste::paste! {
			#[derive(Debug, PartialEq, Eq, Hash, Clone)]
			struct [<$name Label>] {
				prefix: Option<String>,
				name: String,
				colour: ::crossterm::style::Color
			}

			#[derive(Debug, PartialEq)]
			pub struct [<$name LabelError>];

			impl ::std::convert::TryFrom<&crate::returned_issue::Label> for [<$name Label>] {
				type Error = [<$name LabelError>];

				// TODO: this is all very cloney
				fn try_from(label: &crate::returned_issue::Label) -> Result<Self, Self::Error> {
					$(
						if label.name == $whole {
							return Ok(Self {
								prefix: None,
								name: label.name.clone().into(),
								colour: label.color.clone().into()
							})
						}
					)?

					match label.name.split_once(':') {
						Some((prefix, name)) => {
							$(
								if prefix == $prefix {
									return Ok(Self {
										prefix: Some(prefix.into()),
										name: name.into(),
										colour: label.color.clone().into()
									})
								}
							)+
							$(
								$(
									if prefix == $prefixs {
										return Ok(Self {
											prefix: Some(prefix.into()),
											name: name.trim().into(),
											colour: label.color.clone().into()
										})
									}
								)+
							)?
							Err([<$name LabelError>])
						}
						None => Err([<$name LabelError>]),
					}
				}
			}

			impl ::std::fmt::Display for [<$name Label>] {
				fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
					use ::crossterm::style::Stylize;

					// TODO: cloney. also efficiency?
					let prefix = self.prefix.clone().map_or(String::from(""), |p| format!("{p}:"));
					let whole = format!("{}{}", prefix, self.name);
					write!(f, "{}", whole.with(self.colour))
				}
			}
		}
	};
}

pub(crate) use make_source_label;

macro_rules! make_print_table {
	($prefix:ident) => {
		::paste::paste! {
			use crate::status_labels::{Status, Conflicts};

			fn print_table(
				spec: Option<String>,
				fields: &[[<$prefix Field>]],
				requests: &[[<$prefix ReviewRequest>]],
			) {
				// TODO: more functional?
				let mut rows = vec![];
				let mut invalid_reqs = vec![];
				let mut group_labels: HashSet<GroupLabel> = HashSet::new();
				let mut spec_labels: HashSet<SpecLabel> = HashSet::new();

				for request in requests {
					if spec.is_none() {
						if let Some(label) = &request.spec {
							spec_labels.insert(label.clone());
						}
					}

					if let Some(label) = &request.group {
						group_labels.insert(label.clone());
					}

					// FIXME: shouldn't need to clone
					rows.push(request.to_vec_string(&fields));

					if !request.status.is_valid() {
						invalid_reqs.push(vec![
							request.id.to_string(),
							request.title.clone(),
							format!("{}", request.status),
						])
					}
				}

				if !invalid_reqs.is_empty() {
					println!(
						"Requests with invalid statuses due to conflicting labels:\n\n{}\n",
						crate::generate_table::generate_table(vec!["ID", "TITLE", "INVALID STATUS"], invalid_reqs, None, None)
					);
				}

				fn list_domains<T: fmt::Display>(pretty: &str, labels: HashSet<T>) {
					if !labels.is_empty() {
						let mut domains = labels.iter().map(|s| format!("{s}")).collect::<Vec<_>>();
						domains.sort();
						println!("{pretty}: {}\n", domains.join(", "));
					}
				}

				list_domains("Groups", group_labels);
				list_domains("Specs", spec_labels);

				let mut max_widths = HashMap::new();
				let mut title_column_index = None;

				for (i, field) in fields.iter().enumerate() {
					if let Some(max_width) = [<$prefix ReviewRequest>]::max_field_width(field) {
						max_widths.insert(i, max_width);
					}
					if field == &[<$prefix Field>]::Title {
						title_column_index = Some(i)
					}
				}

				let try_first = if let Some(index) = title_column_index {
					Some((index, TRY_TITLE_COLUMN_WIDTH))
				} else {
					None
				};

				let table = crate::generate_table::generate_table(
					fields.iter().map(|h| h.as_ref().to_uppercase()).collect(),
					rows,
					try_first,
					Some(max_widths),
				);
				println!("{table}")
			}
		}
	};
}

pub(crate) use make_print_table;

// TODO: change to return result, because not having the link is an error?
fn get_source_issue_locator(body: &str) -> String {
	let re = Regex::new(r"§ https://github.com/(.+)/(.+)/.+/(\d+)").unwrap();

	if let Some(caps) = re.captures(body) {
		let owner = caps.get(1).unwrap().as_str();
		let repo = caps.get(2).unwrap().as_str();
		let number = caps.get(3).unwrap().as_str();
		return format!("{}/{}#{}", owner, repo, number);
	}

	String::from("UNKNOWN!")
}

#[cfg(test)]
mod tests_get_locator {
	use super::*;

	#[test]
	fn no_crash_if_no_dates() {
		assert_eq!(
			get_source_issue_locator("Invalid request"),
			String::from("UNKNOWN!")
		);
	}

	#[test]
	fn multiple_lines() {
		assert_eq!(
			get_source_issue_locator(
				"**This is a tracker issue.** Only discuss things here if they are a11y group internal meta-discussions about the issue. **Contribute to the actual discussion at the following link:**

§ https://github.com/openui/open-ui/issues/530"
			),
			String::from("openui/open-ui#530")
		);
	}

	#[test]
	fn multiple_lines_pr() {
		assert_eq!(
			get_source_issue_locator(
				"**This is a tracker issue.** Only discuss things here if they are a11y group internal meta-discussions about the issue. **Contribute to the actual discussion at the following link:**

§ https://github.com/whatwg/html/pull/8352"
			),
			String::from("whatwg/html#8352")
		);
	}
}

#[cfg(test)]
mod tests_spec_label {
	use std::assert_eq;

	use crossterm::style::Color;

	use crate::returned_issue::Label;

	// FIXME: test for status labels being invalid

	#[test]
	fn valid_source() {
		make_source_label!(Spec: prefix: "s");
		let label = Label {
			description: "".into(),
			id: "".into(),
			name: "s:html".into(),
			color: (0, 42, 0),
		};
		let result = SpecLabel::try_from(&label).unwrap();
		assert_eq!(
			result,
			SpecLabel {
				prefix: Some(String::from("s")),
				name: String::from("html"),
				colour: Color::Rgb { r: 0, g: 42, b: 0 }
			}
		)
	}

	#[test]
	fn valid_source_group_without_space() {
		make_source_label!(Group: prefix: "wg" "cg" "ig" "bg"; prefixs: "Venue");
		let label = Label {
			description: "".into(),
			id: "".into(),
			name: "wg:apa".into(),
			color: (42, 0, 0),
		};
		let result = GroupLabel::try_from(&label).unwrap();
		assert_eq!(
			result,
			GroupLabel {
				prefix: Some(String::from("wg")),
				name: String::from("apa"),
				colour: Color::Rgb { r: 42, g: 0, b: 0 }
			}
		)
	}

	#[test]
	fn valid_source_group_with_space() {
		make_source_label!(Group: prefix: "wg" "cg" "ig" "bg"; prefixs: "Venue");
		let label = Label {
			description: "".into(),
			id: "".into(),
			name: "Venue: OpenUI".into(),
			color: (0, 0, 42),
		};
		let result = GroupLabel::try_from(&label).unwrap();
		assert_eq!(
			result,
			GroupLabel {
				prefix: Some(String::from("Venue")),
				name: String::from("OpenUI"),
				colour: Color::Rgb { r: 0, g: 0, b: 42 }
			}
		)
	}

	#[test]
	fn invalid_source() {
		make_source_label!(Spec: prefix: "s");
		let label = Label {
			description: "".into(),
			id: "".into(),
			name: "noop:html".into(),
			color: (42, 42, 0),
		};
		let result = SpecLabel::try_from(&label);
		assert_eq!(result, Err(SpecLabelError))
	}
}
