# AGENTS.md

Build and contribution notes for this repo. The previous version of this file
described an interactive brief-building protocol that ran in the coding agent;
that flow now lives inside the binary itself (`research-tui`). What follows is
plain build/test instructions plus a few editing rules.

## Build

```bash
cargo build --release            # produces ./target/release/research
cargo run -- --mock              # launches the TUI without any API keys
docker build -t research .       # full image used to run real sessions
```

The Dockerfile entrypoint is unchanged: the image launches `research` and the
existing `research run …` headless subcommand is what actually performs the
research and renders the PDF.

## Test

```bash
cargo test                                          # unit + snapshot tests
cargo test -- --ignored --test-threads=1            # real-API tests (need keys)
docker build -t research-test -f Dockerfile.test .
docker run --rm research-test
docker run --rm -v "$PWD/tmp_cache:/app/tmp_cache" -e REPORT_FOR research-test \
    -- --ignored --test-threads=1
```

When updating snapshots after an intentional UI change:

```bash
RESEARCH_TUI_BLESS=1 cargo test -p research-tui
```

## Workspace layout

| Crate | Role |
|---|---|
| `research-domain` | Pure data types — `Brief`, `Question`, `ResearchSession`, `Provider`, `Processor`, `Report`. No network. |
| `research-api` | Provider clients (Parallel, Valyu, XAI). Implement `Researchable`. |
| `research-storage` | Repository pattern over `output/`, organizer that names folders/files. |
| `research-pdf` | HTML+CSS pipeline that turns a session's results into the styled PDF. |
| `research-image` | Cover-image generation (Gemini). |
| `research-tui` | The console interface. Owns the brief-building flow that used to live in this file. |
| `research-cli` | Binary `research`. With no args, launches the TUI. With a subcommand (`run`, `list`, etc.), runs headless. |

## Regenerating screenshots

```bash
cargo run --release -- tour --out screenshots
```

`research tour` walks every state with mock data, captures each frame from a
`TestBackend`, and rasterises it to PNG using the vendored fonts in
`crates/research-tui/assets/`. The same fixtures back the snapshot tests in
`crates/research-tui/tests/snapshots.rs`.

## Brief prompt

The system prompt that drives Gemini-side brief generation lives at
`crates/research-tui/src/brief/prompt.rs`. It's a single static string. The
title rules, banned phrasings, and depth/novelty/applied semantics that used
to live here moved into that file.

## Editing rules

- One monospace family in the TUI. One font size. Hierarchy comes from caps,
  colour, position — not from scale.
- `///` doc on every public item. Test names start with `the_` and read as
  sentences (`the_input_screen_cycles_provider`).
- Real-API tests are `#[ignore]`d so `cargo test` stays offline.
- Each screen module exposes `render(&State, Rect, &mut Frame)` and
  `handle_key(&mut State, KeyEvent) -> Action`. Keep state mutation in
  `handle_key`; render is a pure function of `State`.
- Properties and intent live only in `TuiBrief`. They get stripped in
  `to_domain()` before the brief is passed to the headless `run` subcommand.
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --all` before
  committing.
