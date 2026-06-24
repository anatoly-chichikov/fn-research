//! Sample brief — a laconic, intent-first fixture used by `--mock`, the tour,
//! and the no-API-key fallback. Mirrors the real generator's terse voice and
//! the language of the topic (Russian topic → Russian brief).

use std::time::Duration;

use crate::brief::generator::{BriefError, BriefGenerator, BriefRequest};
use crate::brief::{TuiBrief, TuiRoot, TuiSub};

/// Build a sample brief for `topic` in `language` (Russian or English fixture).
pub fn sample(topic: &str, language: &str) -> TuiBrief {
    if language.eq_ignore_ascii_case("russian") {
        build(topic, "Russian", &RU)
    } else {
        build(topic, "English", &EN)
    }
}

/// A laconic fixture: intent + three angles, each a title, a one-line why, and three subs.
struct Fixture {
    intent: &'static str,
    roots: [Root; 3],
}

struct Root {
    title: &'static str,
    note: &'static str,
    knobs: (u8, u8, u8),
    subs: [(&'static str, (u8, u8, u8)); 3],
}

fn build(topic: &str, language: &str, fx: &Fixture) -> TuiBrief {
    let roots = std::array::from_fn(|i| {
        let r = &fx.roots[i];
        TuiRoot {
            title: r.title.to_string(),
            note: r.note.to_string(),
            depth: r.knobs.0,
            novelty: r.knobs.1,
            applied: r.knobs.2,
            subs: std::array::from_fn(|j| TuiSub {
                title: r.subs[j].0.to_string(),
                depth: r.subs[j].1 .0,
                novelty: r.subs[j].1 .1,
                applied: r.subs[j].1 .2,
            }),
        }
    });
    TuiBrief {
        topic: topic.to_string(),
        language: language.to_string(),
        intent: fx.intent.to_string(),
        topics: roots,
    }
}

const EN: Fixture = Fixture {
    intent: "You want a sober map of where quantum computing really is — what runs today, what doesn't, and what would change your mind.",
    roots: [
        Root {
            title: "Where is quantum computing actually today?",
            note: "Headlines vs. what the machines really run.",
            knobs: (4, 3, 3),
            subs: [
                ("Which qubit modality is winning, and why?", (4, 3, 4)),
                ("What did the latest \"advantage\" result really prove?", (4, 4, 3)),
                ("What is the real bottleneck right now?", (3, 2, 4)),
            ],
        },
        Root {
            title: "Is fault tolerance close, or still a wall?",
            note: "Error correction is the only path — how steep?",
            knobs: (5, 4, 2),
            subs: [
                ("Surface vs. LDPC vs. cat qubits — which scales?", (5, 4, 2)),
                ("What did Google's Willow actually show?", (4, 4, 3)),
                ("A sober timeline to breaking real crypto?", (4, 3, 3)),
            ],
        },
        Root {
            title: "What could we run that classical can't?",
            note: "Which speedups survive the full pipeline.",
            knobs: (3, 3, 5),
            subs: [
                ("Which algorithms keep their speedup end-to-end?", (4, 2, 5)),
                ("Why is simulation the most credible early use?", (3, 3, 5)),
                ("Who is funding it, and what is the bet?", (2, 4, 5)),
            ],
        },
    ],
};

const RU: Fixture = Fixture {
    intent: "Хочешь трезвую карту: где квантовые вычисления реально сейчас — что работает, что нет и что изменит твоё мнение.",
    roots: [
        Root {
            title: "Где квантовые вычисления сейчас на деле?",
            note: "Заголовки против того, что машины реально считают.",
            knobs: (4, 3, 3),
            subs: [
                ("Какая платформа кубитов выигрывает и почему?", (4, 3, 4)),
                ("Что реально доказал последний результат «преимущества»?", (4, 4, 3)),
                ("Где сейчас настоящее узкое место?", (3, 2, 4)),
            ],
        },
        Root {
            title: "Отказоустойчивость близко или это стена?",
            note: "Коррекция ошибок — единственный путь; насколько крутой?",
            knobs: (5, 4, 2),
            subs: [
                ("Surface, LDPC или cat-кубиты — что масштабируется?", (5, 4, 2)),
                ("Что на самом деле показал Willow от Google?", (4, 4, 3)),
                ("Трезвый срок до взлома реальной криптографии?", (4, 3, 3)),
            ],
        },
        Root {
            title: "Что запустим, чего классика не может?",
            note: "Какие ускорения выживают на полном пайплайне.",
            knobs: (3, 3, 5),
            subs: [
                ("Какие алгоритмы держат ускорение от и до?", (4, 2, 5)),
                ("Почему симуляция — самый правдоподобный кейс?", (3, 3, 5)),
                ("Кто финансирует и на что ставит?", (2, 4, 5)),
            ],
        },
    ],
};

/// Re-phrase each root's framing to its (new) depth level, keeping structure and
/// knobs, in the prior's language — so a `--mock` regenerate visibly shifts.
pub fn retune(prior: &TuiBrief) -> TuiBrief {
    let ru = prior.language.eq_ignore_ascii_case("russian");
    let variants = if ru { &RU_VARIANTS } else { &EN_VARIANTS };
    let mut next = prior.clone();
    for (i, root) in next.topics.iter_mut().enumerate() {
        let bucket = match root.depth {
            1 | 2 => 0,
            3 => 1,
            _ => 2,
        };
        let (title, note) = variants[i][bucket];
        root.title = title.to_string();
        root.note = note.to_string();
    }
    next
}

type Variants = [[(&'static str, &'static str); 3]; 3];

const EN_VARIANTS: Variants = [
    [
        (
            "What is quantum computing in plain terms?",
            "A readable orientation, no press gloss.",
        ),
        (
            "Where is quantum computing actually today?",
            "Headlines vs. what the machines run.",
        ),
        (
            "At the device level, what has each platform proven?",
            "Straight to the papers and the numbers.",
        ),
    ],
    [
        (
            "Is fault tolerance basically solved?",
            "The big-picture read on error correction.",
        ),
        (
            "Is fault tolerance close, or still a wall?",
            "How steep is the overhead curve?",
        ),
        (
            "How is the logical-qubit overhead curve bending?",
            "Down into surface vs. LDPC vs. cat.",
        ),
    ],
    [
        (
            "What is a quantum computer good for?",
            "Plausible payoffs, minus the hype.",
        ),
        (
            "What could we run that classical can't?",
            "Speedups that survive the pipeline.",
        ),
        (
            "Which workloads clear the full-pipeline bar?",
            "Decision-grade end-to-end economics.",
        ),
    ],
];

const RU_VARIANTS: Variants = [
    [
        (
            "Что такое квантовые вычисления простыми словами?",
            "Понятная ориентировка без глянца.",
        ),
        (
            "Где квантовые вычисления сейчас на деле?",
            "Заголовки против того, что машины считают.",
        ),
        (
            "Что каждая платформа доказала на уровне железа?",
            "Сразу к статьям и цифрам.",
        ),
    ],
    [
        (
            "Отказоустойчивость в целом решена?",
            "Картина по коррекции ошибок.",
        ),
        (
            "Отказоустойчивость близко или это стена?",
            "Насколько крутая кривая накладных?",
        ),
        (
            "Как гнётся кривая накладных на логический кубит?",
            "Вглубь: surface, LDPC, cat.",
        ),
    ],
    [
        (
            "Для чего вообще нужен квантовый компьютер?",
            "Правдоподобные выгоды без хайпа.",
        ),
        (
            "Что запустим, чего классика не может?",
            "Ускорения, выживающие на пайплайне.",
        ),
        (
            "Какие задачи проходят полный пайплайн?",
            "Экономика от и до для решения.",
        ),
    ],
];

/// Mock generator with a tunable artificial delay.
pub struct MockBriefGenerator {
    delay: Duration,
}

impl MockBriefGenerator {
    /// Construct with a chosen delay. Tour wants 0; live mock wants ~600ms.
    pub fn new(delay: Duration) -> Self {
        Self { delay }
    }
}

impl Default for MockBriefGenerator {
    fn default() -> Self {
        Self::new(Duration::from_millis(600))
    }
}

impl BriefGenerator for MockBriefGenerator {
    fn generate(&self, req: &BriefRequest) -> Result<TuiBrief, BriefError> {
        if !self.delay.is_zero() {
            std::thread::sleep(self.delay);
        }
        match &req.prior {
            Some(prior) => Ok(retune(prior)),
            None => Ok(sample(&req.topic, &req.language)),
        }
    }
}
