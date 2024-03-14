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
