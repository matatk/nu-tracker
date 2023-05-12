Nu Tracker
==========

Here are the beginnings of a CLI tool to help W3C Working Groups (WGs) and Task Forces (TFs) track issues logged in GitHub—particularly [GHURLBot-created actions](https://w3c.github.io/GHURLBot/manual.html#create-action) and horizontal review requests—across all of your WG's and TFs' repositories.

**Warning:** This is in the early days of development. As such, it's subject to change. We aim to have it production-ready by [TPAC 2023](https://www.w3.org/2023/09/TPAC/).

**Note:** This tool was created by [@matatk](https://github.com/matatk), who is a co-chair of the [Accessible Platform Architectures WG](https://www.w3.org/WAI/APA/). It's not an official W3C project—any mistakes, in process rules or code, are my own.

This tool is named after [Tracker](https://www.w3.org/2005/06/tracker/), with a hat tip to the [Nu Validator](https://github.com/validator/validator).

What can I do with this tool?
-----------------------------

* Track actions (by due date) across WG's/TFs' repos.

* Track horizontal reviews: specs (by due date), and issue comment requests.

* Make these queries from the perspective of a WG and/or any of its TFs.

When might I _not_ need this tool?
----------------------------------

To find issues specifically assigned to you, you could visit [your assigned GitHub issues page](https://github.com/issues/assigned) or call `gh` as follows.

    gh search issues --assignee @me --state open
    
**Note:** Both of these approaches will include issues assigned to you that are outside of W3C space.

Installation
------------

You'll need to have [Rust](https://www.rust-lang.org/) set up to compile and install Nu Tracker (for now). I haven't got the hang of setting up cross compilation yet.

1. Nu Tracker calls [GitHub's CLI tool `gh`](https://cli.github.com/) to do much of the work. Ensure you have it set up and working. Use `gh auth status` to check that you're logged in to your GitHub account via `gh`.

2. Clone this repo, and `cd` into it. Then use `cargo install --path .` to build in release mode, which will also put the `nt` binary somewhere on your path (via your user or system `cargo` directory).

3. On the first run that doesn't involve only displaying help info, Nu Tracker will create a config directory (details below). You'll be asked for confirmation before it does anything.

How to use it
-------------

There are several sub-commands, used for tracking or doing different things:

* **Issues** (general GitHub issues). The output is done entirely via `gh`, which presents results in a tabular format. This is the least specialised mode of operation.

* **Actions** (GitHub issues with the "action" label). The output of this sub-command is a custom table, sorted by due date.

* **Spec review requests** (horizontal review requests for W3C publications). Again, a custom table, sorted by due date, is provided.

* **Issue comment requests** (horizontal review requests for issues raised by other groups). You can filter for issues with combinations of specific statuses (more on that below).

* There is also a **browse** sub-command that allows you to open any issue from any repo in a browser.

* The **config** sub-command is for managing settings.

* The **help** sub-command (or traditional `--help`/`-h` switches) provide documentation. Each sub-command has specific help, which you can access by calling, e.g. `nt help comments`.

### Usage examples to get you started

* Show actions (issues with the label "action") in all of the WG's repos and all of the TFs' repos, in a table, sorted by ascending due date:

        nt actions -wt

* Assuming we are running from the perspective of the Accessible Platform Architectures WG (more on this below), show actions in all of the Research Questions TF's repos that are assigned to you:

        nt actions -t rq --assignee @me

* Display open spec review requests, in a table, by ascending due date:

        nt specs

* Show issue comment requests that need resolution and have been closed on the source side:

        nt comments --status NC

### What are "main" and "other" repos?

The WG, and each TF, is expected to have at least one repo. This is designated the "main" one, and it's where the group may choose to record general actions.

Both the WG, and TFs, may have more than one repo—e.g. there may be a repo for each separate published document. These repos are the group's "other" repos, and may be used to record document-specific actions. Having "other" repos is optional.

One of the config files that Nu Tracker creates on first run, `repos.json`, stores this information in an easily-editable format, which you can use to share your group's info (more on this later).

### Each run is from the perspective of a single WG

On each invocation, Nu Tracker runs from the perspective of a given WG. On first run, you're asked for your preferred default WG, which is stored in a settings file (details below) and can be changed using the **config** sub-command. You can also pass a WG short name via the `--working-group`/`-g` switch. Known WG names can be found in the `repos.json` config file.

If the default WG name, or the one given the command line, is unknown, Nu Tracker will fall back to "apa".

### How due dates are obtained

#### Actions

Actions' due dates are parsed from the body text of the GitHub issue. The convention is to have the due date (only) on the first line, and add any extra information to the issue's comment after that line.

#### Spec review requests

By convention, the due dates for spec reviews are encoded into the GitHub issue's title.

* If the review request has two dates (in YYYY-MM-DD format) at the end of its title, separated by a greater-than sign or a hyphen and greater-than sign, then the latter of the two dates is taken as the due date.

  The default time window for reviews is 21 days, however Nu Tracker doesn't check the window given via the issue's title.

* If the review request has one date (in YYYY-MM-DD format) at the end of its title, then 21 days are added to that date.

  Nu Tracker doesn't (yet) check that the date in the issue's title is the same as the date the issue was created (or very close). However, convention seems to be that the date in the title is the _start_ date of the review, rather than the end.

### Comment request statuses

**Important:** Please make sure you're familiar with the clear and thorough [official documentation on how to track issue comment requests](https://w3c.github.io/horizontal-issue-tracker/HOWTO).

Each comment request can have a number of labels that indicate its status within the process of managing the request. All possible status labels are listed both in the official documentation, and in the help information for the **comments** sub-command. Examples include "Pending" and "Close?". When using Nu Tracker to query your WG's comment requests, you can filter on any one status, or a combination of them.

Each status has been given a single-character "flag" that's used on the command line, and in Nu Tracker's output.

* To filter comment requests for one or more statuses, use the `--status`/`-s` option. Pass the status flags you're after all together as a single argument, e.g. "P", or "TAP".

  **Note:** The issues returned may have _other_ status labels, too–this just ensures that the ones you asked for are included.

* For a list of status flags and their corresponding labels, use the `--status-flags`/`-f` switch.

Configuration
-------------

On the first run that doesn't involve only displaying help info, Nu Tracker creates a directory containing the following configuration files. The directory will be created in the standard location for such things on your system (with your confirmation).

On macOS, you will be offered a choice of the platform's standard location, or following the XDG standard (i.e. `~/.config/`, or per your `XDG_CONFIG_HOME` environment variable), as would be done on other *nix systems. If you're not sure which to use, go with the first (platform-specific) option. The choice is offered in case it helps you synch your config files across machines, or prefer to have CLI apps use this convention, rather than the macOS standard location.

### `repos.json`

This file contains information on the main, and additional, repositories for WGs and their TFs. You can edit the file to support using the tool with any WG.

#### Contributing repository details for your group

The reason _why_ the `repos.json` file is created is to make it easy for you to add your group's repos to it. Then you could make a PR to have them included by default.

When adding information about a group, here are a couple of guidelines for consistency and ease-of-use:

* Please use the lower-case version of your WG's and TFs' abbreviated names (to make it easier to type them on the command line). E.g. "apa" for "Accessible Platform Architectures".

* Please refrain from including "wg" or "tf" in the group names. E.g. "rq" for APA's Research Questions TF.

#### When there are updates

When changes are made to `repos.json` in this repo, its "version" field will be updated, and an updated version of Nu Tracker will be released. On each run, the version of your local `repos.json` will be checked. If there's a mismatch...

* In most cases, you'll be able to carry on using your version of `repos.json`. If you have made additions for your WG, please consider contributing them.

* You could also delete your local `repos.json` file; Nu Tracker will save the latest version locally on the next run.

* If the _structure_ of `repos.json` is changed in future (which is expected to be rare), then you would need to switch to using that new version, as Nu Tracker will no longer be able to process the previous format. However, you could always use the previous version of the tool until you're ready to move to the new version.

### `settings.json`

This contains your default WG setting. You'll be asked for this on first run, and you can use the **config** sub-command to get or change it.

Accessibility features
----------------------

You can turn off colour output by setting the `NO_COLOR` environment variable to "1". As `gh` supports this too, setting the variable will cover both programs.

For more information, including other programs that support this convention, visit [the NO_COLOR site](https://no-color.org/).

Limitations
-----------

* As per `gh` limitations, only the first/top 30 results will be returned. (If 30 or more results are returned, Nu Tracker will remind you of this.)

  **Warning:** This does mean that, if you have more than 30 open review requests or actions, only the top 30 will be displayed, sorted by due date. **Any older ones will be missed.**

  This will hopefully not be _too_ much of an issue, as this tool is designed to help you keep on top of recent things. It would be possible to alleviate it in future, by using the GitHub GraphQL API—but this will take a lot of work, so is not likely to happen super-soon.

* Because a TF can have multiple WGs as parents, there is some inherent duplication in the `repos.json` file. So far, this seems better (simpler) than de-duping the file, as doing so would make it significantly less human-readable.

Roadmap
-------

### High priority

* Any needed changes to ensure the output is accessible. This may include options to tweak the format of the output.

* Anything needed to make the key tasks listed above smoother/more friendly.

* Checks for horizontal review comment request issues, to ensure that their statuses are set correctly.

* Quality-of-life things such as improved error handling (including sending errors to stderr).

* Using more idiomatic Rust (I am learning the language through writing this tool).

### Long-term

* There are a number of nice-to-haves, such as pagination, that would best be achieved by switching to using GraphQL, but that would require quite a lot of re-work.

* Moving away from `gh`, and querying GitHub directly. Need to research exactly what that would allow that can't be done already, and what constraints it would bring.

* If/when the switch to GraphQL is made, it should be possible to give Nu Tracker the ability to move issue comment review requests to different states within the tracking process.

### Not feasible

* It may sound like offering an option to simplify things and just query all repos under the "w3c" organisation would be a good idea. It would cut down on command-line and `gh` query parameters. However, many W3C-related repos are in other GitHub organisations, so it wouldn't work to just offer the "w3c" organisation as an option. Also, whilst this would be helpful for people trying to track _their_ actions, it wouldn't help WG/TF chairs to track their _group's_ work.

* I would love to be able to search for issues assigned to anyone in a _team_ (e.g. actions assigned to any of APA's members) but it's not possible to do this elegantly with the GitHub API.

* It does not seem feasible to use the [W3C API](https://w3c.github.io/w3c-api/), nor the [Repository Manager](https://labs.w3.org/repo-manager/repos) data to automagically slurp the relationships between WGs, TFs, and their repos. (We need to know, for a given WG: its main and other repos, and all of the main and other repos of the WG's contained TFs.) Suggestions welcome :-).

Contributing
------------

We need your help to collect known WGs and TFs. (I am looking into whether the W3C API could be used for this, but currently it doesn't seem feasible.)

This is not a W3C official project. Currently the simplest way to make a contribution would be to affirm that you assign your copyright to me when you make a PR.

When changes are contributed, I'll add a CONTRIBUTORS file, or similar.

Acknowledgements
----------------

Thanks to [Tracker](https://www.w3.org/2005/06/tracker/) for the years of faithful and helpful service. Thanks to [GHURLBot](https://github.com/w3c/GHURLBot) for taking up the mantle on the creating issues side of things, and to the [Horizontal Review Tracker](https://github.com/w3c/horizontal-issue-tracker) for its helpful web site, and for the issue-labelling infrastructure.
