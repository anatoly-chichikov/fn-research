use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Return deterministic generator seeded with given value.
pub fn ids(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

/// Return deterministic token string from Unicode base and span.
fn token(rng: &mut StdRng, size: usize, base: u32, span: u32) -> String {
    let mut build = String::with_capacity(size);
    for _ in 0..size {
        let pick = rng.random_range(0..span);
        let code = base + pick;
        if let Some(ch) = char::from_u32(code) {
            build.push(ch);
        }
    }
    build
}

/// Return deterministic UUID-like string.
pub fn uuid(rng: &mut StdRng) -> String {
    let high: u64 = rng.random();
    let low: u64 = rng.random();
    let bytes = [high.to_be_bytes(), low.to_be_bytes()].concat();
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u16::from_be_bytes([bytes[4], bytes[5]]),
        u16::from_be_bytes([bytes[6], bytes[7]]),
        u16::from_be_bytes([bytes[8], bytes[9]]),
        u64::from_be_bytes([
            0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        ])
    )
}

/// Return deterministic cyrillic string.
pub fn cyrillic(rng: &mut StdRng, size: usize) -> String {
    token(rng, size, 1040, 32)
}

/// Return deterministic greek string.
pub fn greek(rng: &mut StdRng, size: usize) -> String {
    token(rng, size, 945, 24)
}

/// Return deterministic hiragana string.
pub fn hiragana(rng: &mut StdRng, size: usize) -> String {
    token(rng, size, 12354, 32)
}

/// Return deterministic armenian string.
pub fn armenian(rng: &mut StdRng, size: usize) -> String {
    token(rng, size, 1328, 32)
}

/// Return deterministic hebrew string.
pub fn hebrew(rng: &mut StdRng, size: usize) -> String {
    token(rng, size, 1424, 32)
}

/// Return deterministic arabic string.
pub fn arabic(rng: &mut StdRng, size: usize) -> String {
    token(rng, size, 1536, 32)
}

/// Return deterministic ascii string.
pub fn ascii(rng: &mut StdRng, size: usize) -> String {
    token(rng, size, 97, 26)
}

/// Return deterministic latin extended string.
pub fn latin(rng: &mut StdRng, size: usize) -> String {
    token(rng, size, 256, 64)
}

/// Return deterministic integer in range [0, bound).
pub fn digit(rng: &mut StdRng, bound: u32) -> u32 {
    rng.random_range(0..bound)
}

/// Return deterministic time string matching Clojure test pattern.
pub fn time(rng: &mut StdRng) -> String {
    let day = 1 + rng.random_range(0..8u32);
    let hour = 1 + rng.random_range(0..8u32);
    format!("2026-01-0{}T0{}:00:00", day, hour)
}
