use anyhow::Context;
use fs_err::PathExt;
use serde::Deserialize;
use std::path::PathBuf;

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
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct Pyproject {
    project: NameTable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct Cargo {
    package: NameTable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct NameTable {
    name: String,
}
