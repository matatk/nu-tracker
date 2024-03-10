mod label_string_vec;
pub use label_string_vec::LabelStringVec;
mod make_status_structs;
use make_status_structs::make_status_structs;

/// Functions for linking single-char flags to known status labels
pub trait StatusLabelInfo {
	/// Given a single-character flag, what is the expanded issue label?
	fn label_for(flag: &char) -> Option<&'static str>;
	/// Produce a string that can be printed to enumerate the flags and labels
	fn flags_labels_conflicts() -> String;
}

make_status_structs! {
	Comment:
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

/// An issue label container that knows about comment review request issues
pub type CommentLabels = LabelStringVec<CommentFromStrHelper>;

make_status_structs! {
	Design:
	(progress_untriaged, "Progress: untriaged", 'U'),
	(progress_in_progress, "Progress: in progress", 'i'),
	(progress_pending_external_feedback, "Progress: pending external feedback", 'x'),
}

/// An issue label container that knows about design review request issues
pub type DesignLabels = LabelStringVec<CharterFromStrHelper>;

make_status_structs! {
	Charter:
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

/// An issue label container that knows about charter review request issues
pub type CharterLabels = LabelStringVec<CharterFromStrHelper>;
