# Cover Image Prompt — Structure Audit

Audit of the modular ukiyo-e cover image generation prompt (`resources/cover/*.edn`).

## Critical Contradictions

### 1. "bold outlines" vs "delicate fine lines"

Two files give the model mutually exclusive instructions about line weight.

**`quality_requirements.edn`** — `authenticity_markers` requires:
```
"bold_confident_outlines"
```

**`style_definition.edn`** — `line_work` requires:
```
:weight "delicate_fine_lines_throughout"
:quality "elegant_thin_consistent"
:uniformity "same_line_thickness_for_distant_and_close_subjects"
```

**`generation_process.edn`** — step 7:
```
"apply_consistent_thin_line_work_throughout"
```

The model receives both "bold outlines" and "delicate fine thin lines" simultaneously. Gemini must pick one, and the result is unpredictable.

**Fix:** Decide on one approach. For authentic ukiyo-e (Hokusai), "bold confident outlines" is historically accurate. Update `style_definition.edn` line_work to match, or change `quality_requirements.edn` to "consistent_confident_outlines" as a compromise.

### 2. Six "primary" colors with a limit of five

**`color_palette.edn`** declares 6 colors under `:primary_colors`:
- faded_vermilion, pine_green, prussian_blue, aged_cream, deep_indigo, moss_green

But `:usage_rule` says:
```
"use_maximum_5_colors_per_image_for_authentic_limited_palette"
```

The model doesn't know which of the 6 "primary" colors to drop. Two greens (pine_green, moss_green) in the primary tier is redundant.

**Fix:** Remove one green from primary (moss_green → secondary), or raise limit to 6.

### 3. Generation process step order is lost

**`generation_process.edn`** is an EDN map (unordered). Keys `:step_1` through `:step_9` have no guaranteed order in EDN. After JSON serialization via jsonista, steps arrive in arbitrary order (e.g., 2, 4, 3, 8, 9, 1, 6, 5, 7). The sequential process loses its meaning.

**Fix:** Use a vector of strings instead of a map with step keys:
```clojure
{:generation_process
 {:steps
  ["analyze_topic_identify_core_concept_and_metaphor"
   "select_location_matching_topic_mood"
   "choose_2_to_4_metaphor_elements_visualizing_concept"
   ...]}}
```

## Structural Problems

### 4. Prompt sent as JSON blob, not natural language

`generator.clj` assembles the prompt as `{:topic value, :marketing_image image}` and serializes it to JSON. The model receives nested JSON with snake_case keys like `"edge_behavior"`, `"rendering_quality"`, `"dominant_ratio"`.

Gemini image generation is optimized for natural language prompts, not for parsing deeply nested JSON specifications. Instruction effectiveness may be significantly lower than a well-crafted text prompt.

**Fix:** Add a text serializer that converts the EDN specification into a natural language prompt before sending to the API.

### 5. Full libraries sent with every request

Every API call includes ALL 38 locations and ALL 60+ metaphor elements. The `topic_to_visual_mapping` already provides specific recommendations per domain — only the relevant subset needs to be sent.

Impact: higher token cost, slower generation, potential signal dilution.

**Fix:** Use `topic_to_visual_mapping` to filter libraries down to relevant entries before assembly.

### 6. Redundant repetition without priority differentiation

"No text" appears 3 times across files. "Full bleed" appears 4 times. Repetition can help LLMs, but without explicit priority levels, all instructions appear equally important. Critical hard requirements (no text, no frame) are not distinguished from soft aesthetic preferences (gentle patina, slight yellowing).

**Fix:** Add a `:priority` field ("hard" / "soft") to requirements, or restructure into `must` vs `should` sections.

## Factual Errors

### 7. Incorrect romanization of 簡素

**`wabi_sabi.edn`**: `{:name_jp "簡素", :id "kanzo"}`.

簡素 is read as かんそ (**kanso**), not "kanzo". The 'z' is wrong.

**Fix:** Change `:id "kanzo"` to `:id "kanso"`.

### 8. Inconsistent data structure in locations_library

`atmospheric_conditions` entries lack the `:name_jp` field that all `built_environments` and `natural_landscapes` entries have. Structural inconsistency.

**Fix:** Add `:name_jp` to atmospheric conditions (e.g., `{:name_jp "夜明け", :id "dawn", ...}`).

## Logic Issues

### 9. Surface treatment conflicts with metaphor element visuals

`surface_treatment.edn` requires all readable surfaces to show texture only, no text or glyphs.

But `metaphor_elements_library.edn` describes:
- `map`: `"detailed_map_routes_compass_rose_coastlines_blank"` — compass rose and routes are content on a surface
- `diagram`: `"technical_illustration_cross_sections_abstract_marks"` — marks on a surface
- `scroll`: `"long_scroll_partially_unrolled_aged_paper_ink_wash"` — ink wash patterns

The model receives contradictory signals: "all surfaces blank" vs "map with routes and compass rose."

**Fix:** Update metaphor visual descriptions to be consistent with surface treatment (e.g., map → `"blank_parchment_with_fold_marks_compass_nearby"`).

### 10. No color selection step in generation_process

The 9 steps cover: topic, location, metaphors, atmosphere, composition, edges, lines, surfaces, verification. But there is no explicit step for selecting the 5-color subset from the palette. Step 9 says "verify limited palette" but the selection step is missing.

**Fix:** Add a step between composition and line work: `"select_5_colors_from_palette_matching_mood_and_atmosphere"`.

### 11. Wabi-sabi "simplicity" vs prompt complexity

The kanso principle demands "minimal_elements_with_intentional_omission." The prompt itself contains 14 modular files, 38 locations, 60+ metaphors, 12 domain mappings. The specification volume may overwhelm the simplicity directive.

This is a meta-level tension, not a bug — but it suggests the prompt could be more effective if pared down.
