//! The macro modules, ported one-for-one from the AutoHotkey original.
//!
//! Every macro runs on its own short-lived thread (the original relied on
//! AutoHotkey spawning a thread per hotkey); waits are event driven where the
//! key state matters and QPC-precise where a fixed delay is needed.

#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;

use crate::config::Settings;
use crate::input::{self, Probe};
use crate::keys::{self, Btn, Ev, Key};
use crate::smart;

pub static CRYSTAL_BUSY: AtomicBool = AtomicBool::new(false);
static INV_BUSY: AtomicBool = AtomicBool::new(false);
static DASH_BUSY: AtomicBool = AtomicBool::new(false);
static ANCHOR_BUSY: AtomicBool = AtomicBool::new(false);
static WTAP_BUSY: AtomicBool = AtomicBool::new(false);
static SHIELD_BUSY: AtomicBool = AtomicBool::new(false);
static MACE_BUSY: AtomicBool = AtomicBool::new(false);
static BREACH_BUSY: AtomicBool = AtomicBool::new(false);
static POTION_BUSY: AtomicBool = AtomicBool::new(false);
static FASTUSE_BUSY: AtomicBool = AtomicBool::new(false);
static PEARL_BUSY: AtomicBool = AtomicBool::new(false);
static TOTEM_BUSY: AtomicBool = AtomicBool::new(false);
static AUTOUSE_BUSY: AtomicBool = AtomicBool::new(false);
/// QPC deadline for the next Sprint pulse. Zero means "pulse on the next tick".
static SPRINT_NEXT: AtomicI64 = AtomicI64::new(0);

/// RAII re-entrancy gate for one-shot macros. Holding the guard for the whole
/// sequence stops repeated input events from interleaving hotbar swaps, and
/// releasing it on every return/early-exit keeps a bad binding from wedging a
/// module until restart.
struct Busy(&'static AtomicBool);

impl Busy {
    fn enter(flag: &'static AtomicBool) -> Option<Busy> {
        if flag.swap(true, Ordering::SeqCst) {
            None
        } else {
            Some(Busy(flag))
        }
    }
}

impl Drop for Busy {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

/// Keeps physical W auto-repeat out of the game while a W-Tap has synthetically
/// released forward. Dropped on every exit path so normal W input is untouched
/// outside the short tap window.
struct WRepeatGuard;

impl WRepeatGuard {
    fn on() -> WRepeatGuard {
        input::set_w_repeat_suppressed(true);
        WRepeatGuard
    }
}

impl Drop for WRepeatGuard {
    fn drop(&mut self) {
        input::set_w_repeat_suppressed(false);
    }
}

/// Number of macro activations since launch (shown in the panel).
pub static RUNS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn spawn<F>(s: Arc<Settings>, ev: Ev, f: F)
where
    F: FnOnce(&Settings, Ev) + Send + 'static,
{
    RUNS.fetch_add(1, Ordering::Relaxed);
    let _ = std::thread::Builder::new()
        .stack_size(96 * 1024)
        .spawn(move || {
            if s.low_latency {
                input::boost_thread();
            }
            f(&s, ev)
        });
}

fn tap(binding: &str) {
    if let Some(k) = keys::parse(binding) {
        input::taps(&[k]);
    }
}

fn taps(bindings: &[&str]) {
    let mut ks: Vec<Key> = Vec::with_capacity(bindings.len());
    for b in bindings {
        if let Some(k) = keys::parse(b) {
            ks.push(k);
        }
    }
    input::taps(&ks);
}

fn tap_key(k: Option<Key>) {
    if let Some(k) = k {
        input::taps(&[k]);
    }
}

fn lc(s: &Settings) {
    input::jitter(s.ghost, s.ghost_off);
    input::click2(Btn::Left);
}

fn rc(s: &Settings) {
    input::jitter(s.ghost, s.ghost_off);
    input::click2(Btn::Right);
}

/// The AHK `RC()` shape: jitter, then right-down and right-up as separate
/// SendInput calls instead of one batch.
fn rc_exact(s: &Settings) {
    input::jitter(s.ghost, s.ghost_off);
    input::click_exact(Btn::Right);
}

fn wait_smart(s: &Settings, module: &str, min: i32, max: i32) {
    smart::wait(s.smart, module, min, max);
}

/// Anchor steps are aligned to a Minecraft tick. A 47 ms configured value is
/// legal in the UI, but sending the next hotbar/use action less than one tick
/// later is what makes an in-game cycle skip or double-fire. The clamp keeps the
/// configured 47-53 envelope while guaranteeing the game can observe each step.
fn anchor_wait_ms(s: &Settings) -> i32 {
    smart::pick(s.smart, "v6", s.anchor_min, s.anchor_max).max(input::TICK_MS)
}

fn anchor_wait(s: &Settings) {
    input::sleep_ms(anchor_wait_ms(s) as f64);
}

/// Select a hotbar slot and let the client observe the selection before the
/// following use input. Without this settle, a sequence can occasionally use the
/// previous item and look like it skipped a cycle.
fn tap_slot(k: Option<Key>) {
    tap_exact_key(k);
    input::sleep_ms(input::TICK_MS as f64);
}

fn tap_exact_key(k: Option<Key>) {
    if let Some(k) = k {
        input::tap_exact(k);
    }
}

#[cfg(test)]
fn hit(binding: &str, ev: Ev) -> bool {
    keys::ev(binding).map(|e| e == ev).unwrap_or(false)
}

/// One hotkey press starts exactly one macro action. Every module is a short
/// one-shot sequence; there are no latches, toggles, or background loops to
/// leave injecting input after the key is released.
fn fire(s: &Arc<Settings>, ev: Ev, module: &'static str, f: fn(&Settings, Ev)) {
    smart::on_trigger(module);
    spawn(s.clone(), ev, f);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Dash,
    Shield,
    Mace,
    Breach,
    Potion,
    Crystal,
    Anchor,
    FastUse,
    Pearl,
    Totem,
    AutoUse,
    WTap,
    HoverInv,
}

/// Pre-parsed event-to-action table. The old hook path scanned up to 24
/// binding strings and their hash cache on every event; this table reduces that
/// hot path to one indexed read. Rebuild it whenever settings change.
pub struct TriggerMap {
    keys: [Option<Action>; 256],
    mouse: [Option<Action>; 8],
}

impl Default for TriggerMap {
    fn default() -> Self {
        TriggerMap {
            keys: [None; 256],
            mouse: [None; 8],
        }
    }
}

impl TriggerMap {
    pub fn build(s: &Settings) -> TriggerMap {
        let mut map = TriggerMap::default();
        map.bind(&s.dash_key, Action::Dash);
        map.bind(&s.dash_key2, Action::Dash);
        map.bind(&s.shield_key, Action::Shield);
        map.bind(&s.shield_key2, Action::Shield);
        map.bind(&s.mace_key, Action::Mace);
        map.bind(&s.mace_key2, Action::Mace);
        map.bind(&s.breach_key, Action::Breach);
        map.bind(&s.breach_key2, Action::Breach);
        map.bind(&s.potion_key, Action::Potion);
        map.bind(&s.potion_key2, Action::Potion);
        map.bind(&s.crystal_key, Action::Crystal);
        map.bind(&s.crystal_key2, Action::Crystal);
        map.bind(&s.anchor_key, Action::Anchor);
        map.bind(&s.anchor_key2, Action::Anchor);
        map.bind(&s.fastuse_key, Action::FastUse);
        map.bind(&s.fastuse_key2, Action::FastUse);
        map.bind(&s.pearl_key, Action::Pearl);
        map.bind(&s.pearl_key2, Action::Pearl);
        map.bind(&s.totem_key, Action::Totem);
        map.bind(&s.totem_key2, Action::Totem);
        map.bind(&s.autouse_key, Action::AutoUse);
        map.bind(&s.autouse_key2, Action::AutoUse);
        map.bind(&s.wtap_key, Action::WTap);
        map.bind(&s.hover_inv_key, Action::HoverInv);
        map
    }

    fn bind(&mut self, binding: &str, action: Action) {
        let Some(ev) = keys::ev(binding) else { return; };
        match ev {
            Ev::K(vk) if (vk as usize) < self.keys.len() => {
                let slot = &mut self.keys[vk as usize];
                if slot.is_none() {
                    *slot = Some(action);
                }
            }
            Ev::M(btn) if (btn as usize) < self.mouse.len() => {
                let slot = &mut self.mouse[btn as usize];
                if slot.is_none() {
                    *slot = Some(action);
                }
            }
            _ => {}
        }
    }

    #[inline]
    pub fn get(&self, ev: Ev) -> Option<Action> {
        match ev {
            Ev::K(vk) if (vk as usize) < self.keys.len() => self.keys[vk as usize],
            Ev::M(btn) if (btn as usize) < self.mouse.len() => self.mouse[btn as usize],
            _ => None,
        }
    }
}

/// Route a raw input event to whichever module owns that binding.
pub fn trigger(ev: Ev, s: &Arc<Settings>, map: &TriggerMap) {
    let Some(action) = map.get(ev) else { return; };
    match action {
        Action::Dash => fire(s, ev, "vD", dash),
        Action::Shield => fire(s, ev, "v1", shield),
        Action::Mace => fire(s, ev, "v2", mace_swap),
        Action::Breach => fire(s, ev, "v3", breach),
        Action::Potion => fire(s, ev, "v4", potion),
        Action::Crystal => fire(s, ev, "v5", crystal),
        Action::Anchor => fire(s, ev, "v6", anchor),
        Action::FastUse => fire(s, ev, "vA", fast_use),
        Action::Pearl => fire(s, ev, "v7", pearl),
        Action::Totem => fire(s, ev, "v8", totem_swap),
        Action::AutoUse => fire(s, ev, "vW", auto_use),
        Action::WTap => {
            // Most clicks are ordinary attacks. Only allocate the short-lived
            // W-Tap thread when forward is actually held.
            if s.wtap && input::physical_down(Ev::K(0x57)) {
                smart::on_trigger("wtap");
                spawn(s.clone(), ev, w_tap);
            }
        }
        Action::HoverInv => spawn(s.clone(), ev, inventory_peek),
    }
}

#[cfg(test)]
fn scan_action(s: &Settings, ev: Ev) -> Option<Action> {
    if hit(&s.dash_key, ev) || hit(&s.dash_key2, ev) {
        return Some(Action::Dash);
    }
    if hit(&s.shield_key, ev) || hit(&s.shield_key2, ev) {
        return Some(Action::Shield);
    }
    if hit(&s.mace_key, ev) || hit(&s.mace_key2, ev) {
        return Some(Action::Mace);
    }
    if hit(&s.breach_key, ev) || hit(&s.breach_key2, ev) {
        return Some(Action::Breach);
    }
    if hit(&s.potion_key, ev) || hit(&s.potion_key2, ev) {
        return Some(Action::Potion);
    }
    if hit(&s.crystal_key, ev) || hit(&s.crystal_key2, ev) {
        return Some(Action::Crystal);
    }
    if hit(&s.anchor_key, ev) || hit(&s.anchor_key2, ev) {
        return Some(Action::Anchor);
    }
    if hit(&s.fastuse_key, ev) || hit(&s.fastuse_key2, ev) {
        return Some(Action::FastUse);
    }
    if hit(&s.pearl_key, ev) || hit(&s.pearl_key2, ev) {
        return Some(Action::Pearl);
    }
    if hit(&s.totem_key, ev) || hit(&s.totem_key2, ev) {
        return Some(Action::Totem);
    }
    if hit(&s.autouse_key, ev) || hit(&s.autouse_key2, ev) {
        return Some(Action::AutoUse);
    }
    if hit(&s.wtap_key, ev) {
        return Some(Action::WTap);
    }
    if hit(&s.hover_inv_key, ev) {
        return Some(Action::HoverInv);
    }
    None
}

// ------------------------------------------------------------------- macros

fn dash(s: &Settings, _ev: Ev) {
    if !s.master || !s.dash {
        return;
    }
    // One Dash at a time. The cooldown begins after the Restore inputs, so
    // a second trigger cannot restart the sequence before the hotbar is back.
    let _busy = match Busy::enter(&DASH_BUSY) {
        Some(b) => b,
        None => return,
    };
    // Dash is one click again. The previous second click duplicated the
    // attack and delayed the spear/return swap by a full tick.
    lc(s);
    taps(&[&s.dash_spear, &s.dash_restore, &s.dash_restore]);
    input::sleep_ms(s.dash_cd_ms() as f64);
}

fn shield(s: &Settings, ev: Ev) {
    if !s.master || !s.shield {
        return;
    }
    let _busy = match Busy::enter(&SHIELD_BUSY) {
        Some(b) => b,
        None => return,
    };
    lc(s);
    tap(&s.shield_axe);
    // Held past the tap window: keep breaking the shield until release.
    if !input::wait_up(ev, 60) && s.stun {
        while input::raw_down(ev) {
            lc(s);
            wait_smart(s, "v1", 37, 43);
        }
    }
}

fn mace_swap(s: &Settings, _ev: Ev) {
    if !s.master || !s.mace {
        return;
    }
    let _busy = match Busy::enter(&MACE_BUSY) {
        Some(b) => b,
        None => return,
    };
    // Lead the click by the configured one-tick, two-tick or three-tick delay.
    // This gives the client time to settle the current hotbar action before the
    // mace swap, instead of clipping both inputs into the same frame.
    input::sleep_ms(s.mace_delay_ms() as f64);
    lc(s);
    tap(&s.mace_slot);
}

fn breach(s: &Settings, _ev: Ev) {
    if !s.master || !s.breach {
        return;
    }
    let _busy = match Busy::enter(&BREACH_BUSY) {
        Some(b) => b,
        None => return,
    };
    lc(s);
    taps(&[&s.breach_mace, &s.breach_restore, &s.breach_restore]);
}

fn potion(s: &Settings, ev: Ev) {
    if !s.master || !s.potion {
        return;
    }
    let _busy = match Busy::enter(&POTION_BUSY) {
        Some(b) => b,
        None => return,
    };
    if input::wait_up(ev, 100) {
        return;
    }
    tap(&s.potion_slot1);
    rc(s);
    wait_smart(s, "v4", 37, 43);
    rc(s);
    tap(&s.potion_slot2);
    wait_smart(s, "v4", 37, 43);
    taps(&[&s.potion_restore, &s.potion_restore]);
    wait_smart(s, "v4", 37, 43);
    rc(s);
    input::wait_up(ev, 60_000);
}

fn crystal(s: &Settings, ev: Ev) {
    if !s.master || !s.crystal {
        return;
    }
    let obs = keys::parse(&s.crystal_obsidian);
    let cry = keys::parse(&s.crystal_slot);
    let (Some(obs), Some(cry)) = (obs, cry) else {
        return;
    };
    if obs == cry {
        return;
    }
    let _busy = match Busy::enter(&CRYSTAL_BUSY) {
        Some(b) => b,
        None => return,
    };
    tap_exact_key(Some(obs));
    rc(s);
    wait_smart(s, "v5", s.crystal_min, s.crystal_max);
    tap_exact_key(Some(cry));
    rc(s);

    while input::raw_down(ev) {
        wait_smart(s, "v5", s.crystal_min, s.crystal_max);
        if !input::raw_down(ev) {
            break;
        }
        rc(s);
        wait_smart(s, "v5", s.crystal_min, s.crystal_max);
        if !input::raw_down(ev) {
            break;
        }
        lc(s);
    }
}

/// Anchor PvP, one complete action per press.
///
/// A charged anchor detonates when used without glowstone. The reliable
/// sequence is: place -> charge once -> switch off glowstone -> detonate. One
/// charge is enough for the PvP blast; extra charges add delay without
/// improving reach. Every step keeps the one-tick floor, and all three slot
/// keys must be valid and distinct. The busy guard prevents a second press
/// from interleaving hotbar swaps with the sequence already in flight.
fn anchor(s: &Settings, _ev: Ev) {
    if !s.master || !s.anchor {
        return;
    }
    let _busy = match Busy::enter(&ANCHOR_BUSY) {
        Some(b) => b,
        None => return,
    };
    let anchor = keys::parse(&s.anchor_slot);
    let glow = keys::parse(&s.glowstone_slot);
    let totem = keys::parse(&s.anchor_totem);
    let (Some(anchor), Some(glow), Some(totem)) = (anchor, glow, totem) else {
        return;
    };
    if !anchor_slots_ok(anchor, glow, totem) {
        return;
    }

    normal_anchor(s, anchor, glow, totem);

    // Hold the busy guard through the blast tick so a second trigger cannot
    // start another sequence inside the same game tick.
    input::sleep_ms(input::TICK_MS as f64);
}

/// All three Anchor slots must be valid and distinct. Sharing a slot silently
/// turns the sequence into repeated uses of the same item.
fn anchor_slots_ok(anchor: Key, glow: Key, totem: Key) -> bool {
    anchor != glow && anchor != totem && glow != totem
}

/// Normal Anchor: place, one glowstone charge, switch off glowstone, detonate.
fn normal_anchor(s: &Settings, anchor: Key, glow: Key, totem: Key) {
    tap_slot(Some(anchor));
    rc_exact(s);
    anchor_wait(s);

    tap_slot(Some(glow));
    rc_exact(s);
    anchor_wait(s);

    // Switching away from glowstone is the detonation input.
    tap_slot(Some(totem));
    anchor_wait(s);
    rc_exact(s);
}

fn fast_use(s: &Settings, ev: Ev) {
    if !s.master || !s.fastuse {
        return;
    }
    let _busy = match Busy::enter(&FASTUSE_BUSY) {
        Some(b) => b,
        None => return,
    };
    while input::raw_down(ev) {
        rc(s);
        wait_smart(s, "vA", s.fastuse_min, s.fastuse_max);
    }
}

fn pearl(s: &Settings, ev: Ev) {
    if !s.master || !s.pearl {
        return;
    }
    let _busy = match Busy::enter(&PEARL_BUSY) {
        Some(b) => b,
        None => return,
    };
    rc(s);
    tap(&s.pearl_slot);
    if !input::wait_up(ev, 90) {
        tap(&s.pearl_catch);
        tap(&s.pearl_catch);
        if s.catch_w {
            rc(s);
        }
        input::wait_up(ev, 60_000);
    }
}

fn totem_swap(s: &Settings, _ev: Ev) {
    if !s.master || !s.totem {
        return;
    }
    let _busy = match Busy::enter(&TOTEM_BUSY) {
        Some(b) => b,
        None => return,
    };
    tap(&s.totem_hotbar);
    wait_smart(s, "v8", 47, 53);
    tap(&s.totem_swap);
}

fn auto_use(s: &Settings, _ev: Ev) {
    if !s.master || !s.auto_use {
        return;
    }
    let _busy = match Busy::enter(&AUTOUSE_BUSY) {
        Some(b) => b,
        None => return,
    };
    tap(&s.autouse_slot);
    rc(s);
    wait_smart(s, "vW", 10, 20);
    tap(&s.mace_slot);
}

fn inventory_peek(s: &Settings, _ev: Ev) {
    if !s.master || !s.totem || !s.hover_inv {
        return;
    }
    let _busy = match Busy::enter(&INV_BUSY) {
        Some(b) => b,
        None => return,
    };
    tap(&s.totem_hotbar);
}

fn wtap_hold_ms(s: &Settings) -> i32 {
    // Select one value for the whole tap. The selected slider envelope is the
    // time the relevant key is actually held/released.
    smart::pick(s.smart, "wtap", s.wtap_min, s.wtap_max).max(input::TICK_MS)
}

fn wtap_uses_s(s: &Settings) -> bool {
    s.wtap_mode.eq_ignore_ascii_case("S")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WtapPlan {
    release_forward: bool,
    tap_back: bool,
    hold_ms: i32,
}

fn wtap_plan(s: &Settings) -> WtapPlan {
    WtapPlan {
        // Both modes release W. Holding W and S together cancels movement in
        // Minecraft, so S-tap must let go of forward to actually step back.
        release_forward: true,
        tap_back: wtap_uses_s(s),
        hold_ms: wtap_hold_ms(s),
    }
}

fn w_tap(s: &Settings, _ev: Ev) {
    if !s.master || !s.wtap || CRYSTAL_BUSY.load(Ordering::Relaxed) {
        return;
    }
    let w = Ev::K(0x57);
    if !input::physical_down(w) {
        return;
    }
    let w_key = match keys::parse("w") {
        Some(k) => k,
        None => return,
    };
    // A fast click can release the attack button before this short-lived
    // thread starts. The trigger event is the attack; do not re-check it.
    let _busy = match Busy::enter(&WTAP_BUSY) {
        Some(b) => b,
        None => return,
    };

    let plan = wtap_plan(s);
    let s_ev = Ev::K(0x53);
    let s_key = if plan.tap_back {
        match keys::parse("s") {
            Some(k) => Some(k),
            None => return,
        }
    } else {
        None
    };

    // Let go of forward in both modes. For S-tap, keeping W down would make the
    // game cancel W+S and produce no backward movement at all. Suppress held-W
    // auto-repeat until forward is restored below.
    let _suppress_w = if plan.release_forward {
        Some(WRepeatGuard::on())
    } else {
        None
    };
    if plan.release_forward {
        input::release(w_key);
    }

    let mut own_s = false;
    if let Some(k) = s_key {
        if !input::physical_down(s_ev) {
            input::press(k);
            own_s = true;
        }
    }

    input::sleep_ms(plan.hold_ms as f64);

    if own_s {
        if let Some(k) = s_key {
            if !input::physical_down(s_ev) {
                input::release(k);
            }
        }
    }

    // Restore forward only if the user is still physically holding it. If they
    // let go during the tap, leaving it released avoids a stuck key.
    if plan.release_forward && input::physical_down(w) {
        input::press(w_key);
    }
}

// ------------------------------------------------------------------- watchers

#[inline]
fn sprint_due(next: i64, now: i64) -> bool {
    next == 0 || now >= next
}

#[inline]
fn sprint_deadline(now: i64, qpf: i64) -> i64 {
    now.saturating_add(qpf)
}

/// Watches physical W, never Hydrogen's injected W events. Sprint is the one
/// repeating watcher in Hydrogen: while W is genuinely held it taps the sprint
/// binding once per second. The key is never held down between pulses, and the
/// 16 ms watcher does no binding work until a pulse is actually due.
pub fn sprint_tick(s: &Settings, active: bool) {
    let want = s.master && s.sprint && active && input::physical_down(Ev::K(0x57));
    if !want {
        SPRINT_NEXT.store(0, Ordering::Relaxed);
        return;
    }

    let now = input::now();
    if !sprint_due(SPRINT_NEXT.load(Ordering::Relaxed), now) {
        return;
    }
    // Schedule before injecting so a slow SendInput call cannot double-pulse.
    SPRINT_NEXT.store(sprint_deadline(now, input::qpf()), Ordering::Relaxed);

    let Some(key) = keys::parse(&s.sprint_key) else {
        return;
    };
    // Never inject a pulse over the user's own held sprint key.
    if !input::physical_down(key.ev()) {
        input::taps(&[key]);
    }
}

/// Magenta crosshair detection: swaps to the totem when the pixel under the
/// cursor matches (damage flash), exactly like the original timer.
pub fn totem_watch(s: &Settings, probe: &Probe) {
    if !s.master || !s.totem {
        return;
    }
    let (r, g, b) = probe.sample_cursor();
    if r > 214 && g < 41 && b > 214 {
        tap(&s.totem_swap);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wtap_modes_and_hold_delay_follow_the_slider() {
        let mut s = Settings::default();
        assert!(wtap_uses_s(&s));
        assert_eq!((s.wtap_min, s.wtap_max), (132, 148));
        assert!((132..=148).contains(&wtap_hold_ms(&s)));

        s.wtap_mode = "W".into();
        let w = wtap_plan(&s);
        assert!(!wtap_uses_s(&s));
        assert!(w.release_forward);
        assert!(!w.tap_back);
        assert!((132..=148).contains(&w.hold_ms), "hold={}", w.hold_ms);

        s.wtap_mode = "S".into();
        let tap = wtap_plan(&s);
        assert!(wtap_uses_s(&s));
        assert!(tap.release_forward);
        assert!(tap.tap_back);
        assert!((132..=148).contains(&tap.hold_ms), "hold={}", tap.hold_ms);
    }

    #[test]
    fn sprint_pulses_once_per_second() {
        let qpf = 10_000_000i64;
        let first = 123_456i64;
        let deadline = sprint_deadline(first, qpf);
        assert!(!sprint_due(deadline, first));
        assert!(!sprint_due(deadline, deadline - 1));
        assert!(sprint_due(deadline, deadline));
        assert_eq!(sprint_deadline(deadline, qpf) - deadline, qpf);
    }

    #[test]
    fn anchor_waits_are_never_shorter_than_one_tick() {
        let mut s = Settings::default();
        s.anchor_min = 47;
        s.anchor_max = 53;
        assert!(anchor_wait_ms(&s) >= input::TICK_MS);
        s.anchor_min = 97;
        s.anchor_max = 103;
        let wait = anchor_wait_ms(&s);
        assert!((97..=103).contains(&wait), "wait={wait}");
    }

    #[test]
    fn anchor_slots_must_be_distinct() {
        let a = keys::parse("1").unwrap();
        let b = keys::parse("2").unwrap();
        let c = keys::parse("3").unwrap();
        assert!(anchor_slots_ok(a, b, c));
        assert!(!anchor_slots_ok(a, a, c));
        assert!(!anchor_slots_ok(a, c, a));
        assert!(!anchor_slots_ok(c, b, b));
    }

    #[test]
    fn trigger_index_matches_the_binding_scan_and_is_faster() {
        use std::time::Instant;

        let s = Settings::default();
        let map = TriggerMap::build(&s);
        let events = [
            Ev::K(0x42),
            Ev::K(0x51),
            Ev::K(0x52),
            Ev::M(Btn::X1),
            Ev::K(0x43),
            Ev::K(0x5A),
            Ev::K(0x58),
            Ev::M(Btn::X2),
            Ev::M(Btn::Middle),
            Ev::K(0x4E),
            Ev::K(0x56),
            Ev::M(Btn::Left),
            Ev::K(0x45),
            Ev::K(0x99),
        ];
        for ev in events {
            assert_eq!(map.get(ev), scan_action(&s, ev), "event {ev:?}");
        }
        assert_eq!(map.get(Ev::K(0x58)), Some(Action::Anchor));
        assert_eq!(map.get(Ev::K(0x45)), Some(Action::HoverInv));
        assert_eq!(map.get(Ev::K(0x99)), None);

        let ev = Ev::K(0x58);
        let n = 1_000_000u32;
        for _ in 0..10_000 {
            std::hint::black_box(map.get(ev));
            std::hint::black_box(scan_action(&s, ev));
        }
        let start = Instant::now();
        for _ in 0..n {
            std::hint::black_box(map.get(ev));
        }
        let indexed = start.elapsed().as_nanos() as f64 / n as f64;

        let start = Instant::now();
        for _ in 0..n {
            std::hint::black_box(scan_action(&s, ev));
        }
        let scan = start.elapsed().as_nanos() as f64 / n as f64;
        println!(
            "trigger route: indexed={indexed:.2} ns  scan={scan:.2} ns  speedup={:.1}x",
            scan / indexed
        );
        assert!(indexed < scan, "indexed={indexed} ns scan={scan} ns");
    }
}
