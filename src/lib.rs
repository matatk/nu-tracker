//! Provides functions to query GitHub for issues, actions, and horizontal spec review and issue
//! comment requests, according to W3C conventions.
//!
//! The `gh` command is used to actually make the queries. The output from `gh` is eitehr printed
//! verbatim (in the case of issues), or obtained in JSON format, and processed extensively to add
//! more helpful information to it, to help WG and TF chairs keep track of things.
//!
//! For info on how to use the tool based on this library, refer to [the Nu Tracker README on GitHub](https://github.com/matatk/nu-tracker/blob/main/README.md).
pub mod config;
pub use comments::comments;
pub use issues_actions::{actions, issues};
pub use locator::Locator;
pub use specs::specs;

mod comments;
mod flatten_assignees;
mod issues_actions;
mod locator;
mod make_table;
mod returned_issue;
mod showing;
mod specs;
mod status;

pub use status::{flags_labels_conflicts, LabelStringVec, ParseFlagError, Status};
