[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![CI Status](https://github.com/jwodder/forall/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/forall/actions/workflows/test.yml)
[![codecov.io](https://codecov.io/gh/jwodder/forall/branch/main/graph/badge.svg)](https://codecov.io/gh/jwodder/forall)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.79-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/forall.svg)](https://opensource.org/licenses/MIT)

[GitHub](https://github.com/jwodder/forall) | [Issues](https://github.com/jwodder/forall/issues) | [Changelog](https://github.com/jwodder/forall/blob/main/CHANGELOG.md)

`forall` is a [Rust](https://www.rust-lang.org) program for performing various
operations on multiple local Git repositories at once.  It traverses a
directory tree looking for projects and runs a specified command on each of
them, possibly after excluding certain projects.

Currently, only Git repositories containing Rust projects or
`pyproject.toml`-based Python projects are supported.

Usage
=====

    forall [<global options>] <subcommand> ...

Global Options
--------------

All of the following options can be supplied either before or after the
subcommand.

- `-D`, `--def-branch` — Only operate on projects currently on their default
  branch (`main` or `master`)

- `-f <shellcmd>`, `--filter <shellcmd>` — Run `$SHELL -c <shellcmd>` in each
  project and only operate on those for which the command succeeds

- `-k`, `--keep-going` — By default, if a subcommand fails for a project,
  `forall` terminates immediately.  If `--keep-going` is supplied, `forall`
  will instead continue with the remaining projects and will print a list of
  all failures on exit.

- `--no-def-branch` — Only operate on projects currently not on their default
  branch

- `-q`, `--quiet` — Suppress successful command output

- `--root <dirpath>` — Start traversing from `<dirpath>` [default: the current
  working directory

- `--skip <name>` — Do not operate on the given project.  This option can be
  specified multiple times.

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

Run `git pull` on each project

`forall push`
-------------

    forall [<global options>] push

Run `git push` on each project for which `HEAD` is ahead of `@{upstream}`

`forall run`
------------

    forall [<global options>] run [<options>] <command> [<args> ...]

Run the given command on each project.

### Options

- `--shell` — Run the command with `$SHELL -c <command>`

`forall script`
---------------

    forall [<global options>] script <scriptfile>

Run the script `<scriptfile>` on each project.  The script is run via `perl`
for its shebang-handling, so the script need not be executable, but it does
need to have an appropriate shebang.
