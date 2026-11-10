//! Adaptive local timing for the macro modules.
//!
//! This is deliberately a small, deterministic "brain": it learns the user's
//! recent activation cadence per module and smoothly biases waits toward the
//! responsive end of the configured range when actions are coming quickly.
//! It never reads game memory, never changes a configured bound, and never
//! sends anything outside the process. All state is atomic so macro threads
//! can sample it without allocating or taking a lock.

use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};
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

fn slot(module: &str) -> usize {
    match module {
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
        "wtap" => 11,
        "pearl" => 12,
        "totem" => 13,
        "shield" => 14,
        _ => 15,
    }
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
pub fn pick(enabled: bool, module: &str, min: i32, max: i32) -> i32 {
    let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
    if lo == hi {
        return lo;
    }
    if !enabled {
        return input::rand_range(lo, hi);
    }
    let i = slot(module);
    let base = lo as f32 + (hi - lo) as f32 * bias(i);
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

