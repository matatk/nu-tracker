mod comments;
mod designs;

use regex::Regex;

pub use comments::{comments, CommentField, DisplayableCommentFieldVec};
pub use designs::{designs, DesignField, DisplayableDesignFieldVec};

// FIXME: support whatwg (on its own)
// FIXME: make it optional at print time whether we include the prefix? (not for s:* but for wg:*)
macro_rules! make_source_label {
	($name:ident: $($prefix:expr)+) => {
		paste::paste! {
			#[derive(Debug, PartialEq, Eq, Hash, Clone)]
			struct [<$name Label>](String);

			#[derive(Debug, PartialEq)]
			pub struct [<$name LabelError>];

			impl std::str::FromStr for [<$name Label>] {
				type Err = [<$name LabelError>];

				/// Create a SourceLabel from a text string
				fn from_str(label_str: &str) -> Result<[<$name Label>], [<$name LabelError>]> {
					match label_str.split_once(':') {
						Some((prefix, group)) => {
							$(
								if prefix == $prefix {
									return Ok([<$name Label>](group.into()))
								}
							)+
							Err([<$name LabelError>])
						}
						None => Err([<$name LabelError>]),
					}
				}
			}

			impl std::fmt::Display for [<$name Label>] {
				fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
		make_source_label!(Spec: "s");
		let result = SpecLabel::from_str("s:html").unwrap();
		assert_eq!(result, SpecLabel(String::from("html")))
	}

	#[test]
	fn valid_source_multiple() {
		make_source_label!(Group: "wg" "cg" "ig" "bg" "Venue");
		let result = GroupLabel::from_str("wg:apa").unwrap();
		assert_eq!(result, GroupLabel(String::from("apa")))
	}

	#[test]
	fn invalid_source() {
		make_source_label!(Spec: "s");
		let result = SpecLabel::from_str("noop:html");
		assert_eq!(result, Err(SpecLabelError))
	}
}
