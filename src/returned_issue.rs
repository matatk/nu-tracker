// TODO: DRY issue with and without body?
use serde::{Deserialize, Serialize};
use struct_field_names_as_array::FieldNamesAsArray;

#[derive(Serialize, Deserialize, FieldNamesAsArray)]
pub struct ReturnedIssueANT {
	pub assignees: Vec<Assignee>,
	pub number: u32,
	pub title: String,
}

#[derive(Clone, Serialize, Deserialize, FieldNamesAsArray)]
pub struct ReturnedIssueANTBR {
	pub assignees: Vec<Assignee>,
	pub number: u32,
	pub title: String,
	pub body: String,
	pub repository: Repository,
}

#[derive(Clone, Serialize, Deserialize, FieldNamesAsArray)]
pub struct ReturnedIssueANTBRL {
	pub assignees: Vec<Assignee>,
	pub number: u32,
	pub title: String,
	pub body: String,
	pub repository: Repository,
	pub labels: Vec<Label>,
}

#[derive(Clone, Serialize, Deserialize, FieldNamesAsArray)]
pub struct ReturnedIssueANTBRLA {
	pub assignees: Vec<Assignee>,
	pub number: u32,
	pub title: String,
	pub body: String,
	pub repository: Repository,
	pub labels: Vec<Label>,
	pub author: Assignee,
}

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
	pub name: String,
	pub name_with_owner: String,
}
