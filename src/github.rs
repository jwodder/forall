use crate::http_util::{get_next_link, StatusError};
use anyhow::Context;
use ghrepo::GHRepo;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt;
use ureq::{Agent, AgentBuilder, Response};
use url::Url;

static USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("CARGO_PKG_REPOSITORY"),
    ")",
);

static API_ENDPOINT: &str = "https://api.github.com";

#[derive(Clone, Debug)]
pub(crate) struct GitHub {
    client: Agent,
}

impl GitHub {
    pub(crate) fn new(token: &str) -> GitHub {
        let auth = format!("Bearer {token}");
        let client = AgentBuilder::new()
            .user_agent(USER_AGENT)
            .https_only(true)
            .middleware(move |req: ureq::Request, next: ureq::MiddlewareNext<'_>| {
                next.handle(
                    req.set("Authorization", &auth)
                        .set("Accept", "application/vnd.github+json"),
                )
            })
            .build();
        GitHub { client }
    }

    pub(crate) fn authed() -> anyhow::Result<GitHub> {
        let token = gh_token::get().context("Failed to retrieve GitHub token")?;
        Ok(GitHub::new(&token))
    }

    fn raw_request<T: Serialize>(
        &self,
        method: &str,
        url: Url,
        payload: Option<T>,
    ) -> anyhow::Result<Response> {
        //log::debug!("{} {}", method, url);
        let req = self.client.request_url(method, &url);
        let r = if let Some(p) = payload {
            req.send_json(p)
        } else {
            req.call()
        };
        match r {
            Ok(r) => Ok(r),
            Err(ureq::Error::Status(_, r)) => Err(StatusError::for_response(method, r).into()),
            Err(e) => Err(e).with_context(|| format!("Failed to make {method} request to {url}")),
        }
    }

    fn request<T: Serialize, U: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        payload: Option<T>,
    ) -> anyhow::Result<U> {
        self.raw_request(method, mkurl(path)?, payload)
            .and_then(|r| {
                r.into_json::<U>()
                    .with_context(|| format!("Failed to deserialize response from {path}"))
            })
    }

    fn get<T: DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        self.request::<(), T>("GET", path, None)
    }

    fn post<T: Serialize, U: DeserializeOwned>(&self, path: &str, body: T) -> anyhow::Result<U> {
        self.request::<T, U>("POST", path, Some(body))
    }

    /*
        fn put<T: Serialize, U: DeserializeOwned>(&self, path: &str, body: T) -> anyhow::Result<U> {
            self.request::<T, U>("PUT", path, Some(body))
        }
    */

    fn paginate<T: DeserializeOwned>(&self, path: &str) -> anyhow::Result<Vec<T>> {
        let mut items = Vec::new();
        let mut url = mkurl(path)?;
        loop {
            let r = self.raw_request::<()>("GET", url.clone(), None)?;
            let next_url = get_next_link(&r);
            items.extend(
                r.into_json::<Vec<T>>()
                    .with_context(|| format!("Failed to deserialize response from {path}"))?,
            );
            match next_url {
                Some(u) => url = u,
                None => return Ok(items),
            }
        }
    }

    pub(crate) fn get_repository<R>(&self, repo: &R) -> anyhow::Result<Repository>
    where
        for<'a> R: RepositoryEndpoint<'a>,
    {
        self.get(&repo.api_url().to_string())
    }

    pub(crate) fn create_pull_request<R>(
        &self,
        repo: &R,
        pr: CreatePullRequest<'_>,
    ) -> anyhow::Result<PullRequest>
    where
        for<'a> R: RepositoryEndpoint<'a>,
    {
        self.post(&format!("{}/pulls", repo.api_url()), pr)
    }

    pub(crate) fn get_label_names<R>(&self, repo: &R) -> anyhow::Result<Vec<String>>
    where
        for<'a> R: RepositoryEndpoint<'a>,
    {
        self.paginate::<LabelInfo>(&format!("{}/labels", repo.api_url()))
            .map(|v| v.into_iter().map(|li| li.name).collect::<Vec<_>>())
    }

    pub(crate) fn create_label<R>(&self, repo: &R, label: CreateLabel<'_>) -> anyhow::Result<()>
    where
        for<'a> R: RepositoryEndpoint<'a>,
    {
        self.post::<_, serde::de::IgnoredAny>(&format!("{}/labels", repo.api_url()), label)
            .map(|_| ())
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
        self.post::<_, serde::de::IgnoredAny>(
            &format!("{}/issues/{prnum}/labels", repo.api_url()),
            labels,
        )
        .map(|_| ())
    }
}

fn mkurl(path: &str) -> anyhow::Result<Url> {
    Url::parse(API_ENDPOINT)
        .context("Failed to construct a Url for the GitHub API endpoint")?
        .join(path)
        .with_context(|| format!("Failed to construct a GitHub API URL with path {path:?}"))
}

pub(crate) trait RepositoryEndpoint<'a> {
    type Url: fmt::Display;

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
