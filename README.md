[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![CI Status](https://github.com/jwodder/forall/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/forall/actions/workflows/test.yml)
[![codecov.io](https://codecov.io/gh/jwodder/forall/branch/main/graph/badge.svg)](https://codecov.io/gh/jwodder/forall)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.86-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/forall.svg)](https://opensource.org/licenses/MIT)

[GitHub](https://github.com/jwodder/forall) | [Issues](https://github.com/jwodder/forall/issues) | [Changelog](https://github.com/jwodder/forall/blob/main/CHANGELOG.md)

`forall` is my personal [Rust](https://www.rust-lang.org) program for
performing various operations on multiple local Git repositories at once.  It
traverses one or more directory trees looking for projects and runs a specified
command on each of them, possibly after excluding certain projects.

Currently, only Git repositories containing Rust projects or
`pyproject.toml`-based Python projects are supported.

While this program may in theory be suitable for general use, I make no
guarantees, nor do I intend to release it for general consumption.  Use at your
own risk.

Usage
=====

    forall [<global options>] <subcommand> ...

Global Options
--------------

All of the following options can be supplied either before or after the
subcommand.

- `-D`, `--def-branch` — Only operate on projects currently on their default
  branch (`main` or `master`)

- `--no-def-branch` — Only operate on projects currently not on their default
  branch

- `--exclude <name>` — Do not operate on the given project.  This option can be
  specified multiple times.

- `-f <shellcmd>`, `--filter <shellcmd>` — Run `$SHELL -c <shellcmd>` with the
  current working directory set to each project's directory and only operate on
  those projects for which the command succeeds

- `--has-github` — Only operate on projects that have GitHub remotes

- `--no-github` — Only operate on projects that do not have GitHub remotes

- `--has-stash` — Only operate on projects that have stashed changes

- `--no-stash` — Only operate on projects that do not have stashed changes

- `-L <language>`, `--language <language>` — Only operate on projects written
  in the given language.  Possible options are "Python"/"py" and "Rust"/"rs"
  (all case-insensitive).

- `-k`, `--keep-going` — By default, if a subcommand fails or another error
  occurs for a project, `forall` terminates immediately.  If `--keep-going` is
  supplied, `forall` will instead continue with the remaining projects and will
  print a list of all failures on exit.

- `-q`, `--quiet` — Be less verbose; this option can be specified multiple
  times.  See "Logging" below for more infomation.

- `-R <dirpath>`, `--root <dirpath>` — Start traversing from `<dirpath>`.  This
  option can be specified multiple times to traverse multiple directories.
  [default: the current working directory]

- `-W`, `--workspace` — Only operate on projects that are Rust workspaces

-  `--not-workspace` — Only operate on projects that are not Rust workspaces

- `--virtual` — Only operate on projects that are Rust virtual workspaces

-  `--not-virtual` — Only operate on projects that are not Rust virtual
   workspaces

- `-v`, `--verbose` — Be more verbose.  See "Logging" below for more
  information.

Project Names
-------------

Each project is identified by a name, which is output when operating on the
project and accepted by the `--exclude` option in order to not operate on a
project.  Project names are determined as follows:

- For Python projects and non-workspace Rust projects, the name is the metadata
  name of the sole package in the project.

- For non-virtual Rust workspaces, the name is the metadata name of the root
  package.

- For virtual Rust workspaces, the project's `Cargo.toml` must set
  `workspace.package.repository` to a GitHub repository URL, and the name of
  this repository is used as the project name.

Logging
-------

`forall` logs various messages to stdout and/or stderr during its operation.
The `-q`/`--quiet` and `-v`/`--verbose` options can be used to control which
messages are shown.  The following table indicates when each type of message is
shown for each quiet/verbose level:

| Message Type                      | `-qq` | `-q` |  —  | `-v` | Style  | Stream |
| --------------------------------- | :---: | :--: | :-: | :--: | ------ | ------ |
| Project names                     | ✓     | ✓    | ✓   | ✓    | Bold   | stdout |
| Errors                            | ✓     | ✓    | ✓   | ✓    | Red    | stderr |
| Lists of failures                 | ✓     | ✓    | ✓   | ✓    | Plain  | stdout |
| `run` and `runpr` commands        | ✗     | ✗    | ✓   | ✓    | Cyan   | stderr |
| `run` and `runpr` commands output | ✗     | ✓    | ✓   | ✓    | Plain  | stdout |
| Operational commands              | ✗     | ✗    | ✓   | ✓    | Cyan   | stderr |
| Operational commands output       | ✗     | ✗    | ✓   | ✓    | Plain  | stdout |
| Filter commands                   | ✗     | ✗    | ✗   | ✓    | Cyan   | stderr |
| Filter commands output            | ✗     | ✗    | ✗   | ✗    | —      | —      |
| HTTP requests                     | ✗     | ✗    | ✗   | ✓    | Cyan   | stderr |
| Messages about skipped projects   | ✗     | ✗    | ✗   | ✓    | Yellow | stderr |
| Other informative messages        | ✗     | ✗    | ✓   | ✓    | Yellow | stderr |

Notes:

- If both `--quiet` and `--verbose` are specified on the command line, the
  `--verbose` negates one instance of `--quiet`.

- Passing more than two `--quiet` options (after applying `--verbose` negation)
  is equivalent to passing just two.

- "Errors" includes captured output from failed commands whose output would
  otherwise be suppressed.

- The "commands" message types are messages showing each executed command,
  including arguments and working directory.

- "`run` and `runpr` commands" are commands passed to the `run` and `runpr`
  subcommands for execution.

- "Operational commands" are miscellaneous commands run by `forall`, such as
  `git commit` for `runpr` or `pre-commit autoupdate` for `pre-update`.

- "Filter commands" are commands run in order to determine whether to operate
  on a project.

`forall list`
-------------

    forall [<global options>] list [<options>]

Print the name of each project in the directory tree in sorted order.

### Options

- `-J`, `--json` — Instead of printing the name of each project, print
  newline-delimited JSON objects describing each project.  Each object contains
  the following fields:
    - `name` — project name
    - `dirpath` — path to the directory in which the project is located
    - `language` — the project's language (`"Python"` or `"Rust"`)
    - `ghrepo` — the project's remote GitHub repository in `{owner}/{name}`
      format, or `null` if it does not have a GitHub remote
    - `on_default_branch` — `true` if the Git repository is currently on the
      default branch (`main` or `master`), `false` otherwise
    - `is_workspace` — `true` iff the project is a Rust workspace
    - `is_virtual_workspace` — `true` iff the project is a Rust virtual
      workspace

`forall clean`
-------------

    forall [<global options>] clean

Run `git clean -dXf` on each project that needs it

`forall cloc`
-------------

    forall [<global options>] cloc

Use [`cloc`](https://github.com/AlDanial/cloc/) to count the number of
effective lines in each project, and output a simple table of the results.

`forall gc`
-----------

    forall [<global options>] gc

Run `git gc` on each project

`forall pre-update`
-------------------

    forall [<global options>] pre-update

Run `pre-commit autoupdate` on all projects with `.pre-commit-config.yaml`
files.  `pre-commit run -a` is then run to apply any new formatting, followed
by a second `pre-commit run -a` to ensure that linting is still successful.
Any & all changes are then committed.

`forall pull`
-------------

    forall [<global options>] pull

Run `git pull` on each project that has a GitHub remote

`forall push`
-------------

    forall [<global options>] push

Run `git push` on each project that has a GitHub remote and for which `HEAD` is
ahead of `@{upstream}`

`forall rsclean`
----------------

    forall [<global options>] rsclean

Run `cargo clean` on each Rust project that contains a `target/` directory

`forall run`
------------

    forall [<global options>] run [<options>] <command> [<args> ...]

Run the given command on each project.

The command is run with the current working directory set to each respective
project's directory.

### Options

- `--script` — Treat the command as a path to a script file.  The path is
  canonicalized, and it is run via `perl` for its shebang handling; thus, the
  script need not be executable, but it does need to have an appropriate
  shebang.

- `--shell` — Run the command with `$SHELL -c <command> <args>`

- `-s`, `--stash` — Stash any uncommitted changes before running the command

`forall run-pr`
---------------

    forall [<global options>] run-pr [<options>] <command> [<args> ...]

Run the given command on each project and submit the changes as a GitHub pull
request.

Specifically, for each project that has a non-archived GitHub remote:

- Any uncommitted changes are stashed.

- A new branch is created (starting from the default branch) and checked out.

- The command is run on the new branch, with the current working directory set
  to the project's directory.
    - The command should not perform any Git commits itself at this time.

- `git add .` is run.  If there are no staged changes afterwards, then the
  project's default branch is checked out, the PR branch is deleted, and no
  further steps are taken.

- `git commit` is run, and the branch is pushed to the `origin` remote.

- A pull request is created in the corresponding GitHub repository, and the URL
  of the PR is output.

This command requires a GitHub access token to have been either set via the
`GH_TOKEN` or `GITHUB_TOKEN` environment variable or else saved with
[`gh`](https://github.com/cli/cli) in order to interact with the GitHub REST
API.

### Options

- `-b <NAME>`, `--branch <NAME>` — Set the name for the new branch from which
  the pull request is created.  Defaults to `forall-runpr-%Y%m%d%H%M%S`.

- `-B <FILE>`, `--pr-body-file <FILE>` — Path to a file containing the body to
  use for the pull requests.  If not specified, the PRs will have empty bodies.

- `-l <NAME>`, `--label <NAME>` — Apply the given label to the new pull
  requests.  If the label does not already exist in a repository, it is
  created.  This option can be specified multiple times.

- `-m <TEXT>`, `--message <TEXT>` — The commit message to use.  This option is
  required.

- `--script` — Treat the command as a path to a script file.  The path is
  canonicalized, and it is run via `perl` for its shebang handling; thus, the
  script need not be executable, but it does need to have an appropriate
  shebang.

- `--shell` — Run the command with `$SHELL -c <command> <args>`

- `--soft-label <NAME>` — Apply the given label to the new pull requests.  If
  the label does not already exist in a repository, the label is not applied.
  This option can be specified multiple times.

- `-T <TEXT>`, `--pr-title <TEXT>` — The title to give the pull requests.
  Defaults to the commit message with `[skip ci]` and similar strings removed.
