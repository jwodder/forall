use crate::cmd::{CommandError, CommandPlus};
use anyhow::Context;
use cargo_metadata::{MetadataCommand, TargetKind};
use fs_err::PathExt;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Project {
    dirpath: PathBuf,
    name: String,
    language: Language,
    is_workspace: bool,
    is_virtual_workspace: bool,
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
                is_workspace: false,
                is_virtual_workspace: false,
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

    pub(crate) fn on_default_branch(&self) -> anyhow::Result<bool> {
        let current = self.readcmd("git", ["symbolic-ref", "--short", "-q", "HEAD"])?;
        Ok(current == "main" || current == "master")
    }

    pub(crate) fn to_details(&self) -> anyhow::Result<ProjectDetails> {
        Ok(ProjectDetails {
            name: self.name.clone(),
            dirpath: self.dirpath.clone(),
            on_default_branch: self.on_default_branch()?,
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
                let mut srcs = std::collections::HashSet::new();
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

    pub(crate) fn stash(&self) -> anyhow::Result<()> {
        if !self
            .readcmd("git", ["status", "--porcelain", "-uno"])?
            .is_empty()
        {
            self.runcmd("git").arg("stash").run()?;
        }
        Ok(())
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
            Ok(_) => Ok(true),
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(crate) enum Language {
    Python,
    Rust,
}

impl Language {
    pub(crate) fn ext(&self) -> &'static str {
        match self {
            Language::Python => "py",
            Language::Rust => "rs",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ProjectDetails {
    pub(crate) name: String,
    pub(crate) dirpath: PathBuf,
    pub(crate) language: Language,
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
    repository: ghrepo::GHRepo,
}
