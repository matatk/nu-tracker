// TODO: DRY issue with and without body?
use serde::{Deserialize, Serialize};
use struct_field_names_as_array::FieldNamesAsArray;

#[derive(Clone, Serialize, Deserialize, FieldNamesAsArray)]
pub struct ReturnedIssueHeavy {
	pub assignees: Vec<Assignee>,
	pub body: String,
	pub labels: Vec<Label>,
	pub number: u32,
	pub repository: Repository,
	pub title: String,
}

#[derive(Clone, Serialize, Deserialize, FieldNamesAsArray)]
pub struct ReturnedIssue {
	pub assignees: Vec<Assignee>,
	pub body: String,
	pub number: u32,
	pub repository: Repository,
	pub title: String,
}

#[derive(Serialize, Deserialize, FieldNamesAsArray)]
pub struct ReturnedIssueLight {
	pub assignees: Vec<Assignee>,
	pub number: u32,
	pub title: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Assignee {
	pub id: String,
	pub is_bot: bool,
	pub login: String,
	pub r#type: String,
	pub url: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Label {
	pub id: String,
	pub color: String,
	pub description: String,
	pub name: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
	pub name: String,
	pub name_with_owner: String,
}
