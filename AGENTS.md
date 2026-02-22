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

### Structured brief generation

The brief is built in two phases: root topics first, then sub-topics. The goal is always to focus the research — even a broad subject starts with a specific angle.

#### Topic properties

Every topic (root and sub) carries three properties on a 1–5 scale. These are shown to the user for transparency but NOT included in the final brief text. They act as sliders — the user can move them, and the topic text must be reformulated to match.

| Property | Scale | 1 | 3 | 5 |
|----------|-------|---|---|---|
| depth | 1–5 | surface overview, trends, landscape | mechanisms, how things work, key details | primary sources, internals, technical depth |
| novelty | 1–5 | well-established consensus, textbook knowledge | active development, recent shifts | bleeding edge, few sources, mostly hypotheses |
| applied | 1–5 | pure understanding, "why is it so" | mix of theory and application | concrete decisions, recommendations, "what to do" |

rule Properties are displayed to the user for each topic so they can adjust direction
rule Properties do NOT go into the brief — they are a reasoning aid for the dialog only
rule When user changes a property value, the topic text MUST be reformulated to reflect the new level — changing a number without rewriting the topic is meaningless
rule The model should consider the full 1–5 range when generating topics — use it to calibrate how the topic is phrased
rule Property labels must be translated to the selected language (e.g. Russian: глубина, новизна, прикладность; Spanish: profundidad, novedad, aplicabilidad; Greek: βάθος, καινοτομία, εφαρμοσιμότητα)

#### Phase 1 — root topics (max 3)

After the user gives their short input:

1. Think deeply about the user's input — what they really want to learn, what directions exist, what's obvious and what's not
2. Write a summary (2-4 sentences) showing how you understood the user's request: what you think they actually want to learn, what assumptions you're making about their context and level, what you're deliberately leaving out of scope
3. Generate exactly 3 root topics (in selected language)
4. Present everything as a compact table (see format below)
5. Ask for confirmation

rule The summary comes BEFORE the table — it frames the whole research direction
rule The summary should show genuine reasoning, not generic filler — the user should feel "ah, it actually understood what I need"
rule The 3 topics together must cover the user's intent without overlap
rule Each topic should represent a distinct angle or dimension of the subject
rule Always aim to focus the research — even "I want to learn about X" becomes 3 specific investigative angles
do Surface non-obvious angles and blind spots the user might not have considered
do Use properties to make your reasoning transparent

Output format (Phase 1):

```
**Summary:** [2-4 sentences — this is how I understood your request. What I think you
actually want to learn and why. What I'm assuming about your context and level. What I'm
deliberately leaving out of scope. The user reads this FIRST and corrects if the model
misunderstood — before even looking at the topics below.]

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

rule Summary must NOT mention, preview, or rephrase any of the 3 topics — it exists on a different level
rule Summary is about the USER'S INTENT: "I understood that you want X because Y, I'm assuming Z about your context, and I'm leaving out W"
rule If the user corrects the summary ("нет, я имел в виду другое"), regenerate topics from scratch based on the corrected understanding
rule Each topic block: bold title → reason on next line → properties on separate line in inline code (`backticks`)
rule Property labels in output MUST use the selected language — e.g. for Russian: **глубина:** `3` · **новизна:** `2` · **прикладность:** `4` (the examples above use English only as a template)
rule Keep it compact — no extra blank lines between blocks

Topic text rules:
- The topic text goes into the brief AS IS — the research engine will read it, not the user
- It must be a self-contained research question: clear, specific, understandable without context
- Bad: "Ownership как контракт программиста с компилятором" — too poetic, the engine won't know what to search
- Good: "Система владения (ownership) в Rust — как компилятор управляет памятью без сборщика мусора, зачем нужен borrow checker и что он даёт на практике"
- The topic text can be 1-2 full sentences — it's not a title, it's a research instruction

Reason rules:
- The reason is for the USER, not the engine — it explains WHY this topic was chosen
- It should be accessible: if the topic mentions technical concepts, the reason should briefly explain them in plain language
- Bad: "Covers data races, use-after-free, and specific bug classes that ownership prevents" — this just extends the topic with more jargon
- Good: "Это ключевая фишка Rust — вместо того чтобы программа сама следила за памятью (как в C++, где легко ошибиться), компилятор проверяет всё заранее. Если не понять этот механизм, остальное в Rust не сложится."
- The reason should answer: "why should I care about this topic?" and "what will I learn?"

The user can:
- approve all 3 ("да", "ок", "погнали")
- ask to adjust any topic ("второй попроще", "первый в другую сторону")
- ask to replace a topic entirely ("убери третий, добавь что-то про X")
- adjust properties ("сделай глубже", "менее теоретический")

rule Iterate until the user confirms all 3 root topics
rule After each adjustment, reprint summary + full table with updated values

#### Phase 2 — sub-topics (3 per root)

Once all 3 root topics are confirmed:

1. For each root topic, generate exactly 3 sub-topics
2. Sub-topics expand the root topic deeper, considering the overall research focus established in Phase 1
3. Present as one table per root topic (see format below)
4. Ask for confirmation

rule Sub-topics must not overlap with each other or with other root topics
rule Sub-topics inherit the general direction of their root but can vary in properties
do Consider the overall focus when generating sub-topics — they should work as a coherent research plan

Output format (Phase 2) — grouped by root:

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

The same topic text rules and reason rules from Phase 1 apply to sub-topics.

The user can adjust sub-topics the same way as root topics. Iterate until confirmed.

#### Brief assembly

Once all topics and sub-topics are confirmed, assemble the brief:

brief format:
- title (max 120 chars) + "Research:" + tab-indented plain text
- 3 root topics (no indent), each with 3 sub-topics (one tab indent)
- no numbering, no bullets — plain text lines, tabs for nesting
- dense single-line items, details via dash/colon
- no bold, no subheadings, no extra sections
- properties and reasoning are NOT included — only topic/sub-topic text
- language = result language

Example brief structure (tabs shown as →):
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

rule Use real tab characters (\t) for indentation, not spaces
rule Root topics start at column 0, sub-topics start with one tab

#### Dual runs

If user asks for two runs at once:
- ask the same questions twice, explicitly for run A then run B (no multi-select)
- collect params for run A and run B (topic, language, provider, processor when applicable)
- each run gets its own structured brief generation (Phase 1 + Phase 2)
- start two docker containers (different names) and report both

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
  - Ask user for changes using structured brief format (3 root topics × 3 sub-topics from rs)
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
