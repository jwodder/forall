use mime::{Mime, JSON};
use serde_json::{to_string_pretty, value::Value};
use thiserror::Error;
use ureq::Response;
use url::Url;

/// Error raised for a 4xx or 5xx HTTP response that includes the response body
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("{method} request to {url} returned {status}")]
pub(crate) struct StatusError {
    url: String,
    method: String,
    status: String,
    body: Option<String>,
}

impl StatusError {
    pub(crate) fn for_response(method: &str, r: Response) -> StatusError {
        let url = r.get_url().to_string();
        let status = format!("{} {}", r.status(), r.status_text());
        // If the response body is JSON, pretty-print it.
        let body = if is_json_response(&r) {
            r.into_json::<Value>().ok().map(|v| {
                to_string_pretty(&v).expect("Re-JSONifying a JSON response should not fail")
            })
        } else {
            r.into_string().ok()
        };
        StatusError {
            url,
            status,
            body,
            method: method.to_string(),
        }
    }

    pub(crate) fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }
}

/// Returns `true` iff the response's Content-Type header indicates the body is
/// JSON
pub(crate) fn is_json_response(r: &Response) -> bool {
    r.header("Content-Type")
        .and_then(|v| v.parse::<Mime>().ok())
        .is_some_and(|ct| {
            ct.type_() == "application" && (ct.subtype() == "json" || ct.suffix() == Some(JSON))
        })
}

/// Return the `rel="next"` URL, if any, from the response's "Link" header
pub(crate) fn get_next_link(r: &Response) -> Option<Url> {
    let header_value = r.header("Link")?;
    parse_link_header::parse_with_rel(header_value)
        .ok()?
        .get("next")
        .map(|link| link.uri.clone())
}
