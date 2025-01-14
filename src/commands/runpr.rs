use crate::github::{CreateLabel, CreatePullRequest, GitHub};
use crate::logging::{logfailures, logproject};
use crate::project::Project;
use crate::util::{Options, RunOpts, Runner};
use clap::Args;
use rand::{rng, seq::IndexedRandom, Rng};
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;
use time::{format_description::FormatItem, macros::format_description, OffsetDateTime};

static DEFAULT_BRANCH_FORMAT: &[FormatItem<'_>] =
    format_description!("forall-runpr-[year][month][day][hour][minute][second]");

// These are the "default colors" listed when creating a label via GitHub's web
// UI as of 2023-09-24:
static NEW_LABEL_COLORS: &[&str] = &[
    "0052cc", "006b75", "0e8a16", "1d76db", "5319e7", "b60205", "bfd4f2", "bfdadc", "c2e0c6",
    "c5def5", "d4c5f9", "d93f0b", "e99695", "f9d0c4", "fbca04", "fef2c0",
];

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

    /// Apply the given label to the new pull requests.  If the label does not
    /// already exist in a repository, it is created.  This option can be
    /// specified multiple times.
    #[arg(short, long, value_name = "NAME")]
    label: Vec<String>,

    /// Commit message [required]
    #[arg(short, long, required = true, value_name = "TEXT")]
    message: String,

    /// Title of the pull requests.  Defaults to the commit message.
    #[arg(short = 'T', long, value_name = "TEXT")]
    pr_title: Option<String>,

    /// File containing the body of the pull requests
    #[arg(short = 'B', long, value_name = "FILE")]
    pr_body_file: Option<PathBuf>,

    /// Apply the given label to the new pull requests.  If the label does not
    /// already exist in a repository, the label is not applied.  This option
    /// can be specified multiple times.
    #[arg(long, value_name = "NAME")]
    soft_label: Vec<String>,

    #[command(flatten)]
    pub(crate) run_opts: RunOpts,
}

impl RunPr {
    pub(crate) fn run(self, opts: Options, projects: Vec<Project>) -> anyhow::Result<()> {
        let github = GitHub::authed()?;
        let mut colorgen = RandomColor::new(rng());
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
            logproject(&p);
            let defbranch = p.default_branch()?;
            p.stash(opts.quiet)?;
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
                log::info!("No changes");
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
            if !self.label.is_empty() || !self.soft_label.is_empty() {
                let label_names = github
                    .get_label_names(ghrepo)?
                    .into_iter()
                    .map(|s| s.to_ascii_lowercase())
                    .collect::<HashSet<_>>();
                let mut labels = Vec::new();
                for lbl in &self.label {
                    if !label_names.contains(&lbl.to_ascii_lowercase()) {
                        github.create_label(
                            ghrepo,
                            CreateLabel {
                                name: Cow::from(lbl),
                                color: Cow::from(colorgen.generate()),
                                description: None,
                            },
                        )?;
                        log::info!("Created label {lbl:?} in {ghrepo}");
                    }
                    labels.push(lbl.as_str());
                }
                for lbl in &self.soft_label {
                    if label_names.contains(&lbl.to_ascii_lowercase()) {
                        labels.push(lbl.as_str());
                    }
                }
                if !labels.is_empty() {
                    github.add_labels_to_pr(ghrepo, pr.number, &labels)?;
                }
            }
        }
        logfailures(failures);
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RandomColor<R>(R);

impl<R: Rng> RandomColor<R> {
    fn new(rng: R) -> RandomColor<R> {
        RandomColor(rng)
    }

    fn generate(&mut self) -> &'static str {
        NEW_LABEL_COLORS
            .choose(&mut self.0)
            .expect("NEW_LABEL_COLORS should be nonempty")
            .to_owned()
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
