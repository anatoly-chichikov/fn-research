# fn research

Every deep research tool works the same way: you type a topic, it searches, you get 30 pages of vaguely relevant text. Nobody asked what you actually wanted to know.

This tool builds a research brief *with you* before searching anything.

## How the brief works

You type `rs <topic>` and the agent builds a structured brief in two phases — root topics, then sub-topics — with you in the loop.

**Phase 1** — the agent generates 3 root topics from your request. Each one is a specific investigative angle, not a generic heading. Each comes with three properties — depth, novelty, applied — on a 1–5 scale. These aren't just labels: you can change them. Say "make the first one deeper" or "less theoretical" — the agent will reformulate the topic to match. Same with the topics themselves: replace, rewrite, swap — the brief doesn't lock until you say so.

The agent also writes a summary of how it understood your intent — "I think you want X because Y, I'm assuming Z about your context, I'm leaving W out of scope." If it got you wrong, say so and it regenerates from scratch.

**Phase 2** — 3 sub-topics per root topic. Same interactive refinement. You adjust, it reformulates. No overlap, no generic filler.

The final brief is a tight 3×3 structure that goes to the research engine. Properties and reasoning get stripped — the engine sees only focused, self-contained research questions. But you saw the full reasoning before approving.

The difference: instead of searching "quantum computing" and hoping for the best, you're sending a brief that says exactly which 9 angles to investigate and at what depth.

```
rs parallel ultra Quantum computing
```

> **Summary:** I'm reading this as curiosity about where quantum computing actually
> stands — not a textbook intro, but an honest look at what works, what doesn't yet,
> and what matters for someone tracking the field seriously. I'm assuming you have
> technical literacy but aren't building quantum circuits yourself. I'm leaving out
> quantum cryptography and post-quantum encryption as a separate domain, and not
> covering the physics curriculum (Hilbert spaces, Dirac notation) unless it serves
> a practical point.
>
> ---
>
> **1. The qubit gap between laboratory demonstrations and useful computation —
>    what current hardware can and cannot do, and where the real bottlenecks are**
>    This is the ground truth question: how many qubits actually work together
>    reliably, what error rates look like today, and why "1000 qubits" headlines
>    don't mean what they seem to mean.
>
>    **depth:** `4` · **novelty:** `3` · **applied:** `3`
>
> ---
>
> **2. Error correction as the unsolved engineering problem that determines whether
>    quantum computing scales — current approaches, resource overhead, and realistic
>    timelines**
>    Everyone agrees fault-tolerant quantum needs error correction. The interesting
>    part is how expensive it is: thousands of physical qubits per logical qubit.
>    This topic digs into whether that cost is coming down fast enough.
>
>    **depth:** `5` · **novelty:** `4` · **applied:** `2`
>
> ---
>
> **3. The algorithms that would actually justify building a quantum computer —
>    which problems have proven quantum speedup and which industries would feel
>    it first**
>    Shor's algorithm broke RSA in theory, Grover gives a square-root speedup for
>    search — but what's the realistic portfolio of problems where quantum beats
>    classical in practice, not just asymptotically?
>
>    **depth:** `3` · **novelty:** `3` · **applied:** `5`

## The output

I like Hokusai, so the PDFs have a woodblock print vibe — generated covers, muted colors, wave motifs. Inside it's clean: good typography, table of contents, styled citations. Nothing fancy, just not ugly.

Both engines researching "AI transformation of academic research":
- [Parallel example](./examples/parallel-ai-academic-research.pdf) — 21 pages, strategic focus
- [Valyu example](./examples/valyu-ai-academic-research.pdf) — 25 pages, data-rich

## Quick start

This is not a CLI tool. You use it through an AI coding agent:

1. Open the project folder in Claude Code, Codex, Cursor, or Junie
2. Type `rs <topic>` — the agent handles the rest
3. Answer the agent's questions (language, depth, focus angles)
4. Get a PDF in `./output/`

The agent reads `AGENTS.md` for its instructions. You focus on what to research — it handles Docker, APIs, and file generation.

```
rs parallel ultra Rust ownership model

# The agent will:
# - Ask about language, confirm provider/processor
# - Build a 3×3 brief interactively with you
# - Launch a Docker container
# - Generate a PDF report in ./output/
```

## Forking research

Already have a research run and want to go further? `frk` lets you:

- **Re-brief** — adjust the original brief and run again with different angles or depth
- **Deep-dive** — pick a specific section from the results and investigate it deeper

Both modes show you a diff of the original vs. updated brief before launching. The fork creates a new session — the original stays intact.

## Commands

| Command | What it does |
|---------|-------------|
| `rs [provider] [processor] <topic>` | New research run |
| `frk` | Fork existing research |
| `st` | List all sessions with status and PDF paths |
| `pdf <topic>` | Regenerate PDF for a session |
| `tst` | Run tests in Docker |

## Providers

| | Parallel | Valyu | XAI |
|---|----------|-------|-----|
| **Sources** | Open internet | Open internet + academic & proprietary | Web + X/social |
| **Strength** | Strategic synthesis | Data-rich analysis, better citations | Social signals and discourse |
| **Best for** | Business decisions, implementation planning | Academic research, evidence gathering | Social coverage, trending topics |
| **Processors** | pro, ultra, ultra2x, ultra4x, ultra8x | fast, standard, heavy | social, full |
| **Speed** | 10–40 min | 30–90 min | 5–20 min |

Use `all` as provider (e.g. `rs all ultra <topic>`) to run Parallel then Valyu in the same session.

## Setup

### Requirements

- Docker
- An AI coding agent (Claude Code, Codex, Cursor, Junie)

### Environment variables

```bash
export PARALLEL_API_KEY="..."   # Parallel AI access
export VALYU_API_KEY="..."      # Valyu access
export XAI_API_KEY="..."        # XAI access
export GEMINI_API_KEY="..."     # Optional: cover image generation
export REPORT_FOR="..."         # Optional: name in report attribution
```

### Python dependencies

```bash
uv sync
```

### Testing

```bash
# Via agent:
tst

# Manual:
docker build -t research-test -f Dockerfile.test .
docker run --rm research-test
docker run --rm -v "$PWD/tmp_cache:/app/tmp_cache" -e REPORT_FOR research-test -- --ignored --test-threads=1
```

## License

Apache 2.0
