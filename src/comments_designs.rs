mod comments;
mod designs;

use std::fmt;

use regex::Regex;

pub use comments::{comments, CommentField};
pub use designs::{designs, DesignField};

/// Wrapper around `Vec<AsRef<str>>` that implements [Display](std::fmt::Display)
///
/// This allows the definition of the CLI to be kept simpler, making it easy to use Clap's helpers like [ValueEnum].
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

// TODO: make it optional at print time whether we include the prefix? (not for s:* but for wg:*)
macro_rules! make_source_label {
	($name:ident: prefix: $($prefix:expr)+ $(; prefixs: $($prefixs:expr)+)? $(; whole: $whole:expr)?) => {
		::paste::paste! {
			#[derive(Debug, PartialEq, Eq, Hash, Clone)]
			struct [<$name Label>](String);

			#[derive(Debug, PartialEq)]
			pub struct [<$name LabelError>];

			impl ::std::str::FromStr for [<$name Label>] {
				type Err = [<$name LabelError>];

				/// Create a SourceLabel from a text string
				fn from_str(label_str: &str) -> Result<[<$name Label>], [<$name LabelError>]> {
					$(
						if label_str == $whole {
							return Ok([<$name Label>](label_str.into()));
						}
					)?
					match label_str.split_once(':') {
						Some((prefix, group)) => {
							$(
								if prefix == $prefix {
									return Ok([<$name Label>](group.into()))
								}
							)+
							$(
								$(
									if prefix == $prefixs {
										return Ok([<$name Label>](group.trim().into()))
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
					write!(f, "{}", self.0)
				}
			}
		}
	};
}

pub(crate) use make_source_label;

macro_rules! make_fields_and_request {
	($prefix:ident, $fields_doc:expr, [ $($field:ident $type:ty | $variant:ident $doc:expr $(, $size:expr)?; $to_string:expr),+ ], $from:expr) => {
		::paste::paste! {
			#[doc = $fields_doc]
			#[derive(
				::std::clone::Clone,
				::std::cmp::Eq,
				::std::cmp::PartialEq,
				::std::fmt::Debug,
				::std::hash::Hash,
				::clap::ValueEnum,
				::serde::Deserialize,
				::serde::Serialize,
				::strum_macros::AsRefStr,
			)]
			#[strum(serialize_all = "lowercase")]
			#[serde(rename_all = "lowercase")]
			pub enum [<$prefix Field>] {
				$(
					#[doc = $doc]
					$variant,
				)+
			}

			struct [<$prefix ReviewRequest>] {
				$(
					$field: $type,
				)+
			}

			impl [<$prefix ReviewRequest>] {
				fn from(issue: ReturnedIssueANTBRLA) -> Self {
					$from(issue)
				}

				fn max_field_width(field: &[<$prefix Field>]) -> Option<u16> {
					match field {
						$(
							$(
								[<$prefix Field>]::$variant => Some($size),
							)?
						)+
						_ => None,
					}
				}
			}

			impl crate::ToVecStringWithFields for [<$prefix ReviewRequest>] {
				type Field = [<$prefix Field>];

				fn to_vec_string(&self, fields: &[[<$prefix Field>]]) -> Vec<String> {
					let mut out: Vec<String> = vec![];

					for field in fields {
						out.push(match field {
							$(
								[<$prefix Field>]::$variant => { $to_string(&self) },
							)+
						})
					}

					out
				}
			}
		}
	};
}

pub(crate) use make_fields_and_request;

macro_rules! make_print_table {
	($prefix:ident) => {
		::paste::paste! {
			fn print_table(
				spec: Option<String>,
				fields: &[[<$prefix Field>]],
				show_source_issue: bool,
				requests: &[[<$prefix ReviewRequest>]],
			) {
				// TODO: more functional?
				let mut rows = vec![];
				let mut invalid_reqs = vec![];
				let mut group_labels: HashSet<GroupLabel> = HashSet::new();
				let mut spec_labels: HashSet<SpecLabel> = HashSet::new();

				let mut headers = Vec::from(fields);

				if show_source_issue && !fields.contains(&[<$prefix Field>]::Source) {
					headers.push([<$prefix Field>]::Source);
				}

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
					rows.push(request.to_vec_string(&headers));

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
						make_table(vec!["ID", "TITLE", "INVALID STATUS"], invalid_reqs, None)
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
				for (i, header) in headers.iter().enumerate() {
					if let Some(max_width) = [<$prefix ReviewRequest>]::max_field_width(header) {
						max_widths.insert(i, max_width);
					}
				}

				let table = make_table(
					headers.iter().map(|h| h.as_ref().to_uppercase()).collect(),
					rows,
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
	use std::{assert_eq, str::FromStr};

	// FIXME: test for status labels being invalid

	#[test]
	fn valid_source() {
		make_source_label!(Spec: prefix: "s");
		let result = SpecLabel::from_str("s:html").unwrap();
		assert_eq!(result, SpecLabel(String::from("html")))
	}

	#[test]
	fn valid_source_multiple() {
		make_source_label!(Group: prefix: "wg" "cg" "ig" "bg"; prefixs: "Venue");

		let result = GroupLabel::from_str("wg:apa").unwrap();
		assert_eq!(result, GroupLabel(String::from("apa")));

		let result = GroupLabel::from_str("Venue: OpenUI").unwrap();
		assert_eq!(result, GroupLabel(String::from("OpenUI")))
	}

	#[test]
	fn invalid_source() {
		make_source_label!(Spec: prefix: "s");
		let result = SpecLabel::from_str("noop:html");
		assert_eq!(result, Err(SpecLabelError))
	}
}
