use std::{
	collections::{HashMap, HashSet},
	error::Error,
	fmt, println,
	str::FromStr,
};

use clap::ValueEnum;
use paste::paste;
use regex::Regex;
use serde::{Deserialize, Serialize};
use strum_macros::AsRefStr;

use crate::make_table::make_table;
use crate::query::Query;
use crate::returned_issue::ReturnedIssueANTBRLA;
use crate::status_labels::{CommentLabels, CommentStatus, LabelStringContainer};
use crate::{assignee_query::AssigneeQuery, fetch_sort_print_handler, ReportFormat};
use crate::{flatten_assignees::flatten_assignees, DesignLabels};

macro_rules! make_origin_label {
	($name:ident, [$($prefix:expr),+]$(, $whole:expr)?) => {
		paste! {
			#[derive(Debug, PartialEq, Eq, Hash, Clone)]
			struct [<$name Label>] {
				group_type: Option<String>,
				group_name: String,
				show_type: bool
			}

			#[derive(Debug, PartialEq)]
			pub struct [<$name LabelError>];

			impl FromStr for [<$name Label>] {
				type Err = [<$name LabelError>];

				fn from_str(label_str: &str) -> Result<[<$name Label>], [<$name LabelError>]> {
					$(
						if label_str == $whole {
							return Ok([<$name Label>] {
								group_type: None,
								group_name: label_str.into(),
								show_type: false
							})
						}
					)?
					match label_str.split_once(':') {
						Some((prefix, group)) => {
							$(
								if prefix == $prefix {
									return Ok([<$name Label>] {
										group_type: Some(prefix.into()),
										group_name: group.trim().into(), // support TAG 'Venue: ' labels
										show_type: false
									})
								}
							)+
							Err([<$name LabelError>])
						}
						None => Err([<$name LabelError>])
					}
				}
			}

			impl fmt::Display for [<$name Label>] {
				fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
					if !self.show_type || self.group_type.is_none() {
						write!(f, "{}", self.group_name)
					} else {
						write!(f, "{}:{}", self.group_type.as_ref().unwrap(), self.group_name)
					}
				}
			}
		}
	};
}

make_origin_label!(Spec, ["s"]);
make_origin_label!(Group, ["wg", "cg", "ig", "bg", "Venue"], "whatwg"); // NOTE: "Venue" is TAG-only

// TODO: DRY
/// Comment review request fields
#[derive(Serialize, Deserialize, AsRefStr, Hash, Eq, PartialEq, Clone, ValueEnum, Debug)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CommentField {
	/// Assigned users
	Assignees,
	/// The group the request is from/relates to
	Group,
	/// The tracking issue's number
	Id,
	/// Whether the issue comes from our group
	Our,
	/// The source issue
	Source,
	/// The spec the request relates to
	Spec,
	/// The status of the request
	Status,
	/// The request's title
	Title,
}

// TODO: DRY
/// Comment review request fields
#[derive(Serialize, Deserialize, AsRefStr, Hash, Eq, PartialEq, Clone, ValueEnum, Debug)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DesignField {
	/// Assigned users
	Assignees,
	/// The group the request is from/relates to
	Group,
	/// The tracking issue's number
	Id,
	// NOTE: Our is missing from here
	/// The source issue
	Source,
	/// The spec the request relates to
	Spec,
	/// The status of the request
	Status,
	/// The request's title
	Title,
}

// FIXME: link to the std, and Clap, traits
/// Wrapper around `Vec<CommentField>` that implements Display
///
/// This is here to allow the definition of the CLI to be kept simpler, making it easy to use Clap's helpers like ValueEnum.
pub struct DisplayableCommentFieldVec(Vec<CommentField>);

impl From<Vec<CommentField>> for DisplayableCommentFieldVec {
	fn from(value: Vec<CommentField>) -> Self {
		Self(value)
	}
}

impl fmt::Display for DisplayableCommentFieldVec {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}",
			self.0
				.iter()
				.map(|f| f.as_ref())
				.collect::<Vec<_>>()
				.join(", ")
		)
	}
}

// TODO: Make things return &str instead of String
trait CommentOrDesignReviewRequest: From<ReturnedIssueANTBRLA> {
	fn group(&self) -> Option<GroupLabel>;
	fn id(&self) -> u32;
	fn max_field_width(field: &str) -> Option<u16>;
	fn source(&self) -> String;
	fn spec(&self) -> Option<SpecLabel>;
	fn status(&self) -> CommentStatus;
	fn title(&self) -> String;
	fn to_vec_string(&self, fields: Vec<&str>) -> Vec<String>;
}

struct CommentReviewRequest {
	group: Option<GroupLabel>,
	spec: Option<SpecLabel>,
	status: CommentStatus,
	source: String, // TODO: Make it a Locator? Doesn't seem needed.
	title: String,
	assignees: String,
	id: u32,
	our: bool,
}

// TODO: Only create requested fields.
// TODO: Only _request_ (from gh) requested fields.
impl From<ReturnedIssueANTBRLA> for CommentReviewRequest {
	fn from(issue: ReturnedIssueANTBRLA) -> CommentReviewRequest {
		let mut group = None;
		let mut spec = None;
		let mut status: CommentStatus = CommentStatus::new();

		for label in issue.labels {
			let name = label.name.to_string();

			// TODO: Check for having already inserted?
			if let Ok(gl) = GroupLabel::from_str(&name) {
				group = Some(gl)
			} else if let Ok(sl) = SpecLabel::from_str(&name) {
				spec = Some(sl)
			} else if group.is_none() && spec.is_none() {
				status.is(&name)
			}
		}

		CommentReviewRequest {
			group,
			spec,
			status,
			source: get_source_issue_locator(&issue.body),
			title: issue.title,
			assignees: flatten_assignees(&issue.assignees),
			id: issue.number,
			our: issue.author.to_string() != "w3cbot",
		}
	}
}

impl CommentOrDesignReviewRequest for CommentReviewRequest {
	fn max_field_width(field: &str) -> Option<u16> {
		match field {
			"assignees" => Some(15),
			"group" => Some(11),
			"spec" => Some(15),
			_ => None,
		}
	}

	fn to_vec_string(&self, fields: Vec<&str>) -> Vec<String> {
		let mut out: Vec<String> = vec![];

		for field in fields {
			out.push(match field {
				"group" => {
					if let Some(group) = &self.group {
						group.to_string()
					} else {
						String::from("???")
					}
				}
				"spec" => {
					if let Some(spec) = &self.spec {
						spec.to_string()
					} else {
						String::from("???")
					}
				}
				"status" => format!("{}", self.status),
				"source" => self.source.clone(),
				"title" => self.title.clone(),
				"assignees" => self.assignees.clone(),
				"id" => self.id.to_string(),
				"our" => {
					if self.our {
						String::from("Yes")
					} else {
						String::from(" - ")
					}
				}
				_ => {
					panic!("Invalid comment review request field name: '{field}'")
				}
			})
		}

		out
	}

	fn id(&self) -> u32 {
		self.id
	}

	fn spec(&self) -> Option<SpecLabel> {
		self.spec.clone() // TODO: remove somehow?
	}

	fn group(&self) -> Option<GroupLabel> {
		self.group.clone() // TODO: remove somehow?
	}

	fn title(&self) -> String {
		self.title.clone() // TODO: make &str
	}

	fn status(&self) -> CommentStatus {
		self.status.clone() // TODO: remove somehow?
	}

	fn source(&self) -> String {
		self.source.clone() // TODO: return &str
	}
}

struct DesignReviewRequest {
	group: Option<GroupLabel>,
	spec: Option<SpecLabel>,
	status: CommentStatus,
	source: String, // TODO: Make it a Locator? Doesn't seem needed.
	title: String,
	assignees: String,
	id: u32,
}

// TODO: Only create requested fields.
// TODO: Only _request_ (from gh) requested fields.
impl From<ReturnedIssueANTBRLA> for DesignReviewRequest {
	fn from(issue: ReturnedIssueANTBRLA) -> Self {
		let mut group = None;
		let mut spec = None;
		let mut status: CommentStatus = CommentStatus::new();

		for label in issue.labels {
			let name = label.name.to_string();

			// TODO: Check for having already inserted?
			if let Ok(gl) = GroupLabel::from_str(&name) {
				group = Some(gl)
			} else if let Ok(sl) = SpecLabel::from_str(&name) {
				spec = Some(sl)
			} else if group.is_none() && spec.is_none() {
				status.is(&name)
			}
		}

		Self {
			group,
			spec,
			status,
			source: get_source_issue_locator(&issue.body),
			title: issue.title,
			assignees: flatten_assignees(&issue.assignees),
			id: issue.number,
		}
	}
}

impl CommentOrDesignReviewRequest for DesignReviewRequest {
	fn max_field_width(field: &str) -> Option<u16> {
		return None;
		match field {
			"assignees" => Some(15),
			"group" => Some(11),
			"spec" => Some(15),
			_ => None,
		}
	}

	fn to_vec_string(&self, fields: Vec<&str>) -> Vec<String> {
		let mut out: Vec<String> = vec![];

		for field in fields {
			out.push(match field {
				"group" => {
					if let Some(group) = &self.group {
						group.to_string()
					} else {
						String::from("???")
					}
				}
				"spec" => {
					if let Some(spec) = &self.spec {
						spec.to_string()
					} else {
						String::from("???")
					}
				}
				"status" => format!("{}", self.status),
				"source" => self.source.clone(),
				"title" => self.title.clone(),
				"assignees" => self.assignees.clone(),
				"id" => self.id.to_string(),
				_ => {
					panic!("Invalid comment review request field name: '{field}'")
				}
			})
		}

		out
	}

	fn id(&self) -> u32 {
		self.id
	}

	fn spec(&self) -> Option<SpecLabel> {
		self.spec.clone() // TODO: remove somehow?
	}

	fn group(&self) -> Option<GroupLabel> {
		self.group.clone() // TODO: remove somehow?
	}

	fn title(&self) -> String {
		self.title.clone() // TODO: make &str
	}

	fn status(&self) -> CommentStatus {
		self.status.clone() // TODO: remove somehow?
	}

	fn source(&self) -> String {
		self.source.clone() // TODO: return &str
	}
}

/// Options for querying for comments, and for design reviews
pub struct CommentsDesignsOptions<'a, T: LabelStringContainer, F> {
	/// The comments/design reviews repo
	pub repo: &'a str,
	/// Desired issue status
	pub status: T,
	/// Discount issues with status
	pub not_status: T,
	/// Issues relating to a particular spec
	pub spec: Option<String>,
	/// How issues should be assigned
	pub assignee: AssigneeQuery,
	/// Should the original (not tracking) issue be shown? (Table output only)
	pub show_source_issue: bool,
	/// Output reporting formats
	pub report_formats: &'a [ReportFormat],
	/// Fields/columns to show (table output only)
	pub fields: &'a [F],
	/// Be verbose?
	pub verbose: bool,
}

/// Query for issue comment requests; output a custom report.
pub fn comments(
	options: CommentsDesignsOptions<CommentLabels, CommentField>,
) -> Result<(), Box<dyn Error>> {
	core::<CommentReviewRequest, CommentLabels, CommentField>("Comments", "comments", options)
}

/// Query for design review requests; output a custom report.
pub fn designs(
	options: CommentsDesignsOptions<DesignLabels, DesignField>,
) -> Result<(), Box<dyn Error>> {
	core::<DesignReviewRequest, DesignLabels, DesignField>("Designs", "design reviews", options)
}

fn core<R: CommentOrDesignReviewRequest, T: LabelStringContainer, F>(
	query_name: &str,
	items_name: &str,
	options: CommentsDesignsOptions<T, F>,
) -> Result<(), Box<dyn Error>>
where
	std::string::String: From<<T as IntoIterator>::Item>,
{
	let CommentsDesignsOptions {
		repo,
		status,
		not_status,
		spec,
		assignee,
		show_source_issue,
		report_formats,
		fields,
		verbose,
	} = options;

	let mut query = Query::new(query_name, verbose);

	if let Some(ref spec) = spec {
		query.label(format!("s:{}", spec));
	}

	query
		.labels(status)
		.not_labels(not_status)
		.repo(repo)
		.assignee(&assignee);

	let transmogrify = |issue: ReturnedIssueANTBRLA| Some(R::from(issue));

	fetch_sort_print_handler!(items_name, query, transmogrify, report_formats, [{
		ReportFormat::Table => Box::new(|requests| print_table(spec.clone(), fields, show_source_issue, requests)),
		ReportFormat::Agenda => todo!(),
		ReportFormat::Meeting => Box::new(|requests| print_meeting(repo, requests)),
	}]);
	Ok(())
}

fn print_table<R: CommentOrDesignReviewRequest, F>(
	spec: Option<String>,
	comment_fields: &[F],
	show_source_issue: bool,
	requests: &[R],
) {
	// TODO: more functional?
	let mut rows = vec![];
	let mut invalid_reqs = vec![];
	let mut group_labels: HashSet<GroupLabel> = HashSet::new();
	let mut spec_labels: HashSet<SpecLabel> = HashSet::new();

	let mut headers = comment_fields
		.iter()
		.map(|f| f.as_ref())
		.collect::<Vec<_>>();

	if show_source_issue && !comment_fields.contains(&CommentField::Source) {
		headers.push(CommentField::Source.as_ref());
	}

	for request in requests {
		if spec.is_none() {
			if let Some(label) = &request.spec() {
				spec_labels.insert(label.clone());
			}
		}

		if let Some(label) = &request.group() {
			group_labels.insert(label.clone());
		}

		// FIXME: shouldn't need to clone
		rows.push(request.to_vec_string(headers.clone()));

		if !request.status().is_valid() {
			invalid_reqs.push(vec![
				request.id().to_string(),
				request.title(),
				format!("{}", request.status()),
			])
		}
	}

	if !invalid_reqs.is_empty() {
		println!(
			"Requests with invalid statuses due to conflicting labels:\n\n{}\n",
			make_table(vec!["ID", "TITLE", "INVALID STATUS"], invalid_reqs, None)
		);
	}

	fn list_domains<T: fmt::Display>(pretty: &str, labels: HashSet<T>) {
		if !labels.is_empty() {
			let mut domains = labels.iter().map(|s| format!("{s}")).collect::<Vec<_>>();
			domains.sort();
			println!("{pretty}: {}\n", domains.join(", "));
		}
	}

	list_domains("Groups", group_labels);
	list_domains("Specs", spec_labels);

	let mut max_widths = HashMap::new();
	for (i, header) in headers.iter().enumerate() {
		if let Some(max_width) = CommentReviewRequest::max_field_width(header) {
			max_widths.insert(i, max_width);
		}
	}

	let table = make_table(
		headers.iter().map(|h| h.to_uppercase()).collect(),
		rows,
		Some(max_widths),
	);
	println!("{table}")
}

// FIXME: source issue isn't a link - can we ToString a Repository struct?
// TODO: include an option to print out the status too?
fn print_meeting<R: CommentOrDesignReviewRequest>(repo: &str, requests: &[R]) {
	println!("gb, off\n");
	for request in requests {
		println!(
			"subtopic: {}\nsource: {}\ntracking: https://github.com/{}/issues/{}\n",
			request.title(),
			request.source(),
			repo,
			request.id()
		)
	}
	println!("gb, on")
}

// TODO: change to return result, because not having the link is an error?
fn get_source_issue_locator(body: &str) -> String {
	let re = Regex::new(r"§ https://github.com/(.+)/(.+)/.+/(\d+)").unwrap();

	if let Some(caps) = re.captures(body) {
		let owner = caps.get(1).unwrap().as_str();
		let repo = caps.get(2).unwrap().as_str();
		let number = caps.get(3).unwrap().as_str();
		return format!("{}/{}#{}", owner, repo, number);
	}

	String::from("UNKNOWN!")
}

#[cfg(test)]
mod tests_get_locator {
	use super::*;

	#[test]
	fn no_crash_if_no_dates() {
		assert_eq!(
			get_source_issue_locator("Invalid request"),
			String::from("UNKNOWN!")
		);
	}

	#[test]
	fn multiple_lines() {
		assert_eq!(
			get_source_issue_locator(
				"**This is a tracker issue.** Only discuss things here if they are a11y group internal meta-discussions about the issue. **Contribute to the actual discussion at the following link:**

§ https://github.com/openui/open-ui/issues/530"
			),
			String::from("openui/open-ui#530")
		);
	}

	#[test]
	fn multiple_lines_pr() {
		assert_eq!(
			get_source_issue_locator(
				"**This is a tracker issue.** Only discuss things here if they are a11y group internal meta-discussions about the issue. **Contribute to the actual discussion at the following link:**

§ https://github.com/whatwg/html/pull/8352"
			),
			String::from("whatwg/html#8352")
		);
	}
}

#[cfg(test)]
mod tests_spec_label {
	use std::assert_eq;

	use super::*;

	// FIXME: test for status labels being invalid

	#[test]
	fn valid_source_spec() {
		let result = SpecLabel::from_str("s:html").unwrap();
		assert_eq!(
			result,
			SpecLabel {
				group_type: Some(String::from("s")),
				group_name: String::from("html"),
				show_type: false
			}
		)
	}

	#[test]
	fn valid_source_group() {
		let result = GroupLabel::from_str("wg:apa").unwrap();
		assert_eq!(
			result,
			GroupLabel {
				group_type: Some(String::from("wg")),
				group_name: String::from("apa"),
				show_type: false
			}
		)
	}

	#[test]
	fn invalid_source() {
		let result = SpecLabel::from_str("noop:html");
		assert_eq!(result, Err(SpecLabelError))
	}
}
