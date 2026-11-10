//! The macro modules, ported one-for-one from the AutoHotkey original.
//!
//! Every macro runs on its own short-lived thread (the original relied on
//! AutoHotkey spawning a thread per hotkey); waits are event driven where the
//! key state matters and QPC-precise where a fixed delay is needed.

#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use crate::config::Settings;
use crate::input::{self, Probe};
use crate::keys::{self, Btn, Ev, Key};
use crate::smart;

pub static CRYSTAL_BUSY: AtomicBool = AtomicBool::new(false);
static INV_BUSY: AtomicBool = AtomicBool::new(false);
static DASH_BUSY: AtomicBool = AtomicBool::new(false);
static ANCHOR_BUSY: AtomicBool = AtomicBool::new(false);
/// Air Anchor keeps the anchor placed by the previous pop/place floating in
/// the air. Later cycles charge that anchor instead of trying to place another
/// one on top of it.
static AIR_READY: AtomicBool = AtomicBool::new(false);
/// Bumped on every Air reset. A cycle captures this before it starts working
/// and may only publish readiness if the epoch is still current, so a cycle
/// that finishes after a stop/restart cannot skip the next sequence's prime.
static AIR_EPOCH: AtomicU64 = AtomicU64::new(0);
static WTAP_BUSY: AtomicBool = AtomicBool::new(false);
static SHIELD_BUSY: AtomicBool = AtomicBool::new(false);
static MACE_BUSY: AtomicBool = AtomicBool::new(false);
static BREACH_BUSY: AtomicBool = AtomicBool::new(false);
static POTION_BUSY: AtomicBool = AtomicBool::new(false);
static FASTUSE_BUSY: AtomicBool = AtomicBool::new(false);
static PEARL_BUSY: AtomicBool = AtomicBool::new(false);
static TOTEM_BUSY: AtomicBool = AtomicBool::new(false);
static AUTOUSE_BUSY: AtomicBool = AtomicBool::new(false);
static SPRINT_OWNED: std::sync::Mutex<Option<Key>> = std::sync::Mutex::new(None);

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
/// later is what makes an in-game loop skip or double-fire. The clamp keeps the
/// configured 47-53 envelope while guaranteeing the game can observe each step.
fn anchor_wait_ms(s: &Settings) -> i32 {
    smart::pick(s.smart, "v6", s.anchor_min, s.anchor_max).max(input::TICK_MS)
}

fn anchor_wait(s: &Settings) {
    input::sleep_ms(anchor_wait_ms(s) as f64);
}

/// Select a hotbar slot and let the client observe the selection before the
/// following use input. Without this settle, a loop can occasionally use the
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

fn hit(binding: &str, ev: Ev) -> bool {
    keys::ev(binding).map(|e| e == ev).unwrap_or(false)
}

// ------------------------------------------------------------ toggle spam

/// Modules currently latched in "toggle" mode (drives the UI activity light).
static SPAM: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());

/// One generation counter per toggleable module. A loop thread is valid only
/// for the generation that created it. Releasing and re-latching a module
/// before its previous thread has observed the release must never leave two
/// loops injecting the same macro at once.
const LOOP_SLOTS: usize = 12;
static LOOP_GEN: [AtomicU64; LOOP_SLOTS] = [const { AtomicU64::new(0) }; LOOP_SLOTS];

fn loop_slot(module: &str) -> usize {
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
        _ => 11,
    }
}

fn next_loop_generation(module: &str) -> u64 {
    LOOP_GEN[loop_slot(module)].fetch_add(1, Ordering::SeqCst).wrapping_add(1)
}

fn loop_generation_is(module: &str, generation: u64) -> bool {
    LOOP_GEN[loop_slot(module)].load(Ordering::SeqCst) == generation
}

fn cancel_loops() {
    for generation in &LOOP_GEN {
        generation.fetch_add(1, Ordering::SeqCst);
    }
}

fn start_toggle(ev: Ev, module: &'static str) -> Option<u64> {
    let generation = next_loop_generation(module);
    let on = input::flip_latch(ev);
    set_spam(module, on);
    repaint();
    if on { Some(generation) } else { None }
}

fn set_spam(module: &'static str, on: bool) {
    if let Ok(mut v) = SPAM.lock() {
        v.retain(|m| *m != module);
        if on {
            v.push(module);
        }
    }
}

pub fn spam_active(module: &str) -> bool {
    SPAM.lock().map(|v| v.iter().any(|m| *m == module)).unwrap_or(false)
}

/// Map a mode-chip key to the module switch it drives.
pub fn mode_module(mode_key: &str) -> &'static str {
    match mode_key {
        "vDMd" => "vD",
        "v1Md" => "v1",
        "v2Md" => "v2",
        "v3Md" => "v3",
        "v4Md" => "v4",
        "v5Md" => "v5",
        "v6Md" => "v6",
        "v7Md" => "v7",
        "v8Md" => "v8",
        "vABMd" => "vA",
        "vWMd" => "vW",
        _ => "",
    }
}

/// Drop every latch: called when the master switch or focus is lost.
pub fn stop_all_spam() {
    input::clear_latches();
    cancel_loops();
    reset_air_ready();
    if let Ok(mut v) = SPAM.lock() {
        v.clear();
    }
    repaint();
}

fn repaint() {
    let h = crate::app::main_hwnd();
    if h != 0 {
        unsafe { crate::win::PostMessageW(h, crate::win::WM_APP_TICK, 0, 0) };
    }
}

fn fire(
    s: &Arc<Settings>,
    ev: Ev,
    mode: &str,
    module: &'static str,
    f: fn(&Settings, Ev),
) {
    smart::on_trigger(module);
    if mode.eq_ignore_ascii_case("toggle") {
        // Bump before changing the latch. The bump invalidates any older loop,
        // including one that is still in the middle of a slow Anchor action.
        if let Some(generation) = start_toggle(ev, module) {
            spawn(s.clone(), ev, move |s, ev| {
                while input::latched(ev) && loop_generation_is(module, generation) {
                    // One loop iteration is exactly one normal macro activation.
                    f(s, ev);
                    if !input::latched(ev) || !loop_generation_is(module, generation) {
                        break;
                    }
                    input::wait_range(55, 85);
                }
            });
        }
        return;
    }
    spawn(s.clone(), ev, f);
}

/// Anchor gets a dedicated loop because its action is long and must never be
/// silently skipped while another Anchor action is still finishing. The cycle
/// gap is fixed at one tick instead of the generic random gap, so repeated
/// in-game cycles have a stable rhythm.
fn fire_anchor(s: &Arc<Settings>, ev: Ev) {
    smart::on_trigger("v6");
    if !s.anchor || !s.anchor_air {
        reset_air_ready();
    }
    if s.anchor_mode.eq_ignore_ascii_case("toggle") {
        let generation = start_toggle(ev, "v6");
        // Every fresh toggle sequence starts from a clean Air state. This also
        // invalidates a cycle that is still finishing from the previous run.
        reset_air_ready();
        if let Some(generation) = generation {
            spawn(s.clone(), ev, move |s, ev| {
                while input::latched(ev) && loop_generation_is("v6", generation) {
                    anchor(s, ev);
                    if !input::latched(ev) || !loop_generation_is("v6", generation) {
                        break;
                    }
                    input::sleep_ms(input::TICK_MS as f64);
                }
            });
        }
        return;
    }
    spawn(s.clone(), ev, anchor);
}

/// Route a raw input event to whichever module owns that binding.
pub fn trigger(ev: Ev, s: &Arc<Settings>) {
    if hit(&s.dash_key, ev) || hit(&s.dash_key2, ev) {
        fire(s, ev, &s.dash_mode, "vD", dash);
        return;
    }
    if hit(&s.shield_key, ev) || hit(&s.shield_key2, ev) {
        fire(s, ev, &s.shield_mode, "v1", shield);
        return;
    }
    if hit(&s.mace_key, ev) || hit(&s.mace_key2, ev) {
        fire(s, ev, &s.mace_mode, "v2", mace_swap);
        return;
    }
    if hit(&s.breach_key, ev) || hit(&s.breach_key2, ev) {
        fire(s, ev, &s.breach_mode, "v3", breach);
        return;
    }
    if hit(&s.potion_key, ev) || hit(&s.potion_key2, ev) {
        fire(s, ev, &s.potion_mode, "v4", potion);
        return;
    }
    if hit(&s.crystal_key, ev) || hit(&s.crystal_key2, ev) {
        fire(s, ev, &s.crystal_mode, "v5", crystal);
        return;
    }
    if hit(&s.anchor_key, ev) || hit(&s.anchor_key2, ev) {
        fire_anchor(s, ev);
        return;
    }
    if hit(&s.fastuse_key, ev) || hit(&s.fastuse_key2, ev) {
        fire(s, ev, &s.fastuse_mode, "vA", fast_use);
        return;
    }
    if hit(&s.pearl_key, ev) || hit(&s.pearl_key2, ev) {
        fire(s, ev, &s.pearl_mode, "v7", pearl);
        return;
    }
    if hit(&s.totem_key, ev) || hit(&s.totem_key2, ev) {
        fire(s, ev, &s.totem_mode, "v8", totem_swap);
        return;
    }
    if hit(&s.autouse_key, ev) || hit(&s.autouse_key2, ev) {
        fire(s, ev, &s.autouse_mode, "vW", auto_use);
        return;
    }
    if hit(&s.wtap_key, ev) {
        // Most clicks are ordinary attacks. Only allocate the short-lived
        // W-Tap thread when forward is actually held.
        if s.wtap && input::physical_down(Ev::K(0x57)) {
            smart::on_trigger("wtap");
            spawn(s.clone(), ev, w_tap);
        }
        return;
    }
    if hit(&s.hover_inv_key, ev) {
        spawn(s.clone(), ev, inventory_peek);
    }
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

/// Anchor PvP, one complete anchor action per call.
///
/// A charged anchor detonates when used without glowstone. The reliable
/// sequence is: place -> charge once -> switch off glowstone -> detonate. One
/// charge is enough for the PvP blast; extra charges add delay without
/// improving reach. Air anchor primes one anchor on the first cycle, then
/// charges the floating anchor and pops-and-replaces it in a single client
/// tick before equipping the totem. This function is the single source of
/// truth for both hold and toggle/loop mode, so a loop cycle can never degrade
/// into repeating just the anchor-slot key. Every step keeps the one-tick
/// floor, and all three slot keys must be valid and distinct.
fn anchor(s: &Settings, _ev: Ev) {
    if !s.master || !s.anchor {
        reset_air_ready();
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
        reset_air_ready();
        return;
    };
    if anchor == glow || anchor == totem || glow == totem {
        reset_air_ready();
        return;
    }

    if s.anchor_air {
        air_anchor(s, anchor, glow, totem);
    } else {
        reset_air_ready();
        normal_anchor(s, anchor, glow, totem);
    }

    // Hold the busy guard through the blast tick so a held/toggle trigger
    // cannot start another sequence inside the same game tick.
    input::sleep_ms(input::TICK_MS as f64);
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AirStep {
    Prime,
    Charge,
    PopPlace,
    Totem,
}

/// Air Anchor keeps the floating anchor placed by the previous pop/place, so a
/// later cycle charges and detonates that anchor instead of trying to place a
/// second one on top of it. Only the first cycle primes a fresh anchor.
fn air_steps(ready: bool) -> &'static [AirStep] {
    if ready {
        &[AirStep::Charge, AirStep::PopPlace, AirStep::Totem]
    } else {
        &[
            AirStep::Prime,
            AirStep::Charge,
            AirStep::PopPlace,
            AirStep::Totem,
        ]
    }
}

fn reset_air_ready() {
    AIR_EPOCH.fetch_add(1, Ordering::SeqCst);
    AIR_READY.store(false, Ordering::SeqCst);
}

fn air_is_ready() -> bool {
    AIR_READY.load(Ordering::SeqCst)
}

fn commit_air_ready(epoch: u64) {
    if AIR_EPOCH.load(Ordering::SeqCst) == epoch {
        AIR_READY.store(true, Ordering::SeqCst);
    }
}

fn air_anchor(s: &Settings, anchor: Key, glow: Key, totem: Key) {
    let epoch = AIR_EPOCH.load(Ordering::SeqCst);
    for step in air_steps(air_is_ready()) {
        match step {
            AirStep::Prime => {
                tap_slot(Some(anchor));
                rc_exact(s);
                anchor_wait(s);
            }
            AirStep::Charge => {
                tap_slot(Some(glow));
                rc_exact(s);
                anchor_wait(s);
            }
            AirStep::PopPlace => {
                // Pop the charged anchor and place the next anchor in the same
                // client tick. Two clicks one tick apart let the explosion
                // remove the old anchor before the replacement use lands,
                // which is why the loop kept losing its floating anchor.
                tap_slot(Some(anchor));
                input::jitter(s.ghost, s.ghost_off);
                input::click_pair_exact(Btn::Right);
                anchor_wait(s);
            }
            AirStep::Totem => {
                tap_exact_key(Some(totem));
            }
        }
    }
    commit_air_ready(epoch);
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

fn release_owned(owned: &mut Option<Key>, force: bool) {
    if let Some(k) = owned.take() {
        // Do not send a key-up over a real hold by the user.
        if force || !input::physical_down(k.ev()) {
            input::release(k);
        }
    }
}

/// Watches physical W, never Hydrogen's injected W events. This is what lets
/// Sprint stay held through a W-Tap's synthetic release and re-press.
pub fn sprint_tick(s: &Settings, active: bool) {
    let want = s.master && s.sprint && active && input::physical_down(Ev::K(0x57));
    let mut owned = SPRINT_OWNED.lock().unwrap();
    if !want {
        release_owned(&mut owned, false);
        return;
    }
    let key = match keys::parse(&s.sprint_key) {
        Some(k) => k,
        None => {
            release_owned(&mut owned, false);
            return;
        }
    };
    if owned.map_or(false, |old| old != key) {
        release_owned(&mut owned, false);
    }
    // Never claim a sprint key the user is already holding.
    if owned.is_none() && !input::physical_down(key.ev()) {
        input::press(key);
        *owned = Some(key);
    }
}

pub fn release_sprint(force: bool) {
    let mut owned = SPRINT_OWNED.lock().unwrap();
    release_owned(&mut owned, force);
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
    fn relatching_invalidates_the_previous_loop_generation() {
        let first = next_loop_generation("v6");
        assert!(loop_generation_is("v6", first));
        let second = next_loop_generation("v6");
        assert!(!loop_generation_is("v6", first));
        assert!(loop_generation_is("v6", second));
    }

    #[test]
    fn wtap_modes_and_hold_delay_follow_the_slider() {
        let mut s = Settings::default();
        s.wtap_min = 100;
        s.wtap_max = 120;
        let w = wtap_plan(&s);
        assert!(!wtap_uses_s(&s));
        assert!(w.release_forward);
        assert!(!w.tap_back);
        assert!((100..=120).contains(&w.hold_ms), "hold={}", w.hold_ms);

        s.wtap_mode = "S".into();
        let tap = wtap_plan(&s);
        assert!(wtap_uses_s(&s));
        assert!(tap.release_forward);
        assert!(tap.tap_back);
        assert!((100..=120).contains(&tap.hold_ms), "hold={}", tap.hold_ms);
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
    fn air_readiness_ignores_a_cycle_started_before_reset() {
        reset_air_ready();
        let stale = AIR_EPOCH.load(Ordering::SeqCst);
        assert!(!air_is_ready());
        commit_air_ready(stale);
        assert!(air_is_ready());

        reset_air_ready();
        commit_air_ready(stale);
        assert!(!air_is_ready());
    }

    #[test]
    fn air_anchor_reuses_the_floating_anchor_after_the_first_cycle() {
        assert_eq!(
            air_steps(false),
            &[
                AirStep::Prime,
                AirStep::Charge,
                AirStep::PopPlace,
                AirStep::Totem
            ][..]
        );
        assert_eq!(
            air_steps(true),
            &[AirStep::Charge, AirStep::PopPlace, AirStep::Totem][..]
        );
    }
}
