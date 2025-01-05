use crate::github::{CreatePullRequest, GitHub};
use crate::project::Project;
use crate::util::{Options, RunOpts, Runner};
use clap::Args;
use std::borrow::Cow;
use std::path::PathBuf;
use time::{format_description::FormatItem, macros::format_description, OffsetDateTime};

static DEFAULT_BRANCH_FORMAT: &[FormatItem<'_>] =
    format_description!("forall-runpr-[year][month][day][hour][minute][second]");

/// Run a command on each project and submit the changes as a GitHub pull
/// request.
///
/// Only projects that have non-archived GitHub remotes are considered.
///
/// The command is run with the current working directory set to each
/// respective project's directory.
#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunPr {
    /// Name for the new pull request branch.
    ///
    /// Defaults to `forall-runpr-%Y%m%d%H%M%S`.
    #[arg(short, long, value_name = "NAME")]
    branch: Option<String>,

    /// Commit message [required]
    #[arg(short, long, required = true, value_name = "TEXT")]
    message: String,

    /// Title of the pull requests.  Defaults to the commit message.
    #[arg(short = 'T', long, value_name = "TEXT")]
    pr_title: Option<String>,

    /// File containing the body of the pull requests
    #[arg(short = 'B', long, value_name = "FILE")]
    pr_body_file: Option<PathBuf>,

    #[command(flatten)]
    pub(crate) run_opts: RunOpts,
}

impl RunPr {
    pub(crate) fn run(self, opts: Options, projects: Vec<Project>) -> anyhow::Result<()> {
        let github = GitHub::authed()?;
        let branch = match self.branch {
            Some(b) => b,
            None => OffsetDateTime::now_local()
                .unwrap_or_else(|_| OffsetDateTime::now_utc())
                .format(&DEFAULT_BRANCH_FORMAT)
                .expect("formatting a datetime should not fail"),
        };
        let pr_title = self
            .pr_title
            .as_deref()
            .unwrap_or_else(|| strip_skip(&self.message));
        let pr_body = match self.pr_body_file {
            Some(p) => Some(fs_err::read_to_string(p)?),
            None => None,
        };
        let runner = Runner::try_from(self.run_opts)?;
        let mut failures = Vec::new();
        for p in projects {
            let Some(ghrepo) = p.ghrepo() else {
                continue;
            };
            if github.get_repository(ghrepo)?.archived {
                continue;
            }
            boldln!("{}", p.name());
            let defbranch = p.default_branch()?;
            p.stash()?;
            p.runcmd("git")
                .arg("checkout")
                .arg("-b")
                .arg(&branch)
                .arg(defbranch)
                .quiet(opts.quiet)
                .run()?;
            if !runner.run(&p, opts)? {
                failures.push(p);
                continue;
            }
            p.runcmd("git").args(["add", "."]).quiet(opts.quiet).run()?;
            // XXX: When adding support for commands that commit, also check
            //      whether $branch is ahead of $defbranch.
            if !p.has_staged_changes()? {
                println!("> No changes"); // TODO: Style output?
                p.runcmd("git")
                    .arg("checkout")
                    .arg(defbranch)
                    .quiet(opts.quiet)
                    .run()?;
                p.runcmd("git")
                    .args(["branch", "-d"])
                    .arg(&branch)
                    .quiet(opts.quiet)
                    .run()?;
                continue;
            }
            p.runcmd("git")
                .args(["commit", "-m"])
                .arg(&self.message)
                .quiet(opts.quiet)
                .run()?;
            p.runcmd("git")
                .args(["push", "--set-upstream", "origin"])
                .arg(&branch)
                .quiet(opts.quiet)
                .run()?;
            let pr = github.create_pull_request(
                ghrepo,
                CreatePullRequest {
                    title: Cow::from(pr_title),
                    head: Cow::from(&branch),
                    base: Cow::from(defbranch),
                    body: pr_body.as_deref().map(Cow::from),
                    maintainer_can_modify: true,
                },
            )?;
            println!("{}", pr.html_url); // TODO: Improve?
        }
        if !failures.is_empty() {
            boldln!("\nFailures:");
            for p in failures {
                println!("{}", p.name());
            }
        }
        Ok(())
    }
}

fn strip_skip(mut s: &str) -> &str {
    // <https://docs.github.com/en/actions/managing-workflow-runs-and-deployments/managing-workflow-runs/skipping-workflow-runs>
    // TODO: Delete skip strings in the middle of a commit message
    for skipper in [
        "[skip ci]",
        "[ci skip]",
        "[no ci]",
        "[skip actions]",
        "[actions skip]",
    ] {
        s = s.strip_prefix(skipper).unwrap_or(s);
        s = s.strip_suffix(skipper).unwrap_or(s);
        s = s.trim();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("Foo some bars", "Foo some bars")]
    #[case("[skip ci] Foo some bars", "Foo some bars")]
    #[case("Foo some bars [skip ci]", "Foo some bars")]
    #[case("[ci skip] Foo some bars", "Foo some bars")]
    #[case("Foo some bars [ci skip]", "Foo some bars")]
    #[case("[no ci] Foo some bars", "Foo some bars")]
    #[case("Foo some bars [no ci]", "Foo some bars")]
    #[case("[skip actions] Foo some bars", "Foo some bars")]
    #[case("Foo some bars [skip actions]", "Foo some bars")]
    #[case("[actions skip] Foo some bars", "Foo some bars")]
    #[case("Foo some bars [actions skip]", "Foo some bars")]
    #[case("[skip] Foo some bars", "[skip] Foo some bars")]
    #[case("[ci] Foo some bars", "[ci] Foo some bars")]
    #[case("[skipci] Foo some bars", "[skipci] Foo some bars")]
    #[case("skip ci Foo some bars", "skip ci Foo some bars")]
    fn test_strip_skip(#[case] before: &str, #[case] after: &str) {
        assert_eq!(strip_skip(before), after);
    }
}
