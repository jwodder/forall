use crate::project::{Language, Project};
use crate::util::get_shell;
use anyhow::Context;
use clap::Args;
use fs_err::PathExt;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Finder {
    /// Only operate on projects currently on their default branch
    #[arg(short = 'D', long, overrides_with = "no_def_branch", global = true)]
    def_branch: bool,

    /// Only operate on projects currently not on their default branch
    #[arg(long, global = true)]
    no_def_branch: bool,

    /// Don't operate on the given project.  Can be specified multiple times.
    #[arg(long, global = true, value_name = "NAME")]
    exclude: Vec<String>,

    /// Only operate on projects for which the given shell command succeeds
    ///
    /// The command is run with the current working directory set to each
    /// respective project's directory.
    #[arg(short, long, value_name = "SHELLCMD", global = true)]
    filter: Option<String>,

    /// Only operate on projects that have GitHub remotes
    #[arg(long, overrides_with = "no_github", global = true)]
    has_github: bool,

    /// Only operate on projects that do not have GitHub remotes
    #[arg(long, global = true)]
    no_github: bool,

    /// Only operate on projects that have stashed changes
    #[arg(long, overrides_with = "no_stash", global = true)]
    has_stash: bool,

    /// Only operate on projects that do not have stashed changes
    #[arg(long, global = true)]
    no_stash: bool,

    /// Only operate on projects written in the given language
    ///
    /// Possible options are "Python"/"py" and "Rust"/"rs" (all
    /// case-insensitive).
    #[arg(short = 'L', long, global = true)]
    language: Option<Language>,

    /// Directory to traverse for projects.  Can be specified multiple times to
    /// traverse multiple directories.  [default: current directory]
    #[arg(short = 'R', long, global = true, value_name = "DIRPATH")]
    root: Vec<PathBuf>,

    /// Only operate on Rust workspaces
    #[arg(short = 'W', long, overrides_with = "not_workspace", global = true)]
    workspace: bool,

    /// Only operate on projects that are not Rust workspaces
    #[arg(long, global = true)]
    not_workspace: bool,

    /// Only operate on Rust virtual workspaces
    #[arg(long, overrides_with = "not_virtual", global = true)]
    r#virtual: bool,

    /// Only operate on projects that are not Rust virtual workspaces
    #[arg(long, global = true)]
    not_virtual: bool,
}

impl Finder {
    pub(crate) fn findall(&self) -> anyhow::Result<Vec<Project>> {
        let roots = if self.root.is_empty() {
            &vec![std::env::current_dir().context("failed to determine current directory")?]
        } else {
            &self.root
        };
        let mut projects = Vec::new();
        for dirpath in roots {
            projects.extend(self.find(dirpath, &get_shell())?);
        }
        projects.sort_unstable_by(|p1, p2| p1.name().cmp(p2.name()));
        Ok(projects)
    }

    fn find(&self, dirpath: &Path, shell: &OsStr) -> anyhow::Result<Vec<Project>> {
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
                if let Some(p) = Project::try_for_dirpath(subpath)?
                    && self.accept(&p, shell)?
                {
                    projects.push(p);
                }
            } else {
                projects.extend(self.find(&subpath, shell)?);
            }
        }
        Ok(projects)
    }

    fn accept(&self, p: &Project, shell: &OsStr) -> anyhow::Result<bool> {
        if self.exclude.iter().any(|name| name == p.name()) {
            return Ok(false);
        }
        if let Some(flag) = self.def_branch()
            && p.on_default_branch()? != flag
        {
            return Ok(false);
        }
        if let Some(flag) = self.has_github()
            && p.has_github() != flag
        {
            return Ok(false);
        }
        if let Some(flag) = self.has_stash()
            && p.has_stash()? != flag
        {
            return Ok(false);
        }
        if let Some(lang) = self.language
            && p.language() != lang
        {
            return Ok(false);
        }
        if let Some(flag) = self.is_workspace()
            && p.is_workspace() != flag
        {
            return Ok(false);
        }
        if let Some(flag) = self.is_virtual()
            && p.is_virtual_workspace() != flag
        {
            return Ok(false);
        }
        if let Some(ref cmd) = self.filter
            && !p.check(shell, ["-c", cmd])?
        {
            return Ok(false);
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

    fn has_github(&self) -> Option<bool> {
        match (self.has_github, self.no_github) {
            (false, false) => None,
            (true, false) => Some(true),
            (false, true) => Some(false),
            (true, true) => unreachable!(),
        }
    }

    fn has_stash(&self) -> Option<bool> {
        match (self.has_stash, self.no_stash) {
            (false, false) => None,
            (true, false) => Some(true),
            (false, true) => Some(false),
            (true, true) => unreachable!(),
        }
    }

    fn is_workspace(&self) -> Option<bool> {
        match (self.workspace, self.not_workspace) {
            (false, false) => None,
            (true, false) => Some(true),
            (false, true) => Some(false),
            (true, true) => unreachable!(),
        }
    }

    fn is_virtual(&self) -> Option<bool> {
        match (self.r#virtual, self.not_virtual) {
            (false, false) => None,
            (true, false) => Some(true),
            (false, true) => Some(false),
            (true, true) => unreachable!(),
        }
    }
}
