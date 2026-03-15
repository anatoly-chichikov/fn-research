use std::fmt;
use std::str::FromStr;

/// Research data provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    /// Parallel.ai provider.
    Parallel,
    /// Valyu.ai provider.
    Valyu,
    /// X.ai provider.
    Xai,
}

/// Object with display label.
pub trait Labeled {
    /// Return display name for JSON serialization.
    fn label(&self) -> &str;
}

impl Labeled for Provider {
    fn label(&self) -> &str {
        match self {
            Provider::Parallel => "parallel.ai",
            Provider::Valyu => "valyu.ai",
            Provider::Xai => "x.ai",
        }
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Provider::Parallel => write!(f, "parallel"),
            Provider::Valyu => write!(f, "valyu"),
            Provider::Xai => write!(f, "xai"),
        }
    }
}

impl FromStr for Provider {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "parallel" | "parallel.ai" => Ok(Provider::Parallel),
            "valyu" | "valyu.ai" => Ok(Provider::Valyu),
            "xai" | "x.ai" | "xai.ai" => Ok(Provider::Xai),
            other => Err(format!("{} is not a valid provider", other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_provider_parses_canonical_names() {
        let parallel: Provider = "parallel".parse().unwrap();
        let valyu: Provider = "valyu".parse().unwrap();
        let xai: Provider = "xai".parse().unwrap();
        let ok = parallel == Provider::Parallel && valyu == Provider::Valyu && xai == Provider::Xai;
        assert!(ok, "Provider did not parse canonical names");
    }

    #[test]
    fn the_provider_parses_legacy_parallel_ai() {
        let result: Provider = "parallel.ai".parse().unwrap();
        assert_eq!(
            Provider::Parallel,
            result,
            "Provider did not parse parallel.ai"
        );
    }

    #[test]
    fn the_provider_parses_legacy_valyu_ai() {
        let result: Provider = "valyu.ai".parse().unwrap();
        assert_eq!(Provider::Valyu, result, "Provider did not parse valyu.ai");
    }

    #[test]
    fn the_provider_parses_legacy_x_ai() {
        let result: Provider = "x.ai".parse().unwrap();
        assert_eq!(Provider::Xai, result, "Provider did not parse x.ai");
    }

    #[test]
    fn the_provider_parses_legacy_xai_ai() {
        let result: Provider = "xai.ai".parse().unwrap();
        assert_eq!(Provider::Xai, result, "Provider did not parse xai.ai");
    }

    #[test]
    fn the_provider_rejects_unknown_name() {
        let result = Provider::from_str("unknown");
        assert!(result.is_err(), "Provider accepted unknown name");
    }

    #[test]
    fn the_provider_displays_canonical_name() {
        let parallel = Provider::Parallel.to_string();
        let valyu = Provider::Valyu.to_string();
        let xai = Provider::Xai.to_string();
        let ok = parallel == "parallel" && valyu == "valyu" && xai == "xai";
        assert!(ok, "Provider did not display canonical name");
    }

    #[test]
    fn the_provider_roundtrips_through_display() {
        let original = Provider::Parallel;
        let text = original.to_string();
        let parsed: Provider = text.parse().unwrap();
        assert_eq!(
            original, parsed,
            "Provider did not roundtrip through display"
        );
    }

    #[test]
    fn the_provider_labels_parallel_as_parallel_ai() {
        assert_eq!(
            "parallel.ai",
            Provider::Parallel.label(),
            "Provider label was not parallel.ai"
        );
    }

    #[test]
    fn the_provider_labels_valyu_as_valyu_ai() {
        assert_eq!(
            "valyu.ai",
            Provider::Valyu.label(),
            "Provider label was not valyu.ai"
        );
    }

    #[test]
    fn the_provider_labels_xai_as_x_ai() {
        assert_eq!("x.ai", Provider::Xai.label(), "Provider label was not x.ai");
    }
}
