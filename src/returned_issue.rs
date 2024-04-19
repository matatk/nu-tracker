// FIXME: Only request fields that are needed given the input type - needs a proc macro?
use serde::{Deserialize, Serialize};

pub trait RequiredFieldNames {
	fn required_field_names() -> Vec<String>;
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ReturnedIssue {
	required_field_names: Vec<String>,
	pub assignees: Vec<Assignee>,
	pub number: u32,
	pub title: String,
	pub body: String,
	pub repository: Repository,
	pub labels: Vec<Label>,
	pub author: Assignee,
}

impl RequiredFieldNames for ReturnedIssue {
	fn required_field_names() -> Vec<String> {
		vec![
			"assignees".into(),
			"number".into(),
			"title".into(),
			"body".into(),
			"repository".into(),
			"labels".into(),
			"author".into(),
		]
	}
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Assignee {
	pub id: String,
	pub is_bot: bool,
	pub login: String,
	pub r#type: String,
	pub url: String,
}

impl ToString for Assignee {
	fn to_string(&self) -> String {
		self.login.clone()
	}
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Label {
	pub id: String,
	pub color: String,
	pub description: String,
	pub name: String,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
	pub name: String,
	pub name_with_owner: String,
}
