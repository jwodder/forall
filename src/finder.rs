use crate::project::Project;
use anyhow::Context;
use clap::Args;
use fs_err::PathExt;
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Finder {
    /// Only operate on projects for which the given shell command succeeds
    #[arg(short, long, value_name = "SHELLCMD", global = true)]
    filter: Option<String>,

    /// Only operate on projects currently on their default branch
    #[arg(short = 'D', long, overrides_with = "no_def_branch", global = true)]
    def_branch: bool,

    /// Only operate on projects currently not on their default branch
    #[arg(long, global = true)]
    no_def_branch: bool,

    /// Directory to traverse for projects [default: current directory]
    #[arg(long, global = true, value_name = "DIRPATH")]
    root: Option<PathBuf>,

    /// Skip the given project.  Can be specified multiple times.
    #[arg(long, global = true, value_name = "NAME")]
    skip: Vec<String>,
}

impl Finder {
    pub(crate) fn findall(&self) -> anyhow::Result<Vec<Project>> {
        let root = match &self.root {
            Some(p) => p.clone(),
            None => std::env::current_dir().context("failed to determine current directory")?,
        };
        let shell = std::env::var_os("SHELL").unwrap_or_else(|| OsString::from("sh"));
        let mut projects = self.find(root, &shell)?;
        projects.sort_unstable_by(|p1, p2| p1.name().cmp(p2.name()));
        Ok(projects)
    }

    fn find(&self, dirpath: PathBuf, shell: &OsStr) -> anyhow::Result<Vec<Project>> {
        let mut projects = Vec::new();
        let ignorefile = dirpath.join(".forall-ignore");
        let exclude = match fs_err::read_to_string(ignorefile) {
            Ok(s) => s.lines().map(ToString::to_string).collect::<HashSet<_>>(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => HashSet::new(),
            Err(e) => return Err(e.into()),
        };
        for entry in fs_err::read_dir(dirpath)? {
            let entry = entry?;
            let fname = entry.file_name();
            let Some(fname) = fname.to_str() else {
                continue;
            };
            if fname.starts_with('.') || exclude.contains(fname) {
                continue;
            }
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let subpath = entry.path();
            if subpath.join(".git").fs_err_try_exists()? {
                if let Some(p) = Project::try_for_dirpath(subpath)? {
                    if self.accept(&p, shell)? {
                        projects.push(p);
                    }
                }
            } else {
                projects.extend(self.find(subpath, shell)?);
            }
        }
        Ok(projects)
    }

    fn accept(&self, p: &Project, shell: &OsStr) -> anyhow::Result<bool> {
        if self.skip.iter().any(|name| name == p.name()) {
            return Ok(false);
        }
        if let Some(flag) = self.def_branch() {
            if p.on_default_branch()? != flag {
                return Ok(false);
            }
        }
        if let Some(ref cmd) = self.filter {
            if !p.check(shell, ["-c", cmd])? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn def_branch(&self) -> Option<bool> {
        match (self.def_branch, self.no_def_branch) {
            (false, false) => None,
            (true, false) => Some(true),
            (false, true) => Some(false),
            (true, true) => unreachable!(),
        }
    }
}
