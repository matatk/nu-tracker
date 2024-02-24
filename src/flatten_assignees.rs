use crate::returned_issue::Assignee;

// TODO: Make a wrapped type and implement this on it
pub fn flatten_assignees(assignees: &[Assignee]) -> String {
	let logins = assignees
		.iter()
		.map(|a| a.to_string())
		.collect::<Vec<_>>()
		.join(",");

	if logins.is_empty() {
		String::from("UNASSIGNED")
	} else {
		logins
	}
}
