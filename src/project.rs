use crate::cmd::{CommandError, CommandPlus};
use anyhow::Context;
use fs_err::PathExt;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Stdio;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Project {
    dirpath: PathBuf,
    name: String,
    language: Language,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Language {
    Python,
    Rust,
}

impl Project {
    pub(crate) fn try_for_dirpath(p: PathBuf) -> anyhow::Result<Option<Project>> {
        let pyproject = p.join("pyproject.toml");
        let cargo = p.join("Cargo.toml");
        if pyproject.fs_err_try_exists()? {
            let src = fs_err::read_to_string(&pyproject)?;
            let data = toml::from_str::<Pyproject>(&src)
                .context("failed to deserialize pyproject.toml")?;
            Ok(Some(Project {
                dirpath: p,
                name: data.project.name,
                language: Language::Python,
            }))
        } else if cargo.fs_err_try_exists()? {
            let src = fs_err::read_to_string(&cargo)?;
            let data = toml::from_str::<Cargo>(&src).context("failed to deserialize Cargo.toml")?;
            Ok(Some(Project {
                dirpath: p,
                name: data.package.name,
                language: Language::Rust,
            }))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn on_default_branch(&self) -> anyhow::Result<bool> {
        let current = self.readcmd("git", ["symbolic-ref", "--short", "-q", "HEAD"])?;
        Ok(current == "main" || current == "master")
    }

    pub(crate) fn to_details(&self) -> anyhow::Result<ProjectDetails> {
        Ok(ProjectDetails {
            name: self.name.clone(),
            //dirpath: self.dirpath.to_string_lossy().into_owned(),
            dirpath: self.dirpath.clone(),
            on_default_branch: self.on_default_branch()?,
        })
    }

    pub(crate) fn check<S, I>(&self, cmd: S, args: I) -> anyhow::Result<bool>
    where
        S: AsRef<OsStr>,
        I: IntoIterator<Item: AsRef<OsStr>>,
    {
        let r = CommandPlus::new(cmd)
            .args(args)
            .current_dir(&self.dirpath)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .run();
        match r {
            Ok(()) => Ok(true),
            Err(CommandError::Exit { .. }) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    pub(crate) fn runcmd<S: AsRef<OsStr>>(&self, cmd: S) -> CommandPlus {
        let mut cmd = CommandPlus::new(cmd);
        cmd.current_dir(&self.dirpath);
        cmd
    }

    pub(crate) fn readcmd<S, I>(&self, cmd: S, args: I) -> anyhow::Result<String>
    where
        S: AsRef<OsStr>,
        I: IntoIterator<Item: AsRef<OsStr>>,
    {
        CommandPlus::new(cmd)
            .args(args)
            .current_dir(&self.dirpath)
            .check_output()
            .map(|s| s.trim().to_owned())
            .map_err(Into::into)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ProjectDetails {
    pub(crate) name: String,
    pub(crate) dirpath: PathBuf,
    pub(crate) on_default_branch: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct Pyproject {
    project: NameTable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct Cargo {
    package: NameTable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct NameTable {
    name: String,
}
