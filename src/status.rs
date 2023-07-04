use std::fmt;

use paste::paste;

mod label_string_vec;
pub use label_string_vec::{LabelStringVec, ParseFlagError};

// TODO: Switch to lifetimes rather than Strings?
macro_rules! make_maps_status_conflicts {
	($(($variant:ident, $label:expr, $flag:expr$(, [$($incompatible:ident),+])?)),* $(,)?) => {
		$(
			paste! {
				const [<$variant:upper _FLAG>]: char = $flag;
				const [<$variant:upper _LABEL>]: &str = $label;
			}
		)*

		fn label_for_flag(flag: &char) -> Option<&'static str> {
			paste! {
				match *flag {
					$(
						[<$variant:upper _FLAG>] => Some([<$variant:upper _LABEL>]),
					)*
					_ => None
				}
			}
		}

		pub fn flags_labels_conflicts() -> String {
			let mut output = String::new();

			paste! {
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
			}

			output.trim().to_string()
		}

		#[derive(Clone)]
		pub struct Status {
			$(
				$variant: bool,
			)*
		}

		impl Status {
			pub fn new() -> Self {
				Self {
					$(
						$variant: false,
					)*
				}
			}

			pub fn is(&mut self, label: &str) {
				match label {
					$(
						$label => self.$variant = true,
					)*
					_ => ()
				}
			}

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

		impl Default for Status {
			fn default() -> Self {
				Self::new()
			}
		}

		// TODO: allow choice of long/short - means this approach isn't the right one?
		impl fmt::Display for Status {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				let mut chars: Vec<char> = vec![];
				$(
					if self.$variant {
						paste! {
							chars.push([<$variant:upper _FLAG>]);
						}
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

// TODO: Use a proc macro, so conflicts need be specified only once, and neater output is easy?
make_maps_status_conflicts!(
	(pending, "pending", 'P', [needs_resolution]),
	(close, "close?", 'C'),
	(tracker, "tracker", 'T', [needs_resolution]), // Prefixed, e.g. with "a11y-" in issue in source group's repo.
	(
		needs_resolution,
		"needs-resolution",
		'N',
		[pending, tracker]
	), // Prefixed, e.g. with "a11y-" in issue in source group's repo.
	(recycle, "recycle", 'R'),
	(advice_requested, "advice-requested", 'A'), // Optional - source group is asking for advice
	(needs_attention, "needs-attention", 'X'),   // Optional - HR group realises this is an urgent issue
);

#[cfg(test)]
mod tests {
	use std::assert_eq;

	use super::*;

	make_maps_status_conflicts!(
		(priority_1, "priority-1", '1', [priority_2]),
		(priority_2, "priority-2", '2', [priority_1]),
		(hotifx, "hotifx", 'h')
	);

	#[test]
	fn pretty_empty() {
		let status = Status::new();
		assert_eq!(format!("{}", status), "");
	}

	#[test]
	fn pretty_one() {
		let mut status = Status::new();
		status.is("priority-2");
		assert_eq!(format!("{}", status), "2");
	}

	#[test]
	fn pretty_two() {
		let mut status = Status::new();
		status.is("priority-2");
		status.is("hotifx");
		assert_eq!(format!("{}", status), "2 h");
	}

	#[test]
	fn labels_for_flag_1() {
		assert_eq!(label_for_flag(&'1'), Some("priority-1"));
	}

	#[test]
	fn labels_for_flag_2() {
		assert_eq!(label_for_flag(&'2'), Some("priority-2"));
	}

	#[test]
	fn labels_for_flag_h() {
		assert_eq!(label_for_flag(&'h'), Some("hotifx"));
	}

	#[test]
	fn labels_for_flag_invalid() {
		assert_eq!(label_for_flag(&'q'), None);
	}

	#[test]
	fn pretty_all() {
		assert_eq!(
			flags_labels_conflicts(),
			"1: priority-1 (conflicts with: priority-2)
2: priority-2 (conflicts with: priority-1)
h: hotifx"
		);
	}

	#[test]
	fn empty_status_is_valid() {
		let status = Status::new();
		assert!(status.is_valid());
	}

	#[test]
	fn valid_status_is_valid() {
		let mut status = Status::new();
		status.is("priority-2");
		status.is("hotifx");
		assert!(status.is_valid());
	}

	#[test]
	fn invalid_status_is_invalid() {
		let mut status = Status::new();
		status.is("priority-1");
		status.is("priority-2");
		assert!(!status.is_valid());
	}
}
