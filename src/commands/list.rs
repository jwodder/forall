use crate::finder::Finder;
use clap::Args;

#[derive(Args, Clone, Debug, Eq, PartialEq)]
pub(crate) struct List;

impl List {
    pub(crate) fn run(self, finder: Finder) -> anyhow::Result<()> {
        todo!()
    }
}
