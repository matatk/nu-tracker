use crate::returned_issue::Assignee;

pub fn flatten_assignees(assignees: &[Assignee]) -> String {
	let logins = assignees
		.iter()
		.map(|a| a.login.to_string())
		.collect::<Vec<_>>()
		.join(",");

	if logins.is_empty() {
		String::from("UNASSIGNED")
	} else {
		logins
	}
}
