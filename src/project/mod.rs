mod lang;
pub(crate) use self::lang::*;
use crate::cmd::{CommandError, CommandKind, CommandPlus};
use crate::util::get_ghrepo;
use anyhow::Context;
use cargo_metadata::{MetadataCommand, TargetKind};
use fs_err::PathExt;
use ghrepo::GHRepo;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use thiserror::Error;

static DEFAULT_BRANCHES: &[&str] = &["main", "master"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Project {
    dirpath: PathBuf,
    name: String,
    language: Language,
    is_workspace: bool,
    is_virtual_workspace: bool,
    ghrepo: Option<GHRepo>,
}

impl Project {
    pub(crate) fn try_for_dirpath(p: PathBuf) -> anyhow::Result<Option<Project>> {
        let ghrepo = get_ghrepo(&p)?;
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
                is_workspace: false,
                is_virtual_workspace: false,
                ghrepo,
            }))
        } else if cargo.fs_err_try_exists()? {
            let src = fs_err::read_to_string(&cargo)?;
            let data = toml::from_str::<Cargo>(&src)
                .with_context(|| format!("failed to deserialize {}", cargo.display()))?;
            Ok(Some(Project {
                dirpath: p,
                name: data.name().to_owned(),
                language: Language::Rust,
                is_workspace: data.is_workspace(),
                is_virtual_workspace: data.is_virtual_workspace(),
                ghrepo,
            }))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn dirpath(&self) -> &Path {
        &self.dirpath
    }

    pub(crate) fn language(&self) -> Language {
        self.language
    }

    pub(crate) fn has_github(&self) -> bool {
        self.ghrepo.is_some()
    }

    pub(crate) fn ghrepo(&self) -> Option<&GHRepo> {
        self.ghrepo.as_ref()
    }

    pub(crate) fn on_default_branch(&self) -> anyhow::Result<bool> {
        let current = self.readcmd("git", ["symbolic-ref", "--short", "-q", "HEAD"])?;
        Ok(DEFAULT_BRANCHES.iter().any(|&b| b == current))
    }

    pub(crate) fn default_branch(&self) -> anyhow::Result<&'static str> {
        let branches = self
            .readcmd("git", ["branch", "--format=%(refname:short)"])?
            .lines()
            .map(ToString::to_string)
            .collect::<HashSet<_>>();
        for &guess in DEFAULT_BRANCHES {
            if branches.contains(guess) {
                return Ok(guess);
            }
        }
        anyhow::bail!("Could not determine default branch for {}", self.name())
    }

    pub(crate) fn to_details(&self) -> anyhow::Result<ProjectDetails> {
        Ok(ProjectDetails {
            name: self.name.clone(),
            dirpath: self.dirpath.clone(),
            on_default_branch: self.on_default_branch()?,
            ghrepo: self.ghrepo.clone(),
            language: self.language,
            is_workspace: self.is_workspace,
            is_virtual_workspace: self.is_virtual_workspace,
        })
    }

    pub(crate) fn source_paths(&self) -> anyhow::Result<Vec<PathBuf>> {
        match self.language {
            Language::Python => {
                if self.dirpath.join("src").fs_err_try_exists()? {
                    Ok(vec![PathBuf::from("src")])
                } else {
                    let mut srcs = Vec::new();
                    for entry in fs_err::read_dir(&self.dirpath)? {
                        let entry = entry?;
                        let name = PathBuf::from(entry.file_name());
                        if name.to_string_lossy().ends_with(".py") {
                            srcs.push(name);
                        }
                    }
                    Ok(srcs)
                }
            }
            Language::Rust => {
                let packages = MetadataCommand::new()
                    .manifest_path(self.dirpath.join("Cargo.toml"))
                    .no_deps()
                    .exec()
                    .context("failed to get project metadata")?
                    .packages;
                let mut srcs = HashSet::new();
                for p in packages {
                    for t in p.targets {
                        if t.kind.iter().any(|k| {
                            matches!(k, TargetKind::Lib | TargetKind::Bin | TargetKind::ProcMacro)
                        }) {
                            let Some(pardir) = t.src_path.parent() else {
                                anyhow::bail!("Could not determine parent directory of src_path {} for project {}", t.src_path, self.name());
                            };
                            srcs.insert(pardir.to_owned());
                        }
                    }
                }
                Ok(srcs.into_iter().map(PathBuf::from).collect())
            }
        }
    }

    pub(crate) fn has_staged_changes(&self) -> anyhow::Result<bool> {
        Ok(self
            .runcmd("git")
            .args(["diff", "--cached", "--quiet"])
            .kind(CommandKind::Filter)
            .status()?
            .code()
            == Some(1))
    }

    pub(crate) fn stash(&self) -> anyhow::Result<()> {
        // TODO: Should --ignore-submodules be set to something?
        if !self
            .readcmd("git", ["status", "--porcelain", "-unormal"])?
            .is_empty()
        {
            self.runcmd("git").args(["stash", "-u"]).run()?;
        }
        Ok(())
    }

    pub(crate) fn has_stash(&self) -> anyhow::Result<bool> {
        let r = self.readcmd("git", ["rev-parse", "--verify", "--quiet", "refs/stash"]);
        match r {
            Ok(stdout) => Ok(!stdout.is_empty()),
            Err(CommandError::Exit { rc, .. }) if rc.code() == Some(1) => Ok(false),
            Err(e) => Err(e.into()),
        }
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

    pub(crate) fn readcmd<S, I>(&self, cmd: S, args: I) -> Result<String, CommandError>
    where
        S: AsRef<OsStr>,
        I: IntoIterator<Item: AsRef<OsStr>>,
    {
        CommandPlus::new(cmd)
            .args(args)
            .current_dir(&self.dirpath)
            .kind(CommandKind::Filter)
            .check_output()
            .map(|s| s.trim().to_owned())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ProjectDetails {
    pub(crate) name: String,
    pub(crate) dirpath: PathBuf,
    pub(crate) language: Language,
    pub(crate) ghrepo: Option<GHRepo>,
    pub(crate) on_default_branch: bool,
    pub(crate) is_workspace: bool,
    pub(crate) is_virtual_workspace: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct Pyproject {
    project: NameTable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(try_from = "RawCargo")]
enum Cargo {
    Package {
        package: NameTable,
    },
    Workspace {
        workspace: Workspace,
        package: NameTable,
    },
    Virtual {
        workspace: Workspace,
    },
}

impl Cargo {
    fn name(&self) -> &str {
        match self {
            Cargo::Workspace { package, .. } => &package.name,
            Cargo::Virtual { workspace } => workspace.package.repository.name(),
            Cargo::Package { package } => &package.name,
        }
    }

    fn is_workspace(&self) -> bool {
        matches!(self, Cargo::Workspace { .. } | Cargo::Virtual { .. })
    }

    fn is_virtual_workspace(&self) -> bool {
        matches!(self, Cargo::Virtual { .. })
    }
}

impl TryFrom<RawCargo> for Cargo {
    type Error = FromRawCargoError;

    fn try_from(value: RawCargo) -> Result<Cargo, FromRawCargoError> {
        match value {
            RawCargo {
                package: Some(package),
                workspace: None,
            } => Ok(Cargo::Package { package }),
            RawCargo {
                package: Some(package),
                workspace: Some(workspace),
            } => Ok(Cargo::Workspace { workspace, package }),
            RawCargo {
                package: None,
                workspace: Some(workspace),
            } => Ok(Cargo::Virtual { workspace }),
            RawCargo {
                package: None,
                workspace: None,
            } => Err(FromRawCargoError),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("Cargo.toml lacks both [package] and [workspace] tables")]
pub(crate) struct FromRawCargoError;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct NameTable {
    name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct RawCargo {
    package: Option<NameTable>,
    workspace: Option<Workspace>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct Workspace {
    package: WorkspacePackage,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct WorkspacePackage {
    repository: GHRepo,
}
