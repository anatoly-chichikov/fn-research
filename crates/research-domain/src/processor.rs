use std::fmt;
use std::str::FromStr;

use crate::provider::Provider;

/// Parallel.ai processing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParallelMode {
    /// Standard research.
    Pro,
    /// Fast standard research.
    ProFast,
    /// Deep multi-source research.
    Ultra,
    /// Fast deep research.
    UltraFast,
    /// Complex deep research.
    Ultra2x,
    /// Fast complex research.
    Ultra2xFast,
    /// Very complex research.
    Ultra4x,
    /// Fast very complex research.
    Ultra4xFast,
    /// Maximum depth research.
    Ultra8x,
    /// Fast maximum depth research.
    Ultra8xFast,
}

/// Valyu.ai processing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValyuMode {
    /// Quick lightweight research.
    Fast,
    /// Balanced depth and speed.
    Standard,
    /// Deeper research.
    Heavy,
    /// Exhaustive analysis.
    Max,
}

/// X.ai processing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XaiMode {
    /// Social sources only.
    Social,
    /// Unrestricted web search.
    Full,
}

/// Research processing mode bound to a provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Processor {
    /// Parallel.ai processor.
    Parallel(ParallelMode),
    /// Valyu.ai processor.
    Valyu(ValyuMode),
    /// X.ai processor.
    Xai(XaiMode),
}

impl fmt::Display for ParallelMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParallelMode::Pro => write!(f, "pro"),
            ParallelMode::ProFast => write!(f, "pro-fast"),
            ParallelMode::Ultra => write!(f, "ultra"),
            ParallelMode::UltraFast => write!(f, "ultra-fast"),
            ParallelMode::Ultra2x => write!(f, "ultra2x"),
            ParallelMode::Ultra2xFast => write!(f, "ultra2x-fast"),
            ParallelMode::Ultra4x => write!(f, "ultra4x"),
            ParallelMode::Ultra4xFast => write!(f, "ultra4x-fast"),
            ParallelMode::Ultra8x => write!(f, "ultra8x"),
            ParallelMode::Ultra8xFast => write!(f, "ultra8x-fast"),
        }
    }
}

impl FromStr for ParallelMode {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "pro" => Ok(ParallelMode::Pro),
            "pro-fast" => Ok(ParallelMode::ProFast),
            "ultra" => Ok(ParallelMode::Ultra),
            "ultra-fast" => Ok(ParallelMode::UltraFast),
            "ultra2x" => Ok(ParallelMode::Ultra2x),
            "ultra2x-fast" => Ok(ParallelMode::Ultra2xFast),
            "ultra4x" => Ok(ParallelMode::Ultra4x),
            "ultra4x-fast" => Ok(ParallelMode::Ultra4xFast),
            "ultra8x" => Ok(ParallelMode::Ultra8x),
            "ultra8x-fast" => Ok(ParallelMode::Ultra8xFast),
            other => Err(format!("{} is not a valid parallel processor", other)),
        }
    }
}

impl fmt::Display for ValyuMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValyuMode::Fast => write!(f, "fast"),
            ValyuMode::Standard => write!(f, "standard"),
            ValyuMode::Heavy => write!(f, "heavy"),
            ValyuMode::Max => write!(f, "max"),
        }
    }
}

impl FromStr for ValyuMode {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "fast" => Ok(ValyuMode::Fast),
            "standard" => Ok(ValyuMode::Standard),
            "heavy" => Ok(ValyuMode::Heavy),
            "max" => Ok(ValyuMode::Max),
            other => Err(format!("{} is not a valid valyu processor", other)),
        }
    }
}

impl fmt::Display for XaiMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XaiMode::Social => write!(f, "social"),
            XaiMode::Full => write!(f, "full"),
        }
    }
}

impl FromStr for XaiMode {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "social" | "year" => Ok(XaiMode::Social),
            "full" => Ok(XaiMode::Full),
            other => Err(format!("{} is not a valid xai processor", other)),
        }
    }
}

impl fmt::Display for Processor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Processor::Parallel(m) => write!(f, "{}", m),
            Processor::Valyu(m) => write!(f, "{}", m),
            Processor::Xai(m) => write!(f, "{}", m),
        }
    }
}

/// Resolve processor string for given provider.
pub fn resolve(text: &str, provider: &Provider) -> Result<Processor, String> {
    match provider {
        Provider::Parallel => text.parse::<ParallelMode>().map(Processor::Parallel),
        Provider::Valyu => text.parse::<ValyuMode>().map(Processor::Valyu),
        Provider::Xai => text.parse::<XaiMode>().map(Processor::Xai),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_parallel_mode_parses_pro() {
        let result: ParallelMode = "pro".parse().unwrap();
        assert_eq!(ParallelMode::Pro, result, "ParallelMode did not parse pro");
    }

    #[test]
    fn the_parallel_mode_parses_ultra8x_fast() {
        let result: ParallelMode = "ultra8x-fast".parse().unwrap();
        assert_eq!(
            ParallelMode::Ultra8xFast,
            result,
            "ParallelMode did not parse ultra8x-fast"
        );
    }

    #[test]
    fn the_parallel_mode_roundtrips_through_display() {
        let original = ParallelMode::Ultra2xFast;
        let text = original.to_string();
        let parsed: ParallelMode = text.parse().unwrap();
        assert_eq!(original, parsed, "ParallelMode did not roundtrip");
    }

    #[test]
    fn the_valyu_mode_parses_all_variants() {
        let fast: ValyuMode = "fast".parse().unwrap();
        let standard: ValyuMode = "standard".parse().unwrap();
        let heavy: ValyuMode = "heavy".parse().unwrap();
        let max: ValyuMode = "max".parse().unwrap();
        let ok = fast == ValyuMode::Fast
            && standard == ValyuMode::Standard
            && heavy == ValyuMode::Heavy
            && max == ValyuMode::Max;
        assert!(ok, "ValyuMode did not parse all variants");
    }

    #[test]
    fn the_valyu_mode_roundtrips_through_display() {
        let original = ValyuMode::Heavy;
        let text = original.to_string();
        let parsed: ValyuMode = text.parse().unwrap();
        assert_eq!(original, parsed, "ValyuMode did not roundtrip");
    }

    #[test]
    fn the_xai_mode_parses_social() {
        let result: XaiMode = "social".parse().unwrap();
        assert_eq!(XaiMode::Social, result, "XaiMode did not parse social");
    }

    #[test]
    fn the_xai_mode_parses_year_as_social() {
        let result: XaiMode = "year".parse().unwrap();
        assert_eq!(
            XaiMode::Social,
            result,
            "XaiMode did not parse year as social"
        );
    }

    #[test]
    fn the_xai_mode_parses_full() {
        let result: XaiMode = "full".parse().unwrap();
        assert_eq!(XaiMode::Full, result, "XaiMode did not parse full");
    }

    #[test]
    fn the_xai_mode_roundtrips_through_display() {
        let original = XaiMode::Full;
        let text = original.to_string();
        let parsed: XaiMode = text.parse().unwrap();
        assert_eq!(original, parsed, "XaiMode did not roundtrip");
    }

    #[test]
    fn the_processor_displays_inner_mode() {
        let proc = Processor::Parallel(ParallelMode::Ultra);
        assert_eq!(
            "ultra",
            proc.to_string(),
            "Processor did not display inner mode"
        );
    }

    #[test]
    fn the_resolve_maps_parallel_pro() {
        let result = resolve("pro", &Provider::Parallel).unwrap();
        assert_eq!(
            Processor::Parallel(ParallelMode::Pro),
            result,
            "Resolve did not map parallel pro"
        );
    }

    #[test]
    fn the_resolve_maps_valyu_standard() {
        let result = resolve("standard", &Provider::Valyu).unwrap();
        assert_eq!(
            Processor::Valyu(ValyuMode::Standard),
            result,
            "Resolve did not map valyu standard"
        );
    }

    #[test]
    fn the_resolve_maps_xai_social() {
        let result = resolve("social", &Provider::Xai).unwrap();
        assert_eq!(
            Processor::Xai(XaiMode::Social),
            result,
            "Resolve did not map xai social"
        );
    }

    #[test]
    fn the_resolve_rejects_invalid_combination() {
        let result = resolve("fast", &Provider::Parallel);
        assert!(
            result.is_err(),
            "Resolve accepted invalid combination fast+parallel"
        );
    }

    #[test]
    fn the_resolve_rejects_lite_for_valyu() {
        let result = resolve("lite", &Provider::Valyu);
        assert!(result.is_err(), "Resolve accepted lite for valyu");
    }
}
