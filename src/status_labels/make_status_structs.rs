macro_rules! make_status_structs {
	($name:ident: $(($variant:ident, $label:expr, $flag:expr$(, [$($incompatible:ident),+])?)),* $(,)?) => {
		$(
			paste! {
				const [<$variant:upper _FLAG>]: char = $flag;
				const [<$variant:upper _LABEL>]: &str = $label;
			}
		)*

		paste! {
			#[derive(Clone)]
			pub struct [<$name Validator>] {}

			impl crate::status_labels::LabelInfo for [<$name Validator>] {
				fn label_for(flag: &char) -> Option<&'static str> {
					paste! {
						match *flag {
							$(
								[<$variant:upper _FLAG>] => Some([<$variant:upper _LABEL>]),
							)*
							_ => None
						}
					}
				}

				fn flags_labels_conflicts() -> String {
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
			}
		}

		#[derive(Clone)]
		pub struct $name {
			$(
				$variant: bool,
			)*
		}

		impl $name {
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

		impl Default for $name {
			fn default() -> Self {
				Self::new()
			}
		}

		// TODO: allow choice of long/short - means this approach isn't the right one?
		impl fmt::Display for $name {
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

pub(crate) use make_status_structs;

#[cfg(test)]
mod tests {
	use std::{assert_eq, fmt};

	use paste::paste;

	use crate::status_labels::LabelInfo;

	make_status_structs!(
		Status:
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
		assert_eq!(StatusValidator::label_for(&'1'), Some("priority-1"));
	}

	#[test]
	fn labels_for_flag_2() {
		assert_eq!(StatusValidator::label_for(&'2'), Some("priority-2"));
	}

	#[test]
	fn labels_for_flag_h() {
		assert_eq!(StatusValidator::label_for(&'h'), Some("hotifx"));
	}

	#[test]
	fn labels_for_flag_invalid() {
		assert_eq!(StatusValidator::label_for(&'q'), None);
	}

	#[test]
	fn pretty_all() {
		assert_eq!(
			StatusValidator::flags_labels_conflicts(),
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
