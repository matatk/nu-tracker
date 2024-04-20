/// How results should be searched regarding assignment
pub enum AssigneeQuery {
	/// Search for issues assigned to a particular user
	User(String),
	/// Search for issues assigned to nobody
	Nobody,
	/// Meh
	NotImportantRightNow,
}

impl AssigneeQuery {
	/// Create a new assignee query
	pub fn new(username: Option<String>, unassigend: bool) -> Self {
		if let Some(user) = username {
			return AssigneeQuery::User(user);
		} else if unassigend {
			return AssigneeQuery::Nobody;
		}
		AssigneeQuery::NotImportantRightNow
	}
}
