use crate::project::Project;
use clap::Args;

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

    /// Skip the given project
    #[arg(long, global = true)]
    skip: Vec<String>,
}

impl Finder {
    pub(crate) fn findall(&self) -> Vec<Project> {
        todo!()
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
