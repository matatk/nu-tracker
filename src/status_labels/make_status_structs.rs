macro_rules! make_status_structs {
	($name:ident: $(($variant:ident, $label:expr, $flag:expr$(, [$($incompatible:ident),+])?)),* $(,)?) => {
		::paste::paste! {
			$(
				const [<$variant:upper _FLAG>]: char = $flag;
				const [<$variant:upper _LABEL>]: &str = $label;
			)*

			#[derive(Clone)]
			#[doc = concat!("Links single-char flags to known status labels for ", stringify!($name), " issues")]
			pub struct [<$name FromStrHelper>] {}

			impl crate::status_labels::StatusLabelInfo for [<$name FromStrHelper>] {
				fn label_for(flag: &char) -> Option<&'static str> {
					match *flag {
						$(
							[<$variant:upper _FLAG>] => Some([<$variant:upper _LABEL>]),
						)*
						_ => None
					}
				}

				fn flags_labels_conflicts() -> String {
					let mut output = String::new();

					$(
						output += format!("{}: {}", $flag, $label).as_str();
						$(
							output += format!(" (conflicts with:").as_str();
							$(
								output += format!(" {}", [<$incompatible:upper _LABEL>]).as_str();
							)*
							output += ")";
						)?
						output += "\n";  // FIXME: need proc macro to get rid of this?
					)*

					output.trim().to_string()
				}
			}

			#[derive(Clone)]
			#[doc = concat!("Represents the status of ", stringify!($name), " issues")]
			pub struct [<$name Status>] {
				$(
					$variant: bool,
				)*
			}

			impl [<$name Status>] {
				#[doc = concat!("Make a new status representation for ", stringify!($name), " issues")]
				pub fn new() -> Self {
					Self {
						$(
							$variant: false,
						)*
					}
				}

				/// Specify that the related issue has the given status
				///
				/// This is called when converting an issue returned from the GitHub API
				/// into a particular type of request. If the passed-in label matches a
				/// known status variant, then that flag is set.
				pub fn is(&mut self, label: &str) {
					match label {
						$(
							$label => self.$variant = true,
						)*
						_ => ()
					}
				}

				/// Indicates if the issue's status is valid (i.e. no conflicting labels)
				pub fn is_valid(&self) -> bool {
					$(
						$(
							$(
								if self.$variant && self.$incompatible {
									return false
								}
							)*
						)*
					)?

					true
				}
			}

			impl ::std::default::Default for [<$name Status>] {
				fn default() -> Self {
					Self::new()
				}
			}

			// TODO: allow choice of long/short - means this approach isn't the right one?
			impl ::std::fmt::Display for [<$name Status>] {
				fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
					let mut chars: Vec<char> = vec![];
					$(
						if self.$variant {
							chars.push([<$variant:upper _FLAG>]);
						}
					)*

					if !chars.is_empty() {
						write!(f, "{}", chars
						   .iter()
						   .flat_map(|c| [c, &' '])
						   .take(chars.len() * 2 - 1)
						   .collect::<String>())?;
					}
					Ok(())
				}
			}
		}
	}
}

pub(crate) use make_status_structs;

#[cfg(test)]
mod tests {
	use std::assert_eq;

	use super::super::StatusLabelInfo;

	make_status_structs!(
		Test:
		(priority_1, "priority-1", '1', [priority_2]),
		(priority_2, "priority-2", '2', [priority_1]),
		(hotifx, "hotifx", 'h')
	);

	#[test]
	fn pretty_empty() {
		let status = TestStatus::new();
		assert_eq!(format!("{}", status), "");
	}

	#[test]
	fn pretty_one() {
		let mut status = TestStatus::new();
		status.is("priority-2");
		assert_eq!(format!("{}", status), "2");
	}

	#[test]
	fn pretty_two() {
		let mut status = TestStatus::new();
		status.is("priority-2");
		status.is("hotifx");
		assert_eq!(format!("{}", status), "2 h");
	}

	#[test]
	fn labels_for_flag_1() {
		assert_eq!(TestFromStrHelper::label_for(&'1'), Some("priority-1"));
	}

	#[test]
	fn labels_for_flag_2() {
		assert_eq!(TestFromStrHelper::label_for(&'2'), Some("priority-2"));
	}

	#[test]
	fn labels_for_flag_h() {
		assert_eq!(TestFromStrHelper::label_for(&'h'), Some("hotifx"));
	}

	#[test]
	fn labels_for_flag_invalid() {
		assert_eq!(TestFromStrHelper::label_for(&'q'), None);
	}

	#[test]
	fn pretty_all() {
		assert_eq!(
			TestFromStrHelper::flags_labels_conflicts(),
			"1: priority-1 (conflicts with: priority-2)
2: priority-2 (conflicts with: priority-1)
h: hotifx"
		);
	}

	#[test]
	fn empty_status_is_valid() {
		let status = TestStatus::new();
		assert!(status.is_valid());
	}

	#[test]
	fn valid_status_is_valid() {
		let mut status = TestStatus::new();
		status.is("priority-2");
		status.is("hotifx");
		assert!(status.is_valid());
	}

	#[test]
	fn invalid_status_is_invalid() {
		let mut status = TestStatus::new();
		status.is("priority-1");
		status.is("priority-2");
		assert!(!status.is_valid());
	}
}
