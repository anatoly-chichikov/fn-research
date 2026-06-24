//! Provider/processor catalogues displayed on the input screen.
//!
//! Source of truth is `research_domain` — the strings here must round-trip
//! through `Provider::FromStr` / `processor::resolve`. Annotations are
//! display-only.

/// Provider chip — what the user sees and what the headless binary parses.
pub struct ProviderChip {
    pub id: &'static str,
    pub label: &'static str,
    pub note: &'static str,
}

/// Processor chip — depends on the active provider.
pub struct ProcessorChip {
    pub id: &'static str,
    pub label: &'static str,
    pub note: &'static str,
}

/// Providers shown on the input screen, in display order.
pub const PROVIDERS: &[ProviderChip] = &[
    ProviderChip {
        id: "parallel",
        label: "parallel",
        note: "open internet · strategic synthesis",
    },
    ProviderChip {
        id: "valyu",
        label: "valyu",
        note: "academic + proprietary · data-rich",
    },
    ProviderChip {
        id: "xai",
        label: "xai",
        note: "web + X · social signals",
    },
];

/// Processors per provider, in display order.
pub fn processors_for(provider_id: &str) -> &'static [ProcessorChip] {
    match provider_id {
        "parallel" => &[
            ProcessorChip {
                id: "pro",
                label: "pro",
                note: "~10 min · light",
            },
            ProcessorChip {
                id: "ultra",
                label: "ultra",
                note: "~20 min · default",
            },
            ProcessorChip {
                id: "ultra2x",
                label: "ultra2x",
                note: "~30 min",
            },
            ProcessorChip {
                id: "ultra4x",
                label: "ultra4x",
                note: "~40 min",
            },
            ProcessorChip {
                id: "ultra8x",
                label: "ultra8x",
                note: "~60 min · heaviest",
            },
        ],
        "valyu" => &[
            ProcessorChip {
                id: "fast",
                label: "fast",
                note: "~30 min",
            },
            ProcessorChip {
                id: "standard",
                label: "standard",
                note: "~60 min · default",
            },
            ProcessorChip {
                id: "heavy",
                label: "heavy",
                note: "~90 min",
            },
        ],
        "xai" => &[
            ProcessorChip {
                id: "social",
                label: "social",
                note: "~5 min · social only",
            },
            ProcessorChip {
                id: "full",
                label: "full",
                note: "~20 min · web + social",
            },
        ],
        _ => &[],
    }
}

/// Default processor index for a freshly-selected provider.
pub fn default_processor_idx(provider_id: &str) -> usize {
    match provider_id {
        "parallel" => 1, // ultra
        "valyu" => 1,    // standard
        _ => 0,
    }
}
