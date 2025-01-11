use serde::Serialize;
use thiserror::Error;

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

impl std::str::FromStr for Language {
    type Err = ParseLanguageError;

    fn from_str(s: &str) -> Result<Language, ParseLanguageError> {
        match s.to_ascii_lowercase().as_str() {
            "python" | "py" => Ok(Language::Python),
            "rust" | "rs" => Ok(Language::Rust),
            _ => Err(ParseLanguageError),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("invalid/unknown language name")]
pub(crate) struct ParseLanguageError;

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("Python", Language::Python)]
    #[case("python", Language::Python)]
    #[case("PYTHON", Language::Python)]
    #[case("pYThOn", Language::Python)]
    #[case("py", Language::Python)]
    #[case("Py", Language::Python)]
    #[case("pY", Language::Python)]
    #[case("PY", Language::Python)]
    #[case("Rust", Language::Rust)]
    #[case("RUST", Language::Rust)]
    #[case("rust", Language::Rust)]
    #[case("RusT", Language::Rust)]
    #[case("rs", Language::Rust)]
    #[case("Rs", Language::Rust)]
    #[case("rS", Language::Rust)]
    #[case("RS", Language::Rust)]
    fn test_parse_language(#[case] s: &str, #[case] lang: Language) {
        assert_eq!(s.parse::<Language>().unwrap(), lang);
    }
}
