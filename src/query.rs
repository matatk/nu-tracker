use std::{
	error::Error,
	fmt,
	io::{self, Write},
	process::Command,
	str,
};

use paste::paste;
use serde::Deserialize;

use crate::assignee_query::AssigneeQuery;
use crate::showing::showing;

#[derive(Debug)]
pub enum QueryError {
	GhDidNotRunSuccessfully,
}

impl Error for QueryError {}

impl fmt::Display for QueryError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self {
			QueryError::GhDidNotRunSuccessfully => write!(f, "'gh' did not run successfully")?,
		}
		Ok(())
	}
}

pub struct Query {
	pretty: String,
	cmd: Command,
	verbose: bool,
	include_closed: bool,
	repos: Vec<String>,
	labels: Vec<String>,
	not_labels: Vec<String>,
}

macro_rules! make_setters {
	($thing:ident) => {
		paste! {
			pub fn $thing(&mut self, thing: impl Into<String>) -> &mut Self {
				self.[<$thing s>].push(thing.into());
				self
			}

			pub fn [<$thing s>]<I, S>(&mut self, things: I) -> &mut Self
			where
				I: IntoIterator<Item = S>,
				S: Into<String>
			{
				for thing in things {
					self.$thing(thing);
				}
				self
			}
		}
	};
}

impl Query {
	pub fn new(pretty: impl Into<String>, verbose: bool) -> Self {
		let mut cmd = Command::new("gh");
		cmd.args(["search", "issues"]);
		Self {
			pretty: pretty.into(),
			cmd,
			verbose,
			include_closed: false,
			repos: vec![],
			labels: vec![],
			not_labels: vec![],
		}
	}

	make_setters!(repo);
	make_setters!(label);
	make_setters!(not_label);

	pub fn include_closed(&mut self, include_closed: bool) -> &mut Self {
		self.include_closed = include_closed;
		self
	}

	pub fn assignee(&mut self, aq: AssigneeQuery) -> &mut Self {
		if let AssigneeQuery::User(user) = aq {
			self.cmd.args(vec!["--assignee", &user]);
		} else if let AssigneeQuery::Nobody = aq {
			self.cmd.args(vec!["--no-assignee"]);
		}
		self
	}

	pub fn run_direct(&mut self, web: bool) {
		if web {
			self.cmd.arg("--web");
		}
		self.set_up_args(None);
		if self.verbose {
			println!("{}: running: {:?}", self.pretty, self.cmd);
		}
		self.cmd.status().expect("'gh' should run");
	}

	// FIXME: remove the need to pass in field names - https://stackoverflow.com/a/70123652/1485308
	pub fn run<T>(&mut self, description: &str, fields: Vec<&str>) -> Result<Vec<T>, QueryError>
	where
		T: for<'a> Deserialize<'a>,
	{
		self.set_up_args(Some(fields.join(",")));

		if self.verbose {
			println!("{}: running: {:?}", self.pretty, self.cmd);
		}
		let output = self.cmd.output().expect("'gh' should run");

		if output.status.success() {
			let out = str::from_utf8(&output.stdout).expect("got non-utf8 data from 'gh'");
			let found: Vec<T> = serde_json::from_str(out).unwrap();

			if found.is_empty() {
				println!("No {} found", description);
				return Ok(vec![]);
			} else {
				println!("{} {}\n", showing(found.len()), description)
			}

			Ok(found)
		} else {
			io::stdout().write_all(&output.stdout).unwrap();
			io::stderr().write_all(&output.stderr).unwrap();
			Err(QueryError::GhDidNotRunSuccessfully)
		}
	}

	fn set_up_args(&mut self, field_names_arg: Option<String>) {
		for repo in &self.repos {
			self.cmd.args(["--repo", repo]);
		}

		for label in &self.labels {
			self.cmd.args(["--label", label]);
		}

		if !self.include_closed {
			self.cmd.args(["--state", "open"]);
		}

		if let Some(fields) = field_names_arg {
			self.cmd.args(["--json", &fields]);
		}

		if !self.not_labels.is_empty() {
			self.cmd.arg("--");

			for label in &self.not_labels {
				self.cmd.arg(format!("-label:{}", label));
			}
		}
	}
}
