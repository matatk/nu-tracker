use std::fmt::Display;

use strum::{EnumProperty, VariantArray};

pub trait StatusLabel: EnumProperty + VariantArray + Display {
	fn get_pretty(&self) -> &str {
		self.get_str("pretty").unwrap()
	}

	// TODO: print table including pretty flag
	// TODO: include that output pretty flag in help output
	fn legend() -> String {
		let mut outs = Vec::new();

		for variant in Self::VARIANTS {
			let flag = variant.get_str("flag").unwrap();
			let label = variant.to_string();
			outs.push(format!("{flag}:\t{label}"));
		}

		outs.join("\n")
	}
}

pub trait Status: Display {
	fn new() -> Self;
	fn is(&mut self, label: &str);
}

pub trait Conflicts: Status {
	fn is_valid(&self) -> bool;
}

macro_rules! make_label_and_status {
	($name:ident: [$(($variant:ident, $label:expr, $flag:expr$(, $pretty:expr)?)),* $(,)?]) => {
		::paste::paste! {
			#[doc = concat!("Labels that have meaning for ", stringify!($name), " review requests")]
			#[derive(Clone, ::strum::EnumString, ::strum::EnumProperty, ::strum::VariantArray, ::strum::Display)]
			pub enum [<$name Label>] {
				$(
					#[allow(missing_docs)]
					#[strum(serialize = $label, props(flag = $flag $(, pretty = $pretty)?))]
					[<$variant:camel>],
				)*
			}

			impl From<[<$name Label>]> for String {
				fn from(value: [<$name Label>]) -> Self {
					value.to_string()
				}
			}

			impl crate::status_labels::StatusLabel for [<$name Label>] {}

			impl ::clap::ValueEnum for [<$name Label>] {
				fn value_variants<'a>() -> &'a [Self] {
					<Self as ::strum::VariantArray>::VARIANTS
				}

				fn to_possible_value(&self) -> Option<::clap::builder::PossibleValue> {
					use ::strum::EnumProperty;
					Some(::clap::builder::PossibleValue::new(self.get_str("flag").unwrap()).help(self.to_string()))
				}
			}

			#[doc = concat!("Overall state of a ", stringify!($name), " request, derived from labels that have meaning for ", stringify!($name), " issues")]
			#[derive(Default)]
			pub struct [<$name Status>] {
				$(
					#[allow(missing_docs)]
					$variant: bool,
				)*
			}

			impl crate::status_labels::Status for [<$name Status>] {
				fn new() -> Self {
					Self::default()
				}

				fn is(&mut self, label: &str) {
					match label {
						$(
							$label => self.$variant = true,
						)*
						_ => ()
					}
				}
			}

			impl ::std::fmt::Display for [<$name Status>] {
				fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
					let mut outs = Vec::new();
					$(
						if self.$variant {
							outs.push(one_or_tother!($flag $(, $pretty)?))
						}
					)*
					write!(f, "{}", outs.join(" "))
				}
			}
		}
    };
}

macro_rules! one_or_tother {
	($flag:expr) => {
		$flag
	};
	($flag:expr, $pretty:expr) => {
		$pretty
	};
}

macro_rules! make_conflicts {
	($name:ident: $(($variant:ident, $conflicts_with:ident)),* $(,)?) => {
		::paste::paste! {
			impl crate::status_labels::Conflicts for [<$name Status>] {
				fn is_valid(&self) -> bool {
					$(
						if self.$variant && self.$conflicts_with {
							return false
						}
					)*
					true
				}
			}
		}
	}
}

make_label_and_status! {
	Comment: [
		(pending, "pending", "p", "P"),
		(close, "close?", "c", "C"),
		(tracker, "tracker", "t", "T"), // Prefixed, e.g. with "a11y-" in issue in source group"s repo.
		(needs_resolution, "needs-resolution", "n", "N"), // Prefixed, e.g. with "a11y-" in issue in source group"s repo.
		(recycle, "recycle", "r", "R"),
		(advice_requested, "advice-requested", "a", "A"), // Optional - source group is asking for advice
		(needs_attention, "needs-attention", "x", "X"),   // Optional - HR group realises this is an urgent issue
	]
}

make_conflicts! {
	Comment: (pending, needs_resolution), (tracker, needs_resolution)
}

make_label_and_status! {
	Design: [
		(progress_untriaged, "Progress: untriaged", "pu", "pU"),
		(progress_in_progress, "Progress: in progress", "pi", "pI"),
		(progress_pending_external_feedback, "Progress: pending external feedback", "px", "pX"),
	]
}

make_conflicts! { Design: }

make_label_and_status! {
	Charter: [
		(accessibility_completed, "Accessibility review completed", "a"),
		(accessibility_needs_resolution, "a11y-needs-resolution", "A"),
		(internationalization_completed, "Internationalization review completed", "i"),
		(internationalization_needs_resolution, "i18n-needs-resolution", "I"),
		(privacy_completed, "privacy review completed", "p"),
		(privacy_needs_resolution, "privacy-needs-resolution", "P"),
		(security_completed, "Security review completed", "s"),
		(security_needs_resolution, "security-needs-resolution", "S"),
		(tag_completed, "TAG review completed", "t"),
		(tag_needs_resolution, "tag-needs-resolution", "T"),
	]
}

make_conflicts! { Charter: }

#[cfg(test)]
mod tests {
	// FIXME: test pretty printing and flags (if can't fix help output, or maybe even if can)
	use std::assert_eq;

	use super::{Conflicts, Status};

	make_label_and_status!(
		Test: [
			(priority_1, "priority-1", "1"),
			(priority_2, "priority-2", "2"),
			(hotifx, "hotifx", "h")
		]
	);

	make_conflicts!(Test: (priority_1, priority_2));

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
	fn empty_status_is_valid() {
		let status = TestStatus::new();
		assert_eq!(status.is_valid(), true);
	}

	#[test]
	fn valid_status_is_valid() {
		let mut status = TestStatus::new();
		status.is("priority-2");
		status.is("hotifx");
		assert_eq!(status.is_valid(), true);
	}

	#[test]
	fn invalid_status_is_invalid() {
		let mut status = TestStatus::new();
		status.is("priority-1");
		status.is("priority-2");
		assert_eq!(status.is_valid(), false);
	}
}
