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

rule If message starts with these commands — it's a research operation, not development

---

## Inputs

rule Screenshots can accompany any command or message
do Extract key points from screenshots into the current context
do Ask clarifying questions if anything in screenshots is unclear

---

## rs

New research. Dialog first, then launch.

### Inline parameters

parse rs [provider] [processor] <topic>
  - first token after `rs`: check if known provider (parallel, valyu, xai, all)
  - if provider matched, next token: check if valid processor for that provider
  - remaining tokens = topic
  - if first token is NOT a provider — entire string is topic, ask provider/processor interactively
  - provider without processor: use provider, ask processor only

enum providers
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

rule Always normalize language to canonical English form: "English", "Russian", "Spanish", "Greek" — regardless of user input (ru, рус, en, eng, ελληνικά, etc.)
rule After language selected — switch all follow-up questions to that language

ask provider (skip if inline) Which data provider?
  - parallel (cheaper and faster)
  - valyu (more thorough, premium result)
  - xai (social sources)
  - all (run parallel then valyu)

ask processor (skip if inline) What compute level?
  - parallel: pro, ultra, ultra8x
  - valyu: fast, standard, heavy
  - xai: social (X search + social web only), full (X search + unrestricted web)

### Structured brief generation

Brief is built in two phases: root topics → sub-topics. Goal: focus the research — even a broad subject starts with a specific angle.

#### Topic properties

Every topic (root and sub) carries three properties on a 1–5 scale.

| Property | 1 | 3 | 5 |
|----------|---|---|---|
| depth | surface overview, trends, landscape | mechanisms, how things work, key details | primary sources, internals, technical depth |
| novelty | established consensus, textbook knowledge | active development, recent shifts | bleeding edge, few sources, mostly hypotheses |
| applied | pure understanding, "why is it so" | mix of theory and application | concrete decisions, recommendations, "what to do" |

rule Properties are displayed to the user for transparency — NOT included in the final brief
rule Properties act as sliders — user can move them, topic text MUST be reformulated to match
rule Changing a number without rewriting the topic text is meaningless
rule Use full 1–5 range to calibrate how topics are phrased
rule Property labels must be translated to selected language (Russian: глубина, новизна, прикладность; Spanish: profundidad, novedad, aplicabilidad; Greek: βάθος, καινοτομία, εφαρμοσιμότητα)

#### Phase 1 — root topics (max 3)

step 1 Think deeply about user's input — what they really want, what directions exist, what's obvious and what's not
step 2 Write summary (2-4 sentences): how you understood the request, assumptions about context and level, what's deliberately out of scope
step 3 Generate exactly 3 root topics in selected language
step 4 Present as compact table (see format below)
step 5 Ask for confirmation

rule Summary comes BEFORE topics — frames the whole research direction
rule Summary shows genuine reasoning, not generic filler — user should feel "ah, it actually understood what I need"
rule Summary must NOT mention, preview, or rephrase any of the 3 topics — it exists on a different level
rule Summary is about USER'S INTENT: "I understood that you want X because Y, I'm assuming Z about your context, and I'm leaving out W"
rule If user corrects summary ("нет, я имел в виду другое") — regenerate topics from scratch based on corrected understanding
rule 3 topics together must cover user's intent without overlap
rule Each topic = distinct angle or dimension of the subject
rule Always focus the research — even "I want to learn about X" becomes 3 specific investigative angles

do Surface non-obvious angles and blind spots the user might not have considered
do Use properties to make reasoning transparent

format phase1
```
**Summary:** [2-4 sentences — how I understood your request. What I think you
actually want to learn and why. Assumptions about your context and level. What
I'm deliberately leaving out of scope.]

**1. [topic text as it goes into the brief]**
[1-2 sentences: why this angle, what it covers]
**depth:** `3` · **novelty:** `2` · **applied:** `4`

**2. [topic text as it goes into the brief]**
[1-2 sentences: why this angle, what it covers]
**depth:** `4` · **novelty:** `1` · **applied:** `3`

**3. [topic text as it goes into the brief]**
[1-2 sentences: why this angle, what it covers]
**depth:** `2` · **novelty:** `5` · **applied:** `1`
```

rule Each topic block: bold title → reason on next line → properties on separate line in inline code (`backticks`)
rule Property labels in output MUST use selected language — e.g. Russian: **глубина:** `3` · **новизна:** `2` · **прикладность:** `4` (examples above use English only as template)
rule Keep compact — no extra blank lines between blocks

rule topic_text Goes into brief AS IS — research engine reads it, not user
rule topic_text Must be self-contained research question: clear, specific, understandable without context
rule topic_text Can be 1-2 full sentences — it's a research instruction, not a title

example topic_text bad "Ownership как контракт программиста с компилятором" — too poetic, engine won't know what to search
example topic_text good "Система владения (ownership) в Rust — как компилятор управляет памятью без сборщика мусора, зачем нужен borrow checker и что он даёт на практике"

rule reason Is for the USER, not the engine — explains WHY this topic was chosen
rule reason Must be accessible: technical concepts explained in plain language
rule reason Must answer: "why should I care about this topic?" and "what will I learn?"

example reason bad "Covers data races, use-after-free, and specific bug classes that ownership prevents" — extends topic with more jargon
example reason good "Это ключевая фишка Rust — вместо того чтобы программа сама следила за памятью (как в C++, где легко ошибиться), компилятор проверяет всё заранее. Если не понять этот механизм, остальное в Rust не сложится."

when user approves ("да", "ок", "погнали") → proceed to Phase 2
when user adjusts topic ("второй попроще", "первый в другую сторону") → reformulate and reprint
when user replaces topic ("убери третий, добавь что-то про X") → replace and reprint
when user adjusts properties ("сделай глубже", "менее теоретический") → reformulate topic text and reprint

rule Iterate until user confirms all 3 root topics
rule After each adjustment, reprint summary + full table with updated values

#### Phase 2 — sub-topics (3 per root)

step 1 For each root topic, generate exactly 3 sub-topics
step 2 Sub-topics expand root deeper, considering overall research focus from Phase 1
step 3 Present as one table per root topic (see format below)
step 4 Ask for confirmation

rule Sub-topics must not overlap with each other or with other root topics
rule Sub-topics inherit root direction but can vary in properties
rule Same topic_text and reason rules from Phase 1 apply

do Consider overall focus — sub-topics should work as coherent research plan

format phase2
```
**1. [root topic text]**

**1.1. [sub-topic text]**
[why this matters]
**depth:** `3` · **novelty:** `2` · **applied:** `4`

**1.2. [sub-topic text]**
[why this matters]
**depth:** `4` · **novelty:** `3` · **applied:** `3`

**1.3. [sub-topic text]**
[why this matters]
**depth:** `2` · **novelty:** `1` · **applied:** `2`

**2. [root topic text]**
...
```

when user adjusts sub-topics → same interaction rules as Phase 1
rule Iterate until user confirms all sub-topics

#### Brief assembly

rule Assemble once all topics and sub-topics are confirmed

format brief
  - title (max 120 chars) + "Research:" + tab-indented plain text
  - 3 root topics at column 0, each with 3 sub-topics at one tab indent
  - no numbering, no bullets — plain text lines, tabs for nesting
  - dense single-line items, details via dash/colon
  - no bold, no subheadings, no extra sections
  - properties and reasoning NOT included — only topic/sub-topic text
  - language = result language

rule Use real tab characters (\t) for indentation, not spaces
rule Root topics at column 0, sub-topics at one tab

example brief (tabs shown as →)
```
Research:
[root topic 1 text]
→[sub-topic text]
→[sub-topic text]
→[sub-topic text]
[root topic 2 text]
→[sub-topic text]
→[sub-topic text]
→[sub-topic text]
[root topic 3 text]
→[sub-topic text]
→[sub-topic text]
→[sub-topic text]
```

#### Dual runs

when user asks for two runs at once
  - ask same questions twice, explicitly for run A then run B (no multi-select)
  - collect params for run A and run B (topic, language, provider, processor)
  - each run gets own structured brief generation (Phase 1 + Phase 2)
  - start two docker containers (different names) and report both

### Title

rule Title is most important — appears in PDF, folder name, session list
rule Capture angle or thesis, not just subject area — what exactly is being investigated and why it matters
rule Write as if naming an essay or magazine longread, not a textbook chapter or Wikipedia article
rule Noun phrase, not a question or full sentence
rule No colons, no subtitles, no " — " separators
rule {topic} must be the crafted title from the brief, never raw user input

ban deep dive, overview, best practices, comprehensive, framework, guide, exploration
ban "X vs Y" comparisons in titles — reframe as underlying question

example title bad "How Rust Works" — sounds like tutorial for beginners
example title bad "Enterprise data platform architecture" — textbook chapter heading
example title bad "Marriage in modern European society" — sociology coursework title
example title bad "Goal Framework Anti-Patterns Deep Dive" — buzzword pile-up
example title bad "AI-Augmented Engineering Career Frameworks — 2026 Best Practices" — subtitle + buzzwords
example title bad "Data needs and quality requirements of agentic search platforms" — academic paper abstract

example title good "Ownership as a contract with the compiler" — specific mechanism + metaphor
example title good "The data platform that is not just a set of tools" — captures real tension
example title good "European marriage without obligation" — the actual phenomenon being researched
example title good "The career ladder that ignores AI agents" — specific gap, not generic topic
example title good "The data appetite of agentic search platforms" — active voice, specific angle

test Would a thoughtful person use this phrase describing their research to a colleague over coffee?

### Launch

run docker build -t research .

rule The query argument MUST contain real newlines, not literal \n escapes — use $'...' (ANSI-C quoting)

run docker run -d --name "research-{timestamp}-{slug}" \
    -v "$(pwd)/output:/app/output" \
    -e PARALLEL_API_KEY -e VALYU_API_KEY -e GEMINI_API_KEY -e REPORT_FOR -e XAI_API_KEY \
    research run "{topic}" $'Язык ответа: {language}.\n\n{brief}' --processor "{processor}" --language "{language}" --provider "{provider}"

when two runs → run command twice with different {timestamp}-{slug} values

notify container_name
notify estimated_time
notify pdf_path — exact full path (no wildcards!), build after getting session ID

example output
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
  1. re-brief — adjust the inputs
  2. deep-dive — go deeper into part of the result

when re-brief
  step 1 Load brief from selected session: prefer output/<session>/brief-*.md, fallback output/<session>/input-*.md
  step 2 Ask user for changes using structured brief format (3 root topics × 3 sub-topics from rs)
  step 3 Show diff preview (original brief vs updated brief) before launch, ask confirmation
  step 4 Run new research with updated brief (provider/processor default to original unless user overrides)
  step 5 Save as new session in output/

when deep-dive
  step 1 Load output/<session>/response-*.json into context
  step 2 Summarize sections/fragments and ask which to deepen (3-5 options)
  step 3 Ask clarifying questions about desired focus
  step 4 Generate new brief from chosen fragment
  step 5 Show diff preview (original brief vs new brief) before launch, ask confirmation
  step 6 Run new research with updated brief (provider/processor default to original unless user overrides)
  step 7 Save as new session in output/

rule If multiple providers exist in a session, ask which one to fork

---

## st

List sessions. For each:

notify topic
notify status (in_progress % / completed)
notify pdf_path (full path)
rule If file missing — mark [NOT READY]

example output
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
