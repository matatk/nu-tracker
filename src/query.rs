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
	NoResultsFound(String), // TODO: &str?
}

impl Error for QueryError {}

impl fmt::Display for QueryError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self {
			QueryError::GhDidNotRunSuccessfully => write!(f, "'gh' did not run successfully")?,
			QueryError::NoResultsFound(description) => write!(f, "No {description} found")?,
		}
		Ok(())
	}
}

pub struct Query<'c> {
	task_name: String,
	cmd_args: Vec<&'c str>,
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

impl<'c> Query<'c> {
	pub fn new(task_name: impl Into<String>, verbose: bool) -> Self {
		Self {
			task_name: task_name.into(),
			cmd_args: vec![],
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

	pub fn assignee(&mut self, aq: &'c AssigneeQuery) -> &mut Self {
		if let AssigneeQuery::User(user) = aq {
			self.cmd_args.push("--assignee");
			self.cmd_args.push(user);
		} else if let AssigneeQuery::Nobody = aq {
			self.cmd_args.push("--no-assignee");
		}
		self
	}

	pub fn run_gh(&mut self, web: bool) {
		let mut cmd = self.set_up_args(None);
		if web {
			cmd.arg("--web");
		}
		// TODO: DRY
		if self.verbose {
			println!("{}: running: {:?}", self.task_name, cmd);
		}
		cmd.status().expect("'gh' should run");
	}

	// FIXME: remove the need to pass in field names - https://stackoverflow.com/a/70123652/1485308
	pub fn run<T>(&mut self, description: &str, fields: Vec<&str>) -> Result<Vec<T>, QueryError>
	where
		T: for<'a> Deserialize<'a>,
	{
		let mut cmd = self.set_up_args(Some(fields.join(",")));

		// TODO: DRY
		if self.verbose {
			println!("{}: running: {:?}", self.task_name, cmd);
		}
		let output = cmd.output().expect("'gh' should run");

		if output.status.success() {
			let out = str::from_utf8(&output.stdout).expect("got non-utf8 data from 'gh'");
			let found: Vec<T> = serde_json::from_str(out).unwrap();

			if found.is_empty() {
				return Err(QueryError::NoResultsFound(description.into()));
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

	fn set_up_args(&mut self, field_names_arg: Option<String>) -> Command {
		let mut cmd = Command::new("gh");
		cmd.args(["search", "issues"]);

		for repo in &self.repos {
			cmd.args(["--repo", repo]);
		}

		for label in &self.labels {
			cmd.args(["--label", label]);
		}

		if !self.include_closed {
			cmd.args(["--state", "open"]);
		}

		if let Some(fields) = field_names_arg {
			cmd.args(["--json", &fields]);
		}

		if !self.not_labels.is_empty() {
			cmd.arg("--");

			for label in &self.not_labels {
				cmd.arg(format!("-label:{}", label));
			}
		}

		cmd
	}
}
