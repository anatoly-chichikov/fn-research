# AGENTS.md

Agent instructions for research automation.

## Commands

| Command | Action |
|---------|--------|
| `rs [provider] [processor] <topic>` | New research |
| `frk` | Fork existing research |
| `st` | List sessions with PDF paths |
| `pdf <topic>` | Generate PDF |
| `tst` | Run tests in Docker |

If message starts with these commands — it's a research operation, not development.

---

## Inputs

Screenshots can accompany any command or message. Treat them as additional inputs, extract key points into the current context, and ask clarifying questions if anything is unclear.

---

## rs

New research. Dialog first, then launch.

### Inline parameters

rs supports optional inline provider and processor:
- `rs valyu standard <topic>` — skip provider/processor questions
- `rs parallel ultra <topic>` — skip provider/processor questions
- `rs xai social <topic>` — skip provider/processor questions
- `rs <topic>` — ask provider/processor as before

parse rules:
  - first token after `rs`: check if it matches a known provider (parallel, valyu, xai, all)
  - if provider matched, next token: check if it matches a valid processor for that provider
  - everything after recognized tokens = topic
  - if first token is NOT a provider — entire string is the topic, ask provider/processor interactively
  - provider without processor: use provider, ask processor only
  - valid combinations:
    - parallel: pro, pro-fast, ultra, ultra-fast, ultra2x, ultra2x-fast, ultra4x, ultra4x-fast, ultra8x, ultra8x-fast
    - valyu: fast, standard, heavy
    - xai: social, full
    - all: inherits parallel processors (runs parallel then valyu)

### Interactive questions

ask language Which language for the result?
  - English
  - Russian
  - Spanish
  - Greek

rule Always normalize the language value to its full English form: "English", "Russian", "Spanish", "Greek". Regardless of user input (ru, рус, en, eng, ελληνικά, etc.) — always store and pass the canonical English name.

After language selected — switch all follow-up questions to that language.

ask provider (skip if provided inline) Which data provider?
  - parallel (cheaper and faster)
  - valyu (more thorough, premium result)
  - xai (social sources)
  - all (run parallel then valyu)

ask processor (skip if provided inline) What compute level?
  - parallel:
    - pro
    - ultra
    - ultra8x
  - valyu:
    - fast
    - standard
    - heavy
  - xai:
    - social (X search + social web only)
    - full (X search + unrestricted web)

ask topic — minimum 3 questions, up to 5 (in selected language):
  - Scope: narrow vs broad? specific case or general overview?
  - Audience: who reads this? expert level or beginner-friendly?
  - Focus: which aspects matter most? what angle interests you?
  - Constraints: exclude anything? time period? geography? industry?
  - Output: actionable insights? comparison? trends? recommendations?

rule Always ask at least 3 topic questions before launching
rule User can say "enough" to skip remaining questions
do Surface blind spots and non-obvious angles through questions

If user asks for two runs at once:
- ask the same questions twice, explicitly for run A then run B (no multi-select)
- collect params for run A and run B (topic, language, provider, processor when applicable)
- start two docker containers (different names) and report both

brief format:
- title (max 120 chars) + "Research:" + flat numbered list
- dense single-line items, all details via dash/colon in one line
- no bold, no subheadings, no nested lists, no extra sections
- language = result language

title rules:
- the title is the most important part — it appears in the PDF, folder name, and session list
- capture the angle or thesis, not just the subject area — what exactly is being investigated and why it matters
- write as if naming an essay or magazine longread, not a textbook chapter or Wikipedia article
- noun phrase, not a question or full sentence
- no colons, no subtitles, no " — " separators
- banned words: deep dive, overview, best practices, comprehensive, framework, guide, exploration
- no "X vs Y" comparisons in titles — reframe as the underlying question
- bad titles (too generic, textbook-like, machine-sounding):
  - "How Rust Works" → sounds like a tutorial for beginners
  - "Enterprise data platform architecture" → textbook chapter heading
  - "Marriage in modern European society" → sociology coursework title
  - "Goal Framework Anti-Patterns Deep Dive" → buzzword pile-up
  - "AI-Augmented Engineering Career Frameworks — 2026 Best Practices" → subtitle + buzzwords
  - "Data needs and quality requirements of agentic search platforms" → academic paper abstract
- good titles (specific angle, human voice, convey what's interesting):
  - "Ownership as a contract with the compiler" → specific mechanism + metaphor
  - "The data platform that is not just a set of tools" → captures the real tension
  - "European marriage without obligation" → the actual phenomenon being researched
  - "The career ladder that ignores AI agents" → specific gap, not generic topic
  - "The data appetite of agentic search platforms" → active voice, specific angle
- ask yourself: would a thoughtful person use this phrase to describe their research to a colleague over coffee?

run docker build -t research .
rule {topic} must be the crafted title from the brief, never the raw user input
rule The query argument MUST contain real newlines, not literal \n escapes — use $'...' (ANSI-C quoting) for the query argument so that \n is interpreted as actual newline characters by bash
run docker run -d --name "research-{timestamp}-{slug}" \
    -v "$(pwd)/output:/app/output" \
    -e PARALLEL_API_KEY -e VALYU_API_KEY -e GEMINI_API_KEY -e REPORT_FOR -e XAI_API_KEY \
    research run "{topic}" $'Язык ответа: {language}.\n\n{brief}' --processor "{processor}" --language "{language}" --provider "{provider}"

If two runs requested, run the command twice with different {timestamp}-{slug} values.

notify container_name
notify estimated_time
notify pdf_path — exact full path (no wildcards!), build after getting session ID

Example output:
```
Container: research-20241221-1430-clojure-pdf
Processor: ultra8x
Provider: parallel
Time: 5-50 min
PDF: /Users/chichikov/Work/research/output/2025-12-21_clojure-pdf_3e4fc072/2025-12-21_clojure-pdf-parallel.pdf [NOT READY]
```

---

## frk

Fork existing research. Dialog first, then launch.

ask source Which research to fork?
  - If user already specifies, resolve by meaning (folder name, topic, or hint)
  - Otherwise show last 3-5 from output/ (by mtime) as numbered list
  - Accept selection by number, folder name, or description ("the one about quantum computing")

ask type What should we do?
  1. re-brief - adjust the inputs
  2. deep-dive - go deeper into part of the result

re-brief flow:
  - Load brief from selected session:
    - prefer output/<session>/brief-*.md
    - fallback: output/<session>/input-*.md
  - Ask user for changes, keep brief format from rs
  - Show diff preview (original brief vs updated brief) before launch, ask confirmation
  - Run new research with updated brief (provider/processor default to original unless user overrides)
  - Save as new session in output/

deep-dive flow:
  - Load output/<session>/response-*.json into context
  - Summarize sections/fragments and ask which to deepen (3-5 options)
  - Ask clarifying questions about desired focus
  - Generate new brief from chosen fragment
  - Show diff preview (original brief vs new brief) before launch, ask confirmation
  - Run new research with updated brief (provider/processor default to original unless user overrides)
  - Save as new session in output/

If multiple providers exist in a session, ask which one to fork.

---

## st

List sessions. For each:
- Topic
- Status (in_progress % / completed)
- Full PDF path
- If file missing — mark [NOT READY]

Example:
```
[HITL startups] in_progress (67%)
  PDF: /Users/chichikov/Work/research/output/2025-12-21_hitl-startups_3e4fc072/2025-12-21_hitl-startups-parallel.pdf [NOT READY]

[AI coding assistants] completed
  PDF: /Users/chichikov/Work/research/output/2025-12-20_ai-coding_8f2a1b3c/2025-12-20_ai-coding-parallel.pdf
```

---

## pdf

Generate PDF by topic. Find session by meaning (not by ID).

run docker run --rm \
    -v "$(pwd)/output:/app/output" \
    -e REPORT_FOR \
    research generate {id}

notify pdf_path (full path)

---

## tst

Run tests in Docker container.

run docker build -t research-test -f Dockerfile.test .
run docker run --rm research-test :unit
run docker run --rm -v "$(pwd)/tmp_cache:/app/tmp_cache" research-test :integration

notify test results (pass/fail count)

---

## Parallel processors

| Name | Time | Use case |
|------|------|----------|
| `pro` | 2-10 min | Default, exploratory |
| `ultra` | 5-25 min | Multi-source deep |
| `ultra2x` | 5-50 min | Complex deep research |
| `ultra4x` | 5-90 min | Very complex |
| `ultra8x` | 5 min-2 h | Maximum depth |

Tip: add `-fast` for speed (pro-fast, ultra-fast)

## Valyu models

| Name | Use case |
|------|----------|
| `fast` | Quickest, lighter research |
| `standard` | Balanced depth and speed |
| `heavy` | Deeper, more thorough |

---

## Environment

```bash
export PARALLEL_API_KEY="..."
export VALYU_API_KEY="..."
export XAI_API_KEY="..."
export GEMINI_API_KEY="..."
export REPORT_FOR="..."
```
