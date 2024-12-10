use crate::project::Project;
use clap::Args;

#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct List;

impl List {
    pub(crate) fn run(self, projects: Vec<Project>) {
        for p in projects {
            println!("{}", p.name());
        }
    }
}
