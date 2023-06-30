use std::process::Command;

pub enum AssigneeQuery {
	User(String),
	Nobody,
	NotImportantRightNow,
}

impl AssigneeQuery {
	pub fn new(username: Option<String>, unassigend: bool) -> Self {
		if let Some(user) = username {
			return AssigneeQuery::User(user);
		} else if unassigend {
			return AssigneeQuery::Nobody;
		}
		AssigneeQuery::NotImportantRightNow
	}

	pub fn gh_args(&self, gh: &mut Command) {
		match self {
			AssigneeQuery::User(user) => gh.args(vec!["--assignee", user]),
			AssigneeQuery::Nobody => gh.args(vec!["--no-assignee"]),
			_ => gh,
		};
	}
}
