use indenter::indented;
use mime::{Mime, JSON};
use serde_json::{to_string_pretty, value::Value};
use std::fmt::{self, Write};
use ureq::Response;

/// Error raised for a 4xx or 5xx HTTP response that includes the response body
#[derive(Clone, Debug, Eq, PartialEq)]
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
}

impl fmt::Display for StatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} request to {} returned {}",
            self.method, self.url, self.status
        )?;
        if let Some(text) = &self.body {
            write!(indented(f).with_str("    "), "\n\n{text}\n")?;
        }
        Ok(())
    }
}

impl std::error::Error for StatusError {}

/// Returns `true` iff the response's Content-Type header indicates the body is
/// JSON
pub(crate) fn is_json_response(r: &Response) -> bool {
    r.header("Content-Type")
        .and_then(|v| v.parse::<Mime>().ok())
        .is_some_and(|ct| {
            ct.type_() == "application" && (ct.subtype() == "json" || ct.suffix() == Some(JSON))
        })
}
