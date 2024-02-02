use std::fmt;

use paste::paste;

mod label_string_vec;
pub use label_string_vec::{LabelStringVec, ParseFlagError};
mod make_status_structs;
use make_status_structs::make_status_structs;

pub trait LabelInfo {
	fn label_for(flag: &char) -> Option<&'static str>;
	fn flags_labels_conflicts() -> String;
}

make_status_structs! {
	CommentStatus:
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
}

pub type CommentLabels = LabelStringVec<CommentStatusValidator>;

make_status_structs! {
	CharterStatus:
	(accessibility_completed, "Accessibility review completed", 'a'),
	(accessibility_needs_resolution, "a11y-needs-resolution", 'A'),
	(internationalization_completed, "Internationalization review completed", 'i'),
	(internationalization_needs_resolution, "i18n-needs-resolution", 'I'),
	(privacy_completed, "privacy review completed", 'p'),
	(privacy_needs_resolution, "privacy-needs-resolution", 'P'),
	(security_completed, "Security review completed", 's'),
	(security_needs_resolution, "security-needs-resolution", 'S'),
	(tag_completed, "TAG review completed", 't'),
	(tag_needs_resolution, "tag-needs-resolution", 'T'),
}

pub type CharterLabels = LabelStringVec<CharterStatusValidator>;
