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
}
