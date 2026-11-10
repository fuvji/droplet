//! Adaptive local timing for the macro modules.
//!
//! This is deliberately a small, deterministic "brain": it learns the user's
//! recent activation cadence per module and smoothly biases waits toward the
//! responsive end of the configured range when actions are coming quickly.
//! It never reads game memory, never changes a configured bound, and never
//! sends anything outside the process. All state is atomic so macro threads
//! can sample it without allocating or taking a lock.
//!
//! An optional `combat.profile` can provide a learned timing prior from the
//! offline combat lab. It is only a bias: the configured Min/Max always wins.

use std::sync::atomic::{AtomicI32, AtomicI64, AtomicU32, Ordering};
use std::sync::OnceLock;

use crate::input;

const SLOTS: usize = 16;

struct Stats {
    last: [AtomicI64; SLOTS],
    gap: [AtomicI64; SLOTS],
    count: [AtomicU32; SLOTS],
}

impl Stats {
    fn new() -> Stats {
        Stats {
            last: std::array::from_fn(|_| AtomicI64::new(0)),
            gap: std::array::from_fn(|_| AtomicI64::new(0)),
            count: std::array::from_fn(|_| AtomicU32::new(0)),
        }
    }
}

fn stats() -> &'static Stats {
    static STATS: OnceLock<Stats> = OnceLock::new();
    STATS.get_or_init(Stats::new)
}

/// Learned preferred milliseconds per module. Zero means "no profile entry".
static LEARNED: OnceLock<[AtomicI32; SLOTS]> = OnceLock::new();

fn learned_slots() -> &'static [AtomicI32; SLOTS] {
    LEARNED.get_or_init(|| std::array::from_fn(|_| AtomicI32::new(0)))
}

fn known_slot(module: &str) -> Option<usize> {
    Some(match module {
        "vD" => 0,
        "v1" => 1,
        "v2" => 2,
        "v3" => 3,
        "v4" => 4,
        "v5" => 5,
        "v6" => 6,
        "v7" => 7,
        "v8" => 8,
        "vA" => 9,
        "vW" => 10,
        "vC" | "wtap" => 11,
        "pearl" => 12,
        "totem" => 13,
        "shield" => 14,
        _ => return None,
    })
}

fn slot(module: &str) -> usize {
    known_slot(module).unwrap_or(15)
}

/// Parse the compact profile format. Unknown modules, comments, malformed
/// lines, and out-of-range values are ignored; the runtime clamps again to the
/// selected Min/Max before injection.
fn parse_profile(text: &str) -> Vec<(usize, i32)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let Some((name, value)) = line.split_once('=') else {
            continue;
        };
        let Some(i) = known_slot(name.trim()) else {
            continue;
        };
        let Ok(value) = value.trim().parse::<i32>() else {
            continue;
        };
        if (1..=1000).contains(&value) {
            out.push((i, value));
        }
    }
    out
}

/// Load `%LOCALAPPDATA%\Hydrogen\combat.profile` once at startup. Missing or
/// invalid files leave the existing local cadence brain unchanged.
pub fn load_profile() {
    let path = crate::config::data_dir_path().join("combat.profile");
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    {
        let slots = learned_slots();
        for (i, value) in parse_profile(&text) {
            slots[i].store(value, Ordering::Relaxed);
        }
    }
}

fn learned(i: usize) -> Option<i32> {
    let value = learned_slots()[i].load(Ordering::Relaxed);
    if value > 0 { Some(value) } else { None }
}

/// Called once per macro activation. The EWMA keeps a bad frame or a long
/// pause from making the next timing decision jump around.
pub fn on_trigger(module: &str) {
    let i = slot(module);
    let now = input::now();
    let s = stats();
    let last = s.last[i].swap(now, Ordering::Relaxed);
    s.count[i].fetch_add(1, Ordering::Relaxed);
    if last <= 0 || now <= last {
        return;
    }
    let gap_ms = ((now - last) as f64 * 1000.0 / input::qpf() as f64) as i64;
    let gap = gap_ms.clamp(70, 900);
    let old = s.gap[i].load(Ordering::Relaxed);
    let next = if old <= 0 { gap } else { (old * 3 + gap) / 4 };
    s.gap[i].store(next, Ordering::Relaxed);
}

/// 0.0 = fastest / most responsive, 1.0 = calmest. A quick cadence biases
/// toward the low end; deliberate, spaced activations use more of the range.
fn bias(i: usize) -> f32 {
    let gap = stats().gap[i].load(Ordering::Relaxed);
    if gap <= 0 {
        return 0.42;
    }
    let t = ((gap as f32 - 80.0) / 480.0).clamp(0.0, 1.0);
    (0.18 + t * 0.62).clamp(0.18, 0.92)
}

/// Pick a timing inside the configured bounds. With Smart AI disabled this is
/// the original uniform random range, so the feature is strictly optional.
/// A learned profile is a 65% prior blended with the local cadence bias; the
/// selected Min/Max still clamps the result.
pub fn pick(enabled: bool, module: &str, min: i32, max: i32) -> i32 {
    let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
    if lo == hi {
        return lo;
    }
    if !enabled {
        return input::rand_range(lo, hi);
    }
    let i = slot(module);
    let adaptive = lo as f32 + (hi - lo) as f32 * bias(i);
    let base = match learned(i) {
        Some(prior) => {
            let blended = prior as f32 * 0.65 + adaptive * 0.35;
            blended.clamp(lo as f32, hi as f32)
        }
        None => adaptive,
    };
    // A small, bounded humanising jitter plus a rotating phase avoids a
    // perfectly periodic pattern while staying inside the user's limits.
    let phase = (stats().count[i].load(Ordering::Relaxed).wrapping_mul(37) % 7) as i32 - 3;
    let value = base.round() as i32 + input::rand_range(-2, 2) + phase / 3;
    value.clamp(lo, hi)
}

#[inline]
pub fn wait(enabled: bool, module: &str, min: i32, max: i32) -> i32 {
    let ms = pick(enabled, module, min, max);
    input::sleep_ms(ms as f64);
    ms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_parser_accepts_known_modules_and_ignores_bad_values() {
        let parsed = parse_profile(
            "# generated profile\nv6=50\nvC = 105\nv5=0\nv5=2000\nbogus=44\nvA=oops\n",
        );
        assert_eq!(parsed, vec![(6, 50), (11, 105)]);
    }

    #[test]
    fn profile_parser_maps_the_wtap_alias_to_the_same_slot() {
        assert_eq!(known_slot("vC"), known_slot("wtap"));
        assert_eq!(known_slot("v5"), Some(5));
        assert_eq!(known_slot("nope"), None);
    }
}
