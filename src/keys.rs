//! Key names, virtual-key resolution and mouse buttons. Names follow the
//! AutoHotkey spelling used by the original script so existing bindings keep
//! working.

#![allow(dead_code)]

use std::cell::RefCell;

use crate::win;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Btn {
    Left = 1,
    Right = 2,
    Middle = 3,
    X1 = 4,
    X2 = 5,
}

impl Btn {
    pub fn name(self) -> &'static str {
        match self {
            Btn::Left => "LButton",
            Btn::Right => "RButton",
            Btn::Middle => "MButton",
            Btn::X1 => "XButton1",
            Btn::X2 => "XButton2",
        }
    }
    pub fn vk(self) -> u32 {
        match self {
            Btn::Left => 0x01,
            Btn::Right => 0x02,
            Btn::Middle => 0x04,
            Btn::X1 => 0x05,
            Btn::X2 => 0x06,
        }
    }
}

/// An input event: a keyboard virtual-key or a mouse button.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ev {
    K(u32),
    M(Btn),
}

impl Ev {
    /// Identifier used by the release-signal map.
    pub fn id(self) -> u32 {
        match self {
            Ev::K(vk) => vk,
            Ev::M(b) => 0x1000 + b as u32,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    Vk { vk: u32, sc: u16, ext: bool },
    Mouse(Btn),
}

impl Key {
    pub fn ev(self) -> Ev {
        match self {
            Key::Vk { vk, .. } => Ev::K(vk),
            Key::Mouse(b) => Ev::M(b),
        }
    }
}

struct EvCache {
    hash: u64,
    name: String,
    ev: Option<Ev>,
}

thread_local! {
    static EV_CACHE: RefCell<Vec<EvCache>> = RefCell::new(Vec::with_capacity(24));
}

fn fnv(s: &str) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x1000_0000_01B3);
    }
    h
}

/// Resolve a binding name into an input event. Bindings change rarely, so the
/// parsed result is memoized per thread: the global hook stops reparsing two
/// dozen strings on every key press.
pub fn ev(name: &str) -> Option<Ev> {
    let hash = fnv(name);
    let cached = EV_CACHE.with(|c| {
        c.borrow()
            .iter()
            .rev()
            .find(|e| e.hash == hash && e.name == name)
            .map(|e| e.ev)
    });
    if let Some(hit) = cached {
        return hit;
    }
    let parsed = ev_uncached(name);
    EV_CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() >= 64 {
            c.remove(0);
        }
        c.push(EvCache { hash, name: name.to_string(), ev: parsed });
    });
    parsed
}

fn ev_uncached(name: &str) -> Option<Ev> {
    let n = name.trim();
    if n.is_empty() || n == "..." {
        return None;
    }
    match n.to_ascii_lowercase().as_str() {
        "lbutton" => return Some(Ev::M(Btn::Left)),
        "rbutton" => return Some(Ev::M(Btn::Right)),
        "mbutton" => return Some(Ev::M(Btn::Middle)),
        "xbutton1" => return Some(Ev::M(Btn::X1)),
        "xbutton2" => return Some(Ev::M(Btn::X2)),
        _ => {}
    }
    vk_of(name).map(Ev::K)
}

/// Resolve a binding name into something we can inject.
pub fn parse(name: &str) -> Option<Key> {
    match ev(name)? {
        Ev::M(b) => Some(Key::Mouse(b)),
        Ev::K(vk) => {
            let scx = unsafe { win::MapVirtualKeyW(vk, win::MAPVK_VK_TO_VSC_EX) };
            let ext = (scx & 0xFF00) != 0;
            let mut sc = (scx & 0xFF) as u16;
            if sc == 0 {
                sc = unsafe { win::MapVirtualKeyW(vk, win::MAPVK_VK_TO_VSC) } as u16;
            }
            Some(Key::Vk { vk, sc, ext })
        }
    }
}

pub fn vk_of(name: &str) -> Option<u32> {
    let n = name.trim();
    if n.is_empty() || n == "..." {
        return None;
    }
    let lower = n.to_ascii_lowercase();
    let vk = match lower.as_str() {
        "backspace" => 0x08,
        "tab" => 0x09,
        "enter" | "return" => 0x0D,
        "shift" => 0x10,
        "control" | "ctrl" => 0x11,
        "alt" => 0x12,
        "pause" => 0x13,
        "capslock" => 0x14,
        "escape" | "esc" => 0x1B,
        "space" => 0x20,
        "pageup" | "pgup" => 0x21,
        "pagedown" | "pgdn" => 0x22,
        "end" => 0x23,
        "home" => 0x24,
        "left" => 0x25,
        "up" => 0x26,
        "right" => 0x27,
        "down" => 0x28,
        "printscreen" => 0x2C,
        "insert" | "ins" => 0x2D,
        "delete" | "del" => 0x2E,
        "numlock" => 0x90,
        "scrolllock" => 0x91,
        "lshift" => 0xA0,
        "rshift" => 0xA1,
        "lcontrol" | "lctrl" => 0xA2,
        "rcontrol" | "rctrl" => 0xA3,
        "lalt" => 0xA4,
        "ralt" => 0xA5,
        "numpadmult" => 0x6A,
        "numpadadd" => 0x6B,
        "numpadsub" => 0x6D,
        "numpaddiv" => 0x6F,
        "numpaddot" | "numpaddel" => 0x6E,
        "-" => 0xBD,
        "=" => 0xBB,
        "[" => 0xDB,
        "]" => 0xDD,
        "\\" => 0xDC,
        ";" => 0xBA,
        "'" => 0xDE,
        "," => 0xBC,
        "." => 0xBE,
        "/" => 0xBF,
        "`" => 0xC0,
        _ => {
            if let Some(rest) = lower.strip_prefix('f') {
                if let Ok(num) = rest.parse::<u32>() {
                    if (1..=24).contains(&num) {
                        return Some(0x70 + num - 1);
                    }
                }
            }
            if let Some(rest) = lower.strip_prefix("numpad") {
                if let Ok(num) = rest.parse::<u32>() {
                    if num <= 9 {
                        return Some(0x60 + num);
                    }
                }
                return None;
            }
            let mut chars = n.chars();
            let first = chars.next()?;
            if chars.next().is_some() {
                return None;
            }
            let up = first.to_ascii_uppercase();
            if up.is_ascii_alphabetic() || up.is_ascii_digit() {
                up as u32
            } else {
                return None;
            }
        }
    };
    Some(vk)
}

/// Human readable name for a captured virtual-key.
pub fn name_of_vk(vk: u32) -> String {
    match vk {
        0x08 => return "Backspace".into(),
        0x09 => return "Tab".into(),
        0x0D => return "Enter".into(),
        0x10 => return "Shift".into(),
        0x11 => return "Control".into(),
        0x12 => return "Alt".into(),
        0x14 => return "CapsLock".into(),
        0x1B => return "Escape".into(),
        0x20 => return "Space".into(),
        0x21 => return "PageUp".into(),
        0x22 => return "PageDown".into(),
        0x23 => return "End".into(),
        0x24 => return "Home".into(),
        0x25 => return "Left".into(),
        0x26 => return "Up".into(),
        0x27 => return "Right".into(),
        0x28 => return "Down".into(),
        0x2D => return "Insert".into(),
        0x2E => return "Delete".into(),
        0x90 => return "NumLock".into(),
        0x91 => return "ScrollLock".into(),
        0xBD => return "-".into(),
        0xBB => return "=".into(),
        0xDB => return "[".into(),
        0xDD => return "]".into(),
        0xDC => return "\\".into(),
        0xBA => return ";".into(),
        0xDE => return "'".into(),
        0xBC => return ",".into(),
        0xBE => return ".".into(),
        0xBF => return "/".into(),
        0xC0 => return "`".into(),
        _ => {}
    }
    if (0x41..=0x5A).contains(&vk) {
        return ((vk as u8 as char).to_ascii_lowercase()).to_string();
    }
    if (0x30..=0x39).contains(&vk) {
        return (vk as u8 as char).to_string();
    }
    if (0x70..=0x87).contains(&vk) {
        return format!("F{}", vk - 0x70 + 1);
    }
    if (0x60..=0x69).contains(&vk) {
        return format!("Numpad{}", vk - 0x60);
    }
    format!("vk{:X}", vk)
}

pub fn display(binding: &str) -> String {
    display_ref(binding).to_string()
}

/// Paint-path variant: no allocation for labels that are already stored.
pub fn display_ref(binding: &str) -> &str {
    if binding.is_empty() {
        "..."
    } else {
        binding
    }
}
