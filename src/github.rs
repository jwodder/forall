use anyhow::Context;
use ghrepo::GHRepo;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

static USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("CARGO_PKG_REPOSITORY"),
    ")",
);

#[derive(Clone, Debug)]
pub(crate) struct GitHub(minigh::Client);

impl GitHub {
    pub(crate) fn new(token: &str) -> Result<GitHub, minigh::BuildClientError> {
        Ok(GitHub(
            minigh::Client::builder()
                .with_token(token)
                .with_user_agent(USER_AGENT)
                .build()?,
        ))
    }

    pub(crate) fn authed() -> anyhow::Result<GitHub> {
        let token = gh_token::get().context("Failed to retrieve GitHub token")?;
        GitHub::new(&token).map_err(Into::into)
    }

    pub(crate) fn get_repository<R>(&self, repo: &R) -> anyhow::Result<Repository>
    where
        for<'a> R: RepositoryEndpoint<'a>,
    {
        self.0.get(repo.api_url().as_ref()).map_err(Into::into)
    }

    pub(crate) fn create_pull_request<R>(
        &self,
        repo: &R,
        pr: CreatePullRequest<'_>,
    ) -> anyhow::Result<PullRequest>
    where
        for<'a> R: RepositoryEndpoint<'a>,
    {
        self.0
            .post(&format!("{}/pulls", repo.api_url().as_ref()), &pr)
            .map_err(Into::into)
    }

    pub(crate) fn get_label_names<R>(&self, repo: &R) -> anyhow::Result<Vec<String>>
    where
        for<'a> R: RepositoryEndpoint<'a>,
    {
        self.0
            .paginate::<LabelInfo>(&format!("{}/labels", repo.api_url().as_ref()))
            .map_ok(|li| li.name)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub(crate) fn create_label<R>(&self, repo: &R, label: CreateLabel<'_>) -> anyhow::Result<()>
    where
        for<'a> R: RepositoryEndpoint<'a>,
    {
        self.0.post::<_, serde::de::IgnoredAny>(
            &format!("{}/labels", repo.api_url().as_ref()),
            &label,
        )?;
        Ok(())
    }

    pub(crate) fn add_labels_to_pr<R>(
        &self,
        repo: &R,
        prnum: u64,
        labels: &[&str],
    ) -> anyhow::Result<()>
    where
        for<'a> R: RepositoryEndpoint<'a>,
    {
        self.0.post::<_, serde::de::IgnoredAny>(
            &format!("{}/issues/{prnum}/labels", repo.api_url().as_ref()),
            &labels,
        )?;
        Ok(())
    }
}

pub(crate) trait RepositoryEndpoint<'a> {
    type Url: AsRef<str>;

    fn api_url(&'a self) -> Self::Url;
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct Repository {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) full_name: String,
    pub(crate) url: String,
    pub(crate) private: bool,
    pub(crate) archived: bool,
    //pub(crate) html_url: String,
    //pub(crate) description: String,
    //pub(crate) ssh_url: String,
    //pub(crate) topics: Vec<String>,
    // owner?
}

impl<'a> RepositoryEndpoint<'a> for Repository {
    type Url = &'a str;

    fn api_url(&'a self) -> &'a str {
        &self.url
    }
}

impl<'a> RepositoryEndpoint<'a> for GHRepo {
    type Url = String;

    fn api_url(&'a self) -> String {
        self.api_url()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct CreatePullRequest<'a> {
    pub(crate) title: Cow<'a, str>,
    pub(crate) head: Cow<'a, str>,
    pub(crate) base: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) body: Option<Cow<'a, str>>,
    pub(crate) maintainer_can_modify: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct PullRequest {
    pub(crate) url: String,
    pub(crate) html_url: String,
    pub(crate) number: u64,
    //pub(crate) title: String,
    //#[serde(default)]
    //pub(crate) body: Option<String>,
    //labels
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct LabelInfo {
    name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct CreateLabel<'a> {
    pub(crate) name: Cow<'a, str>,
    pub(crate) color: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<Cow<'a, str>>,
}
