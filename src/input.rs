//! Input injection, precision waiting, release signalling and pixel probing.

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::Duration;

use crate::keys::{Btn, Ev, Key};
use crate::win::{self, INPUT, INPUTUNION, KEYBDINPUT, MOUSEINPUT};

/// Marker written into `dwExtraInfo` so our own hook ignores injected events.
pub const INJ: usize = 0x4859_4452; // "HYDR"

/// One Minecraft game tick. Anchor steps are floored to this so two actions
/// never collapse into the same tick (the "too fast" ghost-click failure).
pub const TICK_MS: i32 = 50;

static QPF: AtomicI64 = AtomicI64::new(1);
static SEQ: AtomicU64 = AtomicU64::new(0);

/// Raise the calling thread to time-critical priority (low latency mode).
pub fn boost_thread() {
    unsafe {
        win::SetThreadPriority(win::GetCurrentThread(), win::THREAD_PRIORITY_TIME_CRITICAL);
    }
}

pub fn init() {
    let mut f = 0i64;
    unsafe { win::QueryPerformanceFrequency(&mut f) };
    QPF.store(if f > 0 { f } else { 10_000_000 }, Ordering::Relaxed);
}

#[inline]
pub fn now() -> i64 {
    let mut v = 0i64;
    unsafe { win::QueryPerformanceCounter(&mut v) };
    v
}

#[inline]
pub fn qpf() -> i64 {
    QPF.load(Ordering::Relaxed)
}

/// Sleep with sub-millisecond accuracy: coarse sleep, then a bounded spin.
pub fn sleep_ms(ms: f64) {
    if ms <= 0.0 {
        return;
    }
    let freq = qpf() as f64;
    let target = now() + (ms * freq / 1000.0) as i64;
    loop {
        let remain = target - now();
        if remain <= 0 {
            break;
        }
        let remain_ms = remain as f64 * 1000.0 / freq;
        if remain_ms > 1.6 {
            std::thread::sleep(Duration::from_micros(((remain_ms - 1.2) * 1000.0) as u64));
        } else {
            std::hint::spin_loop();
        }
    }
}

// ------------------------------------------------------------------- random

fn seed() -> u64 {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    (now() as u64) ^ n.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03
}

thread_local! {
    static RNG: std::cell::Cell<u64> = std::cell::Cell::new(seed());
}

fn next_rand() -> u64 {
    RNG.with(|c| {
        let mut x = c.get();
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        c.set(x);
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    })
}

/// Inclusive integer range, mirroring AutoHotkey's `Random(min, max)`.
pub fn rand_range(a: i32, b: i32) -> i32 {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    let span = (hi as i64 - lo as i64 + 1) as u64;
    (lo as i64 + (next_rand() % span) as i64) as i32
}

/// Wait a random delay in the given range, returning the value used.
pub fn wait_range(a: i32, b: i32) -> i32 {
    let ms = rand_range(a, b);
    sleep_ms(ms as f64);
    ms
}

/// Random wait with a floor, used by the tick-paced Anchor sequence.
pub fn wait_range_floor(a: i32, b: i32, floor: i32) -> i32 {
    let ms = rand_range(a, b).max(floor);
    sleep_ms(ms as f64);
    ms
}

// ------------------------------------------------------------------ inject

#[inline]
fn mk_key(k: Key, up: bool) -> INPUT {
    match k {
        Key::Vk { vk, sc, ext } => {
            let mut flags = 0;
            if ext {
                flags |= win::KEYEVENTF_EXTENDEDKEY;
            }
            if up {
                flags |= win::KEYEVENTF_KEYUP;
            }
            INPUT {
                r#type: win::INPUT_KEYBOARD,
                u: INPUTUNION {
                    ki: KEYBDINPUT {
                        wVk: vk as u16,
                        wScan: sc,
                        dwFlags: flags,
                        time: 0,
                        dwExtraInfo: INJ,
                    },
                },
            }
        }
        Key::Mouse(b) => mk_mouse(b, up),
    }
}

#[inline]
fn mk_mouse(b: Btn, up: bool) -> INPUT {
    let (flags, data) = match b {
        Btn::Left => (if up { win::MOUSEEVENTF_LEFTUP } else { win::MOUSEEVENTF_LEFTDOWN }, 0),
        Btn::Right => (if up { win::MOUSEEVENTF_RIGHTUP } else { win::MOUSEEVENTF_RIGHTDOWN }, 0),
        Btn::Middle => (if up { win::MOUSEEVENTF_MIDDLEUP } else { win::MOUSEEVENTF_MIDDLEDOWN }, 0),
        Btn::X1 => (if up { win::MOUSEEVENTF_XUP } else { win::MOUSEEVENTF_XDOWN }, 1),
        Btn::X2 => (if up { win::MOUSEEVENTF_XUP } else { win::MOUSEEVENTF_XDOWN }, 2),
    };
    INPUT {
        r#type: win::INPUT_MOUSE,
        u: INPUTUNION {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: INJ,
            },
        },
    }
}

fn send(buf: &[INPUT]) {
    if buf.is_empty() {
        return;
    }
    unsafe {
        win::SendInput(buf.len() as u32, buf.as_ptr(), std::mem::size_of::<INPUT>() as i32);
    }
}

/// Press and release every key, in one batched SendInput call.
pub fn taps(keys: &[Key]) {
    let mut buf = [unsafe { std::mem::zeroed::<INPUT>() }; 16];
    let mut n = 0usize;
    for k in keys {
        if n + 2 > buf.len() {
            send(&buf[..n]);
            n = 0;
        }
        buf[n] = mk_key(*k, false);
        n += 1;
        buf[n] = mk_key(*k, true);
        n += 1;
    }
    if n > 0 {
        send(&buf[..n]);
    }
}

pub fn press(k: Key) {
    send(&[mk_key(k, false)]);
}

pub fn release(k: Key) {
    send(&[mk_key(k, true)]);
}

pub fn click(btn: Btn, up: bool) {
    send(&[mk_mouse(btn, up)]);
}

pub fn click2(btn: Btn) {
    send(&[mk_mouse(btn, false), mk_mouse(btn, true)]);
}

/// One key down + up as two separate SendInput calls (the AHK `SQ` shape).
pub fn tap_exact(k: Key) {
    press(k);
    release(k);
}

/// One button down + up as two separate SendInput calls (the AHK `RC` shape).
pub fn click_exact(btn: Btn) {
    click(btn, false);
    click(btn, true);
}

/// Humanised micro-jitter applied before clicks (the "ghost click" feature).
pub fn jitter(on: bool, amount: i32) {
    if !on {
        return;
    }
    let a = amount.clamp(1, 24);
    let dx = rand_range(-a, a);
    let dy = rand_range(-a, a);
    let inp = INPUT {
        r#type: win::INPUT_MOUSE,
        u: INPUTUNION {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: win::MOUSEEVENTF_MOVE,
                time: 0,
                dwExtraInfo: INJ,
            },
        },
    };
    send(&[inp]);
}

// ------------------------------------------------------------ key state/sync

pub fn raw_down(e: Ev) -> bool {
    let vk = match e {
        Ev::K(vk) => vk,
        Ev::M(b) => b.vk(),
    };
    unsafe { (win::GetAsyncKeyState(vk as i32) as u16 & 0x8000) != 0 }
}

/// Physical key/button state as seen by the hooks, excluding every synthetic
/// event Hydrogen sends. W-Tap can therefore release W without the sprint
/// helper thinking the user let go, and it can avoid re-pressing W if the user
/// actually released it during the tap window.
static PHYS_K: [AtomicU64; 4] = [const { AtomicU64::new(0) }; 4];
static PHYS_M: [AtomicBool; 8] = [const { AtomicBool::new(false) }; 8];

/// Set while a W-Tap owns the forward key. The physical W key remains held by
/// the user, so Windows can keep generating auto-repeat key-down events after
/// we inject the release. Those repeats would immediately cancel an S-Tap's
/// backward movement; the keyboard hook drops only the repeats, never the
/// initial press or the physical release.
static SUPPRESS_W_REPEAT: AtomicBool = AtomicBool::new(false);

pub fn set_w_repeat_suppressed(on: bool) {
    SUPPRESS_W_REPEAT.store(on, Ordering::SeqCst);
}

pub fn w_repeat_suppressed() -> bool {
    SUPPRESS_W_REPEAT.load(Ordering::SeqCst)
}

pub fn set_physical(e: Ev, down: bool) {
    match e {
        Ev::K(vk) => {
            let i = (vk as usize) >> 6;
            if i >= PHYS_K.len() {
                return;
            }
            let bit = 1u64 << (vk & 63);
            if down {
                PHYS_K[i].fetch_or(bit, Ordering::Relaxed);
            } else {
                PHYS_K[i].fetch_and(!bit, Ordering::Relaxed);
            }
        }
        Ev::M(b) => {
            PHYS_M[(b as usize).min(PHYS_M.len() - 1)].store(down, Ordering::Relaxed);
        }
    }
}

pub fn physical_down(e: Ev) -> bool {
    match e {
        Ev::K(vk) => {
            let i = (vk as usize) >> 6;
            if i >= PHYS_K.len() {
                return false;
            }
            (PHYS_K[i].load(Ordering::Relaxed) & (1u64 << (vk & 63))) != 0
        }
        Ev::M(b) => PHYS_M[(b as usize).min(PHYS_M.len() - 1)].load(Ordering::Relaxed),
    }
}

struct Sig {
    generation: Mutex<u64>,
    cv: Condvar,
}

fn sigs() -> &'static Mutex<HashMap<u32, Arc<Sig>>> {
    static MAP: OnceLock<Mutex<HashMap<u32, Arc<Sig>>>> = OnceLock::new();
    MAP.get_or_init(|| Mutex::new(HashMap::new()))
}

fn sig_for(id: u32) -> Arc<Sig> {
    let mut map = sigs().lock().unwrap();
    map.entry(id)
        .or_insert_with(|| {
            Arc::new(Sig {
                generation: Mutex::new(0),
                cv: Condvar::new(),
            })
        })
        .clone()
}

/// Called from the input hook on every release: wakes waiting macro threads.
pub fn signal_release(e: Ev) {
    let sig = {
        let map = sigs().lock().unwrap();
        map.get(&e.id()).cloned()
    };
    if let Some(sig) = sig {
        let mut g = sig.generation.lock().unwrap();
        *g = g.wrapping_add(1);
        sig.cv.notify_all();
    }
}

/// Block until the key is released or `ms` elapses. Returns `true` when the key
/// is up (event driven, so the reaction time is a single wake-up).
pub fn wait_up(e: Ev, ms: u64) -> bool {
    let sig = sig_for(e.id());
    let g = sig.generation.lock().unwrap();
    let base = *g;
    if !raw_down(e) {
        return true;
    }
    let res = sig
        .cv
        .wait_timeout_while(g, Duration::from_millis(ms), |v| *v == base)
        .unwrap();
    !res.1.timed_out() || !raw_down(e)
}

// --------------------------------------------------------------- pixel probe

/// 1x1 screen sampler. A BitBlt into a cached DIB is roughly an order of
/// magnitude faster than `GetPixel` on the display DC.
pub struct Probe {
    mem: win::HDC,
    bmp: win::HBITMAP,
    bits: *mut u32,
}

impl Probe {
    pub fn new() -> Option<Probe> {
        unsafe {
            let screen = win::GetDC(0);
            if screen == 0 {
                return None;
            }
            let mem = win::CreateCompatibleDC(screen);
            win::ReleaseDC(0, screen);
            if mem == 0 {
                return None;
            }
            let mut bmi = win::BITMAPINFO::default();
            bmi.bmiHeader.biSize = std::mem::size_of::<win::BITMAPINFOHEADER>() as u32;
            bmi.bmiHeader.biWidth = 1;
            bmi.bmiHeader.biHeight = -1;
            bmi.bmiHeader.biPlanes = 1;
            bmi.bmiHeader.biBitCount = 32;
            bmi.bmiHeader.biCompression = 0;
            let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
            let bmp = win::CreateDIBSection(mem, &bmi, win::DIB_RGB_COLORS, &mut bits, 0, 0);
            if bmp == 0 || bits.is_null() {
                win::DeleteDC(mem);
                return None;
            }
            win::SelectObject(mem, bmp);
            Some(Probe {
                mem,
                bmp,
                bits: bits as *mut u32,
            })
        }
    }

    pub fn sample(&self, x: i32, y: i32) -> (u8, u8, u8) {
        unsafe {
            let screen = win::GetDC(0);
            win::BitBlt(self.mem, 0, 0, 1, 1, screen, x, y, win::SRCCOPY);
            win::ReleaseDC(0, screen);
            let v = *self.bits;
            ((v & 0xFF) as u8, ((v >> 8) & 0xFF) as u8, ((v >> 16) & 0xFF) as u8)
        }
    }

    pub fn sample_cursor(&self) -> (u8, u8, u8) {
        let mut p = win::POINT::default();
        unsafe { win::GetCursorPos(&mut p) };
        self.sample(p.x, p.y)
    }
}

impl Drop for Probe {
    fn drop(&mut self) {
        unsafe {
            win::DeleteObject(self.bmp);
            win::DeleteDC(self.mem);
        }
    }
}
