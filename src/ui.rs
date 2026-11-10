//! The panel: a compact, flat, icon-led surface. Every layout number is in
//! logical pixels and multiplied by the window's DPI scale at draw time.

#![allow(dead_code)]

use crate::config::Settings;
use crate::gp::{Canvas, Fonts};

pub const WIN_W: f32 = 624.0;
pub const WIN_H: f32 = 408.0;

pub const HEADER_H: f32 = 44.0;
const RAIL_W: f32 = 100.0;
const FOOTER_H: f32 = 22.0;
const CARD_TITLE: f32 = 22.0;
const ROW_H: f32 = 21.0;
const CARD_PAD: f32 = 5.0;
const GAP: f32 = 8.0;
const PAD: f32 = 10.0;
const COL_W: f32 = (WIN_W - RAIL_W - PAD * 2.0 - GAP) / 2.0;

// Shared domains for the three fast timing modules. The default is 47/53;
// Min can reach 97 and Max can reach 103 without ever exposing 10-200.
const FAST_MIN_LO: i32 = 47;
const FAST_MIN_HI: i32 = 97;
const FAST_MAX_LO: i32 = 53;
const FAST_MAX_HI: i32 = 103;

// ------------------------------------------------------------------ helpers

#[derive(Clone, Copy, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }
    pub fn has(&self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.w && y < self.y + self.h
    }
}

pub const fn with_alpha(c: u32, a: u8) -> u32 {
    (c & 0x00FF_FFFF) | ((a as u32) << 24)
}

pub fn mix(a: u32, b: u32, t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let ch = |sh: u32| {
        let x = ((a >> sh) & 0xFF) as f32;
        let y = ((b >> sh) & 0xFF) as f32;
        ((x + (y - x) * t) as u32) & 0xFF
    };
    let alb = ((a >> 24) & 0xFF) as f32;
    let blb = ((b >> 24) & 0xFF) as f32;
    let al = ((alb + (blb - alb) * t) as u32) & 0xFF;
    (al << 24) | (ch(16) << 16) | (ch(8) << 8) | ch(0)
}

fn wid(s: &str, slot: u32) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325 ^ slot as u64;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x1000_0000_01B3);
    }
    h
}

fn card_mono(title: &str) -> &'static str {
    match title {
        "Shield" => "SH",
        "Mace" => "MC",
        "Breach" => "BR",
        "Potion" => "PT",
        "Auto Use" => "AU",
        "Dash" => "DS",
        "Auto Crystal" => "AC",
        "Anchor" => "AN",
        "Hover Totem" => "HT",
        "Pearl" => "PL",
        "Fast Use" => "FU",
        "W-Tap" => "WT",
        "Sprint" => "SP",
        "Ghost Click" => "GC",
        "Behaviour" => "BH",
        "Session" => "SE",
        "Performance" => "PF",
        "Danger zone" => "DZ",
        "Color" => "CL",
        _ => "",
    }
}

// ------------------------------------------------------------------- theme

pub struct Theme {
    pub bg: u32,
    pub rail: u32,
    pub card: u32,
    pub card_on: u32,
    pub border: u32,
    pub border_on: u32,
    pub text: u32,
    pub dim: u32,
    pub faint: u32,
    pub accent: u32,
    pub on_accent: u32,
    pub chip: u32,
    pub track: u32,
    pub danger: u32,
}

impl Theme {
    pub fn new(s: &Settings) -> Theme {
        let (r, g, b) = s.theme_rgb();
        let accent = 0xFF00_0000 | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
        // Keep switch labels and the knob readable on both dark and light
        // accents. This is a cheap perceptual threshold, not a full WCAG pass.
        let luma = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
        Theme {
            bg: 0xFF0A_0A0B,
            rail: 0xFF0D_0D0F,
            card: 0xFF13_1315,
            card_on: 0xFF19_191C,
            border: with_alpha(accent, 24),
            border_on: with_alpha(accent, 48),
            text: 0xFFF4_F4F5,
            dim: 0xFF9D_9DA3,
            faint: 0xFF66_666C,
            accent,
            on_accent: if luma > 150.0 { 0xFF0A_0A0B } else { 0xFFF4_F4F5 },
            chip: 0xFF1D_1D20,
            track: 0xFF30_3034,
            danger: 0xFFD2_6E6E,
        }
    }
}

// ---------------------------------------------------------------- actions

#[derive(Clone, Copy)]
pub enum Act {
    Tab(usize),
    Toggle(&'static str),
    Capture(&'static str),
    Clear(&'static str),
    SetInt(&'static str, i32),
    SetStr(&'static str, &'static str),
    Mode(&'static str),
    Theme(usize, i32),
    Preset(i32, i32, i32),
    Reset,
    Discord,
    Destruct,
    Hide,
    Close,
}

#[derive(Clone, Copy)]
pub struct Drag {
    pub key: &'static str,
    pub ch: usize,
    pub x: f32,
    pub w: f32,
    pub min: i32,
    pub max: i32,
}

#[derive(Default)]
pub struct Shared {
    pub drag: Option<Drag>,
}

// ---------------------------------------------------------------------- ui

pub struct Ui<'a> {
    pub c: &'a mut Canvas,
    pub f: &'a mut Fonts,
    pub th: Theme,
    pub s: &'a Settings,
    pub tab: usize,
    pub mx: f32,
    pub my: f32,
    pub inside: bool,
    pub down: bool,
    pub pressed: bool,
    pub rpressed: bool,
    pub dt: f32,
    pub scale: f32,
    pub time: f32,
    pub capture: Option<&'static str>,
    pub sd: f32,
    pub game: bool,
    pub focused: bool,
    pub activations: u32,
    pub render_us: u32,
    pub anims: &'a mut std::collections::HashMap<u64, f32>,
    pub shared: &'a mut Shared,
    pub acts: Vec<Act>,
    pub timer_100ns: u32,
    pub priority: &'static str,
}

impl<'a> Ui<'a> {
    // ---- primitives

    fn label(&mut self, x: f32, y: f32, w: f32, h: f32, s: &str, px: f32, bold: bool, color: u32) {
        let sc = self.scale;
        let f = self.f.font(px * sc, bold);
        self.c.text(s, x, y, w, h, f, color, 0);
    }

    fn label_center(&mut self, x: f32, y: f32, w: f32, h: f32, s: &str, px: f32, bold: bool, color: u32) {
        let sc = self.scale;
        let f = self.f.font(px * sc, bold);
        self.c.text(s, x, y, w, h, f, color, 1);
    }

    fn label_right(&mut self, x: f32, y: f32, w: f32, h: f32, s: &str, px: f32, color: u32) {
        let sc = self.scale;
        let f = self.f.font(px * sc, false);
        self.c.text(s, x, y, w, h, f, color, 2);
    }

    fn hovered(&self, r: Rect) -> bool {
        self.inside && r.has(self.mx, self.my)
    }

    fn anim(&mut self, id: u64, target: f32) -> f32 {
        let dt = self.dt;
        let v = self.anims.entry(id).or_insert(target);
        let k = (dt * 16.0).min(1.0);
        *v += (target - *v) * k;
        *v
    }

    fn col(&self, i: usize) -> f32 {
        RAIL_W + PAD + (COL_W + GAP) * i as f32
    }

    fn card_h(&self, rows: usize) -> f32 {
        (CARD_TITLE + ROW_H * rows as f32 + CARD_PAD) * self.scale
    }

    fn row_y(&self, card: Rect, i: usize) -> f32 {
        card.y + (CARD_TITLE + ROW_H * i as f32) * self.scale
    }

    // ---- frame

    pub fn frame(&mut self) {
        let s = self.scale;
        self.c.rect(0.0, 0.0, WIN_W * s, WIN_H * s, self.th.bg);
        self.c.rect(0.0, 0.0, RAIL_W * s, WIN_H * s, self.th.rail);
        self.c.rect(RAIL_W * s, 0.0, 1.0, WIN_H * s, with_alpha(self.th.text, 8));
        self.header();
        self.rail();
        match self.tab {
            0 => self.tab_core(),
            1 => self.tab_crystal(),
            2 => self.tab_assist(),
            _ => self.tab_system(),
        }
        self.footer();
        // Keep the hairline one pixel inside DWM's rounded clip. Drawing it
        // on the exact edge caused the corner to look clipped on some scales.
        self.c.round_stroke(
            1.0,
            1.0,
            WIN_W * s - 2.0,
            WIN_H * s - 2.0,
            12.0 * s,
            with_alpha(self.th.text, 14),
            1.0,
        );
    }

    fn header(&mut self) {
        let s = self.scale;
        // No wordmark: the droplet mark alone carries the brand, centered in
        // the rail. The compact header keeps the title on the same optical axis.
        self.logo((RAIL_W - 22.0) * 0.5 * s, 10.0 * s, 22.0 * s);

        let titles = ["Core", "Crystal", "Assist", "System"];
        self.label(
            (RAIL_W + PAD) * s,
            10.0 * s,
            200.0 * s,
            22.0 * s,
            titles[self.tab],
            16.0,
            false,
            self.th.text,
        );

        let [master, hide, close] = header_buttons(WIN_W, s);
        let armed = self.s.bool_at("v0");
        let hot = self.hovered(master);
        let hv = self.anim(wid("master", 1), if hot { 1.0 } else { 0.0 });
        let bg = with_alpha(self.th.text, (6.0 + 6.0 * hv) as u8);
        let radius = master.h * 0.5;
        self.c.round_rect(master.x, master.y, master.w, master.h, radius, bg);
        let dot = if armed { self.th.accent } else { self.th.faint };
        self.c.circle(master.x + 13.0 * s, master.y + master.h * 0.5, 2.5 * s, dot);
        let fg = if armed { self.th.text } else { self.th.dim };
        self.label(
            master.x + 21.0 * s,
            master.y,
            master.w - 26.0 * s,
            master.h,
            if armed { "Armed" } else { "Safe" },
            9.0,
            false,
            fg,
        );
        if self.pressed && hot {
            self.acts.push(Act::Toggle("v0"));
        }

        self.icon_button(hide, 0, Act::Hide);
        self.icon_button(close, 1, Act::Close);

        self.c.rect(
            RAIL_W * s,
            HEADER_H * s - 1.0,
            (WIN_W - RAIL_W) * s,
            1.0,
            self.th.border,
        );
    }

    fn icon_button(&mut self, r: Rect, glyph: u8, act: Act) {
        let s = self.scale;
        let hot = self.hovered(r);
        let id = if glyph == 0 { "min" } else { "close" };
        let hv = self.anim(wid(id, 18), if hot { 1.0 } else { 0.0 });
        if hv > 0.01 {
            self.c.round_rect(r.x, r.y, r.w, r.h, r.h * 0.5, with_alpha(self.th.text, (12.0 * hv) as u8));
        }
        let color = mix(self.th.faint, self.th.text, hv);
        let cx = r.x + r.w * 0.5;
        let cy = r.y + r.h * 0.5;
        if glyph == 0 {
            self.c.line(cx - 4.0 * s, cy + 0.5 * s, cx + 4.0 * s, cy + 0.5 * s, color, 1.3 * s);
        } else {
            let d = 3.5 * s;
            self.c.line(cx - d, cy - d, cx + d, cy + d, color, 1.3 * s);
            self.c.line(cx - d, cy + d, cx + d, cy - d, color, 1.3 * s);
        }
        if self.pressed && hot {
            self.acts.push(act);
        }
    }

    fn logo(&mut self, x: f32, y: f32, sz: f32) {
        let w = sz * 0.68;
        let h = sz * 0.90;
        let cx = x + sz * 0.5;
        self.c.drop_mark(
            cx,
            y + sz * 0.05,
            w,
            h,
            with_alpha(self.th.accent, 28),
            self.th.accent,
            1.25 * self.scale,
        );
    }

    fn rail(&mut self) {
        let s = self.scale;
        let items = [("Core", 0usize, 0u8), ("Crystal", 1, 1), ("Assist", 2, 2), ("System", 3, 3)];
        let mut y = (HEADER_H + 8.0) * s;
        for (name, idx, kind) in items {
            let r = Rect::new(8.0 * s, y, (RAIL_W - 16.0) * s, 34.0 * s);
            let active = self.tab == idx;
            let hot = self.hovered(r);
            let hv = self.anim(wid(name, 2), if hot { 1.0 } else { 0.0 });
            if active {
                self.c.round_rect(r.x, r.y, r.w, r.h, 9.0 * s, with_alpha(self.th.text, 7));
                self.c.round_rect(r.x, r.y + 10.0 * s, 2.0 * s, r.h - 20.0 * s, 1.0 * s, self.th.accent);
            } else if hot {
                self.c.round_rect(r.x, r.y, r.w, r.h, 9.0 * s, with_alpha(self.th.text, 5));
            }
            let tint = if active { self.th.accent } else { mix(self.th.dim, self.th.text, hv * 0.35) };
            self.icon(kind, r.x + 10.0 * s, r.y + 10.0 * s, 14.0 * s, tint);
            self.label(
                r.x + 30.0 * s,
                r.y,
                r.w - 34.0 * s,
                r.h,
                name,
                9.5,
                false,
                if active { self.th.text } else { tint },
            );
            if self.pressed && hot {
                self.acts.push(Act::Tab(idx));
            }
            y += 36.0 * s;
        }
    }

    fn icon(&mut self, kind: u8, x: f32, y: f32, sz: f32, color: u32) {
        let w = 1.25;
        let cx = x + sz * 0.5;
        match kind {
            0 => {
                // shield
                self.c.line(cx, y + 0.5, x + sz - 0.5, y + sz * 0.30, color, w);
                self.c.line(x + sz - 0.5, y + sz * 0.30, x + sz * 0.84, y + sz * 0.74, color, w);
                self.c.line(x + sz * 0.84, y + sz * 0.74, cx, y + sz - 0.5, color, w);
                self.c.line(cx, y + sz - 0.5, x + sz * 0.16, y + sz * 0.74, color, w);
                self.c.line(x + sz * 0.16, y + sz * 0.74, x + 0.5, y + sz * 0.30, color, w);
                self.c.line(x + 0.5, y + sz * 0.30, cx, y + 0.5, color, w);
            }
            1 => {
                // crystal
                self.c.line(cx, y + 0.5, x + sz * 0.86, y + sz * 0.42, color, w);
                self.c.line(x + sz * 0.86, y + sz * 0.42, cx, y + sz - 0.5, color, w);
                self.c.line(cx, y + sz - 0.5, x + sz * 0.14, y + sz * 0.42, color, w);
                self.c.line(x + sz * 0.14, y + sz * 0.42, cx, y + 0.5, color, w);
                self.c.line(cx, y + 0.5, cx, y + sz - 0.5, with_alpha(color, 120), 1.0);
            }
            2 => {
                // pointer
                self.c.line(x + sz * 0.28, y + 0.5, x + sz * 0.28, y + sz * 0.80, color, w);
                self.c.line(x + sz * 0.28, y + sz * 0.80, x + sz * 0.44, y + sz * 0.64, color, w);
                self.c.line(x + sz * 0.44, y + sz * 0.64, x + sz * 0.55, y + sz * 0.92, color, w);
                self.c.line(x + sz * 0.55, y + sz * 0.92, x + sz * 0.68, y + sz * 0.86, color, w);
                self.c.line(x + sz * 0.68, y + sz * 0.86, x + sz * 0.57, y + sz * 0.60, color, w);
                self.c.line(x + sz * 0.57, y + sz * 0.60, x + sz * 0.78, y + sz * 0.57, color, w);
                self.c.line(x + sz * 0.78, y + sz * 0.57, x + sz * 0.28, y + 0.5, color, w);
            }
            _ => {
                // sliders
                let h2 = y + sz * 0.32;
                let h3 = y + sz * 0.72;
                self.c.line(x + 0.5, h2, x + sz - 0.5, h2, color, w);
                self.c.line(x + 0.5, h3, x + sz - 0.5, h3, color, w);
                self.c.circle(x + sz * 0.68, h2, 2.0, color);
                self.c.circle(x + sz * 0.34, h3, 2.0, color);
            }
        }
    }

    // ---- card chrome

    fn card(&mut self, r: Rect, title: &str, toggle: Option<&'static str>) {
        let s = self.scale;
        let on = toggle.map(|k| self.s.bool_at(k)).unwrap_or(true);
        let active = toggle.is_none() || on;
        self.c.round_rect(r.x, r.y, r.w, r.h, 12.0 * s, if on { self.th.card_on } else { self.th.card });
        self.c.round_stroke(
            r.x,
            r.y,
            r.w,
            r.h,
            12.0 * s,
            if on { self.th.border_on } else { self.th.border },
            1.0,
        );
        let color = if active { self.th.text } else { self.th.dim };
        let mono = card_mono(title);
        let hue = if title == "Danger zone" { self.th.danger } else { self.th.accent };
        let chip = Rect::new(r.x + 12.0 * s, r.y + 5.0 * s, 18.0 * s, 18.0 * s);
        self.c.round_rect(chip.x, chip.y, chip.w, chip.h, chip.w * 0.5, with_alpha(hue, if active { 13 } else { 7 }));
        self.label_center(chip.x, chip.y - 0.5 * s, chip.w, chip.h, mono, 7.5, false, if active { hue } else { self.th.faint });
        let tr = Rect::new(chip.x + chip.w + 8.0 * s, r.y + 4.0 * s, r.w - 12.0 * s - chip.w - 8.0 * s - 44.0 * s, 17.0 * s);
        self.label(tr.x, tr.y, tr.w, tr.h, title, 11.5, false, color);
        if let Some(k) = toggle {
            let sw = Rect::new(r.x + r.w - 12.0 * s - 28.0 * s, r.y + 5.0 * s, 28.0 * s, 16.0 * s);
            self.switch(sw.x, sw.y, k, on);
        }
    }

    fn switch(&mut self, x: f32, y: f32, key: &'static str, on: bool) {
        let s = self.scale;
        let w = 28.0 * s;
        let h = 16.0 * s;
        let r = Rect::new(x, y, w, h);
        let hot = self.hovered(r);
        let t = self.anim(wid(key, 11), if on { 1.0 } else { 0.0 });
        let hv = self.anim(wid(key, 12), if hot { 1.0 } else { 0.0 });
        let off = mix(self.th.track, 0xFF3A_3A3E, hv);
        self.c.round_rect(x, y, w, h, h * 0.5, mix(off, self.th.accent, t));
        let kx = x + 2.0 * s + t * (w - 4.0 * s - 10.0 * s);
        let knob = mix(self.th.text, self.th.on_accent, t);
        self.c.circle(kx + 5.0 * s, y + h * 0.5, 5.0 * s, knob);
        if self.pressed && hot {
            self.acts.push(Act::Toggle(key));
        }
    }

    fn chip(&mut self, r: Rect, key: &'static str) {
        let s = self.scale;
        let binding = self.s.str_at(key);
        let listening = self.capture == Some(key);
        let hot = self.hovered(r);
        let hv = self.anim(wid(key, 13), if hot { 1.0 } else { 0.0 });
        let pulse = 0.5 + 0.5 * (self.time * 5.0).sin();
        let bg = if listening {
            with_alpha(self.th.accent, (16.0 + 10.0 * pulse) as u8)
        } else {
            with_alpha(self.th.text, (5.0 + 6.0 * hv) as u8)
        };
        self.c.round_rect(r.x, r.y, r.w, r.h, r.h * 0.5, bg);
        let text = if listening { "press…".to_string() } else { crate::keys::display(&binding) };
        let color = if listening { self.th.accent } else { self.th.text };
        // Long binding names must not bleed into the mode chip beside them.
        self.c.clip(r.x, r.y, r.w, r.h);
        self.label_center(r.x, r.y - 0.5 * s, r.w, r.h, &text, 9.0, false, color);
        self.c.reset_clip();
        if self.pressed && hot {
            self.acts.push(Act::Capture(key));
        }
        if self.rpressed && hot {
            self.acts.push(Act::Clear(key));
        }
    }

    // ---- rows

    fn row_key(&mut self, card: Rect, i: usize, label: &str, key: &'static str, mode: Option<&'static str>) {
        let s = self.scale;
        let y = self.row_y(card, i);
        let mode_w = if mode.is_some() { 42.0 * s } else { 0.0 };
        let gap = if mode.is_some() { 4.0 * s } else { 0.0 };
        let key_w = (card.w - 12.0 * s - 10.0 * s - mode_w - gap).min(66.0 * s);
        let kx = card.x + card.w - 10.0 * s - mode_w - gap - key_w;
        // Keep the label inside the space left of the key chips, never under them.
        let label_w = (kx - (card.x + 12.0 * s) - 6.0 * s).max(0.0);
        self.label(card.x + 12.0 * s, y, label_w, ROW_H * s, label, 9.5, false, self.th.dim);
        self.chip(Rect::new(kx, y + 2.5 * s, key_w, 17.0 * s), key);
        if let Some(mk) = mode {
            self.mode_chip(Rect::new(kx + key_w + gap, y + 2.5 * s, mode_w, 17.0 * s), mk);
        }
    }

    fn mode_chip(&mut self, r: Rect, key: &'static str) {
        let s = self.scale;
        let toggle = self.s.mode_is_toggle(key);
        let live = toggle && crate::macros::toggle_active(crate::macros::mode_module(key));
        let hot = self.hovered(r);
        let hv = self.anim(wid(key, 14), if hot { 1.0 } else { 0.0 });
        let pulse = 0.5 + 0.5 * (self.time * 5.0).sin();
        if live {
            self.c.round_rect(r.x, r.y, r.w, r.h, r.h * 0.5, with_alpha(self.th.accent, (18.0 + 14.0 * pulse) as u8));
        } else if toggle {
            self.c.round_rect(r.x, r.y, r.w, r.h, r.h * 0.5, with_alpha(self.th.accent, 9));
        } else if hv > 0.01 {
            self.c.round_rect(r.x, r.y, r.w, r.h, r.h * 0.5, with_alpha(self.th.text, (4.0 * hv) as u8));
        }
        let color = if toggle { self.th.accent } else { mix(self.th.dim, self.th.text, hv) };
        let text = if live { "On" } else if toggle { "Toggle" } else { "Hold" };
        self.label_center(r.x, r.y - 0.5 * s, r.w, r.h, text, 7.5, false, color);
        if live {
            self.c.circle(r.x + r.w - 4.0 * s, r.y + r.h * 0.5, 1.6 * s, with_alpha(self.th.accent, 220));
        }
        if self.pressed && hot {
            self.acts.push(Act::Mode(key));
        }
    }

    fn row_slot(&mut self, card: Rect, i: usize, label: &str, key: &'static str) {
        let s = self.scale;
        let y = self.row_y(card, i);
        self.label(card.x + 12.0 * s, y, card.w - 100.0 * s, ROW_H * s, label, 9.5, false, self.th.dim);
        let w = 66.0 * s;
        self.chip(Rect::new(card.x + card.w - 10.0 * s - w, y + 2.5 * s, w, 17.0 * s), key);
    }

    fn row_switch(&mut self, card: Rect, i: usize, label: &str, key: &'static str) {
        let s = self.scale;
        let y = self.row_y(card, i);
        let on = self.s.bool_at(key);
        let lr = Rect::new(card.x + 12.0 * s, y, card.w - 58.0 * s, ROW_H * s);
        self.label(lr.x, lr.y, lr.w, lr.h, label, 9.5, false, self.th.dim);
        let sw = Rect::new(card.x + card.w - 12.0 * s - 28.0 * s, y + 3.0 * s, 28.0 * s, 16.0 * s);
        self.switch(sw.x, sw.y, key, on);
    }

    fn row_slider(&mut self, card: Rect, i: usize, label: &str, key: &'static str, min: i32, max: i32, suffix: &str) {
        let s = self.scale;
        let y = self.row_y(card, i);
        let v = self.s.int_at(key).clamp(min, max);
        self.label(card.x + 12.0 * s, y, 72.0 * s, ROW_H * s, label, 9.5, false, self.th.dim);
        let tx = card.x + 84.0 * s;
        let tw = card.w - 84.0 * s - 42.0 * s;
        let cy = y + ROW_H * 0.5 * s;
        let t = if max > min { (v - min) as f32 / (max - min) as f32 } else { 0.0 };
        self.c.round_rect(tx, cy - 1.0 * s, tw, 2.0 * s, 1.0 * s, self.th.track);
        self.c.round_rect(tx, cy - 1.0 * s, tw * t, 2.0 * s, 1.0 * s, self.th.accent);
        self.c.circle(tx + tw * t, cy, 4.0 * s, self.th.text);
        self.label_right(
            card.x + card.w - 10.0 * s - 36.0 * s,
            y,
            36.0 * s,
            ROW_H * s,
            &format!("{}{}", v, suffix),
            8.5,
            self.th.dim,
        );
        let hit = Rect::new(tx - 8.0 * s, y, tw + 16.0 * s, ROW_H * s);
        if self.pressed && self.hovered(hit) {
            self.shared.drag = Some(Drag { key, ch: 9, x: tx, w: tw, min, max });
        }
        if let Some(d) = self.shared.drag {
            if d.key == key && self.down {
                let rel = ((self.mx - d.x) / d.w).clamp(0.0, 1.0);
                let nv = d.min + ((d.max - d.min) as f32 * rel).round() as i32;
                if nv != v {
                    self.acts.push(Act::SetInt(key, nv));
                }
            }
        }
    }

    fn row_slider3(
        &mut self,
        card: Rect,
        i: usize,
        label: &str,
        key: &'static str,
        values: [i32; 3],
        suffix: &str,
    ) {
        let s = self.scale;
        let y = self.row_y(card, i);
        let v = self.s.int_at(key).clamp(1, 3);
        self.label(card.x + 12.0 * s, y, 72.0 * s, ROW_H * s, label, 9.5, false, self.th.dim);
        let tx = card.x + 84.0 * s;
        let tw = card.w - 84.0 * s - 42.0 * s;
        let cy = y + ROW_H * 0.5 * s;
        let t = (v - 1) as f32 * 0.5;
        self.c.round_rect(tx, cy - 1.0 * s, tw, 2.0 * s, 1.0 * s, self.th.track);
        self.c.round_rect(tx, cy - 1.0 * s, tw * t, 2.0 * s, 1.0 * s, self.th.accent);
        for n in 0..3 {
            let x = tx + tw * (n as f32 * 0.5);
            let selected = n + 1 == v;
            self.c.circle(
                x,
                cy,
                if selected { 4.0 * s } else { 2.4 * s },
                if selected { self.th.text } else { self.th.faint },
            );
        }
        self.label_right(
            card.x + card.w - 10.0 * s - 36.0 * s,
            y,
            36.0 * s,
            ROW_H * s,
            &format!("{}{}", values[(v - 1) as usize], suffix),
            8.5,
            self.th.dim,
        );
        let hit = Rect::new(tx - 8.0 * s, y, tw + 16.0 * s, ROW_H * s);
        if self.pressed && self.hovered(hit) {
            self.shared.drag = Some(Drag { key, ch: 10, x: tx, w: tw, min: 1, max: 3 });
        }
        if let Some(d) = self.shared.drag {
            if d.key == key && self.down {
                let rel = ((self.mx - d.x) / d.w).clamp(0.0, 1.0);
                let nv = 1 + (rel * 2.0).round() as i32;
                if nv != v {
                    self.acts.push(Act::SetInt(key, nv));
                }
            }
        }
    }

    fn row_seg(&mut self, card: Rect, i: usize, label: &str, key: &'static str, options: [&'static str; 2]) {
        let s = self.scale;
        let y = self.row_y(card, i);
        let cur = self.s.str_at(key);
        self.label(card.x + 12.0 * s, y, 72.0 * s, ROW_H * s, label, 9.5, false, self.th.dim);
        let total = 100.0 * s;
        let x = card.x + card.w - 10.0 * s - total;
        self.c.round_rect(x, y + 2.5 * s, total, 17.0 * s, 8.5 * s, with_alpha(self.th.text, 4));
        let half = total * 0.5;
        for (n, opt) in options.iter().enumerate() {
            let r = Rect::new(x + half * n as f32, y + 2.5 * s, half, 17.0 * s);
            let on = cur == *opt;
            if on {
                self.c.round_rect(r.x + 1.5 * s, r.y + 1.5 * s, r.w - 3.0 * s, r.h - 3.0 * s, (r.h - 3.0 * s) * 0.5, with_alpha(self.th.accent, 13));
            }
            let color = if on { self.th.accent } else { self.th.faint };
            let hot = self.hovered(r);
            let c2 = if hot && !on { self.th.dim } else { color };
            self.label_center(r.x, r.y - 0.5 * s, r.w, r.h, opt, 8.5, false, c2);
            if self.pressed && hot {
                self.acts.push(Act::SetStr(key, *opt));
            }
        }
    }

    fn row_presets(&mut self, card: Rect, i: usize) {
        let s = self.scale;
        let y = self.row_y(card, i);
        let (cr, cg, cb) = self.s.theme_rgb();
        let presets: [(u32, i32, i32, i32); 6] = [
            (0xFFC8_C8FF, 200, 200, 255),
            (0xFF8F_DAFF, 143, 218, 255),
            (0xFF8C_F5C8, 140, 245, 200),
            (0xFFFF_D6_8A, 255, 214, 138),
            (0xFFFF_9A_9A, 255, 154, 154),
            (0xFFE6_E8EF, 230, 232, 239),
        ];
        let sw = 14.0 * s;
        let gap = 4.0 * s;
        let total = sw * presets.len() as f32 + gap * (presets.len() as f32 - 1.0);
        let left = card.x + card.w - 10.0 * s - total;
        let label_w = (left - (card.x + 12.0 * s) - 6.0 * s).max(0.0);
        self.label(card.x + 12.0 * s, y, label_w, ROW_H * s, "Presets", 9.5, false, self.th.dim);
        let mut x = left;
        for (color, r, g, b) in presets {
            let rr = Rect::new(x, y + 4.0 * s, sw, sw);
            let hot = self.hovered(rr);
            let selected = cr as i32 == r && cg as i32 == g && cb as i32 == b;
            self.c.round_rect(rr.x, rr.y, rr.w, rr.h, rr.w * 0.5, color);
            if selected {
                self.c.round_stroke(rr.x - 1.5 * s, rr.y - 1.5 * s, rr.w + 3.0 * s, rr.h + 3.0 * s, (rr.w + 3.0 * s) * 0.5, with_alpha(self.th.text, 200), 1.1);
            } else if hot {
                self.c.round_stroke(rr.x - 1.0 * s, rr.y - 1.0 * s, rr.w + 2.0 * s, rr.h + 2.0 * s, (rr.w + 2.0 * s) * 0.5, with_alpha(self.th.text, 150), 1.0);
            }
            if self.pressed && hot {
                self.acts.push(Act::Preset(r, g, b));
            }
            x += sw + gap;
        }
    }

    fn row_theme(&mut self, card: Rect, i: usize, label: &str, ch: usize, value: i32) {
        let s = self.scale;
        let y = self.row_y(card, i);
        let v = value.clamp(0, 255);
        let color = match ch {
            0 => 0xFFE0_6A6A,
            1 => 0xFF6A_D99A,
            _ => 0xFF6A_9B_E0,
        };
        self.label(card.x + 12.0 * s, y, 72.0 * s, ROW_H * s, label, 9.5, false, self.th.dim);
        let tx = card.x + 84.0 * s;
        let tw = card.w - 84.0 * s - 42.0 * s;
        let cy = y + ROW_H * 0.5 * s;
        let t = v as f32 / 255.0;
        self.c.round_rect(tx, cy - 1.0 * s, tw, 2.0 * s, 1.0 * s, self.th.track);
        self.c.round_rect(tx, cy - 1.0 * s, tw * t, 2.0 * s, 1.0 * s, color);
        self.c.circle(tx + tw * t, cy, 4.0 * s, self.th.text);
        self.label_right(card.x + card.w - 10.0 * s - 36.0 * s, y, 36.0 * s, ROW_H * s, &v.to_string(), 8.5, self.th.dim);
        let hit = Rect::new(tx - 8.0 * s, y, tw + 16.0 * s, ROW_H * s);
        if self.pressed && self.hovered(hit) {
            self.shared.drag = Some(Drag { key: "thmC", ch, x: tx, w: tw, min: 0, max: 255 });
        }
        if let Some(d) = self.shared.drag {
            if d.key == "thmC" && d.ch == ch && self.down {
                let rel = ((self.mx - d.x) / d.w).clamp(0.0, 1.0);
                let nv = d.min + ((d.max - d.min) as f32 * rel).round() as i32;
                if nv != v {
                    self.acts.push(Act::Theme(ch, nv));
                }
            }
        }
    }

    fn row_button(&mut self, card: Rect, i: usize, text: &str, act: Act) {
        let s = self.scale;
        let y = self.row_y(card, i);
        let w = 90.0 * s;
        let r = Rect::new(card.x + 12.0 * s, y + 2.5 * s, w, 17.0 * s);
        let hot = self.hovered(r);
        let primary = matches!(act, Act::Discord);
        let bg = if primary {
            with_alpha(self.th.accent, if hot { 22 } else { 14 })
        } else {
            with_alpha(self.th.text, if hot { 11 } else { 6 })
        };
        self.c.round_rect(r.x, r.y, r.w, r.h, r.h * 0.5, bg);
        let fg = if primary { self.th.accent } else if hot { self.th.text } else { self.th.dim };
        self.label_center(r.x, r.y - 0.5 * s, r.w, r.h, text, 9.0, false, fg);
        if self.pressed && hot {
            self.acts.push(act);
        }
    }

    fn row_value(&mut self, card: Rect, i: usize, label: &str, value: &str) {
        let s = self.scale;
        let y = self.row_y(card, i);
        // 12 left + 80 value + 12 right + 6 breathing room.
        self.label(card.x + 12.0 * s, y, card.w - 110.0 * s, ROW_H * s, label, 9.5, false, self.th.dim);
        self.label_right(card.x + card.w - 12.0 * s - 80.0 * s, y, 80.0 * s, ROW_H * s, value, 9.5, self.th.text);
    }

    fn row_danger(&mut self, card: Rect, i: usize, text: &str) {
        let s = self.scale;
        let y = self.row_y(card, i);
        let r = Rect::new(card.x + 12.0 * s, y + 2.5 * s, 108.0 * s, 17.0 * s);
        let hot = self.hovered(r);
        let bg = with_alpha(self.th.danger, if hot { 26 } else { 15 });
        self.c.round_rect(r.x, r.y, r.w, r.h, r.h * 0.5, bg);
        self.label_center(r.x, r.y - 0.5 * s, r.w, r.h, text, 9.0, false, self.th.danger);
        if self.pressed && hot {
            self.acts.push(Act::Destruct);
        }
    }

    // ---- tabs

    fn tab_core(&mut self) {
        let s = self.scale;
        let gap = GAP * s;
        let mut y = (HEADER_H + PAD) * s;
        let (x0, x1) = (self.col(0) * s, self.col(1) * s);
        let w = COL_W * s;

        // Nine rows per column keeps the four-row Dash card inside the panel.
        let r = Rect::new(x0, y, w, self.card_h(3));
        self.card(r, "Shield", Some("v1"));
        self.row_key(r, 0, "Trigger", "v1Bn", Some("v1Md"));
        self.row_slot(r, 1, "Axe slot", "v1S1");
        self.row_switch(r, 2, "Stun lock", "vF");
        y += r.h + gap;

        let r = Rect::new(x0, y, w, self.card_h(3));
        self.card(r, "Mace", Some("v2"));
        self.row_key(r, 0, "Trigger", "v2Bn", Some("v2Md"));
        self.row_slot(r, 1, "Mace slot", "v2S1");
        self.row_slider3(r, 2, "Input delay", "v2D", [50, 100, 150], "ms");
        y += r.h + gap;

        let r = Rect::new(x0, y, w, self.card_h(4));
        self.card(r, "Potion", Some("v4"));
        self.row_key(r, 0, "Trigger", "v4Bn", Some("v4Md"));
        self.row_slot(r, 1, "Potion I", "v4S1");
        self.row_slot(r, 2, "Potion II", "v4S2");
        self.row_slot(r, 3, "Return", "v4S3");

        let mut y = (HEADER_H + PAD) * s;
        let r = Rect::new(x1, y, w, self.card_h(3));
        self.card(r, "Breach", Some("v3"));
        self.row_key(r, 0, "Trigger", "v3Bn", Some("v3Md"));
        self.row_slot(r, 1, "Mace", "v3S1");
        self.row_slot(r, 2, "Return", "v3S2");
        y += r.h + gap;

        let r = Rect::new(x1, y, w, self.card_h(2));
        self.card(r, "Auto Use", Some("vW"));
        self.row_key(r, 0, "Trigger", "vWBn", Some("vWMd"));
        self.row_slot(r, 1, "Item", "vWS1");
        y += r.h + gap;

        let r = Rect::new(x1, y, w, self.card_h(5));
        self.card(r, "Dash", Some("vD"));
        self.row_key(r, 0, "Trigger", "vDBn", Some("vDMd"));
        self.row_slot(r, 1, "Spear", "vDS1");
        self.row_slot(r, 2, "Return", "vDS2");
        self.row_slider3(r, 3, "Cooldown", "vDCd", [50, 100, 150], "ms");
        self.row_value(r, 4, "Clicks", "1x");
    }

    fn tab_crystal(&mut self) {
        let s = self.scale;
        let gap = GAP * s;
        let w = COL_W * s;
        let (x0, x1) = (self.col(0) * s, self.col(1) * s);
        let mut y = (HEADER_H + PAD) * s;

        let r = Rect::new(x0, y, w, self.card_h(5));
        self.card(r, "Auto Crystal", Some("v5"));
        self.row_key(r, 0, "Trigger", "v5Bn", Some("v5Md"));
        self.row_slot(r, 1, "Obsidian", "v5S1");
        self.row_slot(r, 2, "Crystal", "v5S2");
        self.row_slider(r, 3, "Min delay", "v5D1", FAST_MIN_LO, FAST_MIN_HI, "ms");
        self.row_slider(r, 4, "Max delay", "v5D2", FAST_MAX_LO, FAST_MAX_HI, "ms");
        y += r.h + gap;

        let r = Rect::new(x0, y, w, self.card_h(4));
        self.card(r, "Hover Totem", Some("v8"));
        self.row_key(r, 0, "Trigger", "v8Bn", Some("v8Md"));
        self.row_slot(r, 1, "Totem", "v8Sp");
        self.row_slot(r, 2, "Hotbar", "v8T");
        self.row_switch(r, 3, "Auto inventory", "vI");

        let mut y = (HEADER_H + PAD) * s;
        let r = Rect::new(x1, y, w, self.card_h(6));
        self.card(r, "Anchor", Some("v6"));
        self.row_key(r, 0, "Trigger", "v6Bn", Some("v6Md"));
        self.row_slot(r, 1, "Anchor", "v6S1");
        self.row_slot(r, 2, "Glowstone", "v6S2");
        self.row_slot(r, 3, "Totem", "v6S3");
        self.row_slider(r, 4, "Min delay", "v6D1", FAST_MIN_LO, FAST_MIN_HI, "ms");
        self.row_slider(r, 5, "Max delay", "v6D2", FAST_MAX_LO, FAST_MAX_HI, "ms");
        y += r.h + gap;

        let r = Rect::new(x1, y, w, self.card_h(4));
        self.card(r, "Pearl", Some("v7"));
        self.row_key(r, 0, "Trigger", "v7Bn", Some("v7Md"));
        self.row_slot(r, 1, "Pearl", "v7S1");
        self.row_slot(r, 2, "Catch", "v7S3");
        self.row_switch(r, 3, "Catch on hold", "v9");
    }

    fn tab_assist(&mut self) {
        let s = self.scale;
        let gap = GAP * s;
        let w = COL_W * s;
        let (x0, x1) = (self.col(0) * s, self.col(1) * s);
        let mut y = (HEADER_H + PAD) * s;

        let r = Rect::new(x0, y, w, self.card_h(3));
        self.card(r, "Fast Use", Some("vA"));
        self.row_key(r, 0, "Trigger", "vABn", Some("vABMd"));
        self.row_slider(r, 1, "Min delay", "vAD1", FAST_MIN_LO, FAST_MIN_HI, "ms");
        self.row_slider(r, 2, "Max delay", "vAD2", FAST_MAX_LO, FAST_MAX_HI, "ms");
        y += r.h + gap;

        let r = Rect::new(x0, y, w, self.card_h(5));
        self.card(r, "W-Tap", Some("vC"));
        self.row_key(r, 0, "Attack", "vCBn", None);
        self.row_seg(r, 1, "Mode", "vM", ["W", "S"]);
        self.row_slider(r, 2, "Min delay", "vCD1", 50, 300, "ms");
        self.row_slider(r, 3, "Max delay", "vCD2", 50, 300, "ms");
        self.row_switch(r, 4, "Smart AI", "vAI");

        let mut y = (HEADER_H + PAD) * s;
        let r = Rect::new(x1, y, w, self.card_h(1));
        self.card(r, "Sprint", Some("vB"));
        self.row_slot(r, 0, "Sprint key", "vBBn");
        y += r.h + gap;

        let r = Rect::new(x1, y, w, self.card_h(1));
        self.card(r, "Ghost Click", Some("vG"));
        self.row_slider(r, 0, "Offset", "vGMx", 1, 12, "px");
    }

    fn tab_system(&mut self) {
        let s = self.scale;
        let gap = GAP * s;
        let w = COL_W * s;
        let (x0, x1) = (self.col(0) * s, self.col(1) * s);
        let mut y = (HEADER_H + PAD) * s;

        let r = Rect::new(x0, y, w, self.card_h(4));
        self.card(r, "Color", None);
        self.row_presets(r, 0);
        let (red, green, blue) = self.s.theme_rgb();
        self.row_theme(r, 1, "Red", 0, red as i32);
        self.row_theme(r, 2, "Green", 1, green as i32);
        self.row_theme(r, 3, "Blue", 2, blue as i32);
        y += r.h + gap;

        let r = Rect::new(x0, y, w, self.card_h(4));
        self.card(r, "Behaviour", None);
        self.row_switch(r, 0, "Keep panel open", "vP");
        self.row_switch(r, 1, "Low latency mode", "vLL");
        self.row_switch(r, 2, "Only in Minecraft", "vFO");
        self.row_value(r, 3, "Capture", "Visible");

        let mut y = (HEADER_H + PAD) * s;
        let r = Rect::new(x1, y, w, self.card_h(3));
        self.card(r, "Session", None);
        self.row_value(r, 0, "Macro runs", &format!("{}", self.activations));
        self.row_button(r, 1, "Discord", Act::Discord);
        self.row_button(r, 2, "Reset config", Act::Reset);
        y += r.h + gap;

        let r = Rect::new(x1, y, w, self.card_h(1));
        self.card(r, "Danger zone", None);
        self.row_danger(r, 0, "Self-destruct");
        y += r.h + gap;

        let r = Rect::new(x1, y, w, self.card_h(4));
        self.card(r, "Performance", None);
        self.row_value(r, 0, "Priority", self.priority);
        let render = if self.render_us == 0 {
            "—".to_string()
        } else {
            format!("{:.2} ms", self.render_us as f32 / 1000.0)
        };
        self.row_value(r, 1, "Render", &render);
        let timer = if self.s.bool_at("vLL") {
            format!("{:.1} ms", self.timer_100ns as f32 / 10_000.0)
        } else {
            "15.6 ms".to_string()
        };
        self.row_value(r, 2, "Timer", &timer);
        self.row_value(r, 3, "AI brain", if self.s.bool_at("vAI") { "Smart" } else { "Manual" });
    }

    fn footer(&mut self) {
        let s = self.scale;
        let y = (WIN_H - FOOTER_H) * s;
        self.c.rect(RAIL_W * s, y, (WIN_W - RAIL_W) * s, 1.0, with_alpha(self.th.text, 8));
        let armed = self.s.bool_at("v0");
        let dot = if armed { self.th.accent } else { self.th.faint };
        self.c.circle(RAIL_W * s + 15.0 * s, y + FOOTER_H * 0.5 * s, 2.0 * s, dot);
        let n = self.on_count();
        let status = format!("{} · {} on", if armed { "Armed" } else { "Safe" }, n);
        self.label(
            RAIL_W * s + 23.0 * s,
            y + 4.0 * s,
            150.0 * s,
            16.0 * s,
            &status,
            8.5,
            false,
            if armed { self.th.dim } else { self.th.faint },
        );
        if self.sd > 0.0 {
            let label = format!("self-destruct {:.0}%", self.sd * 100.0);
            self.label_center(RAIL_W * s, y + 4.0 * s, (WIN_W - RAIL_W) * s, 16.0 * s, &label, 8.5, false, self.th.danger);
            self.c.rect(0.0, WIN_H * s - 2.0 * s, WIN_W * s * self.sd, 2.0 * s, self.th.danger);
            return;
        }
        let (game_dot, game_text, game_color) = if self.focused {
            (self.th.accent, "Game focused", self.th.dim)
        } else if self.game {
            (self.th.dim, "Game detected", self.th.dim)
        } else {
            (self.th.faint, "Waiting for Minecraft", self.th.faint)
        };
        // Dock the dot to the measured label so the gap stays constant for
        // "Game focused", "Game detected" and "Waiting for Minecraft".
        let game_font = self.f.font(8.5 * s, false);
        let game_w = self.c.measure(game_text, game_font);
        let game_x = WIN_W * s - 12.0 * s - game_w;
        let dot_x = game_x - 8.0 * s;
        self.c.circle(dot_x, y + FOOTER_H * 0.5 * s, 2.0 * s, with_alpha(game_dot, 220));
        self.c.text(
            game_text,
            game_x,
            y + 4.0 * s,
            game_w.max(1.0),
            16.0 * s,
            game_font,
            game_color,
            0,
        );
    }

    fn on_count(&self) -> u32 {
        let keys = ["v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "vA", "vB", "vC", "vD", "vW"];
        keys.iter().filter(|k| self.s.bool_at(k)).count() as u32
    }
}

// ------------------------------------------------------------- header hit

pub fn header_buttons(w: f32, sc: f32) -> [Rect; 3] {
    let close = Rect::new(w - 10.0 * sc - 20.0 * sc, 12.0 * sc, 20.0 * sc, 20.0 * sc);
    let hide = Rect::new(close.x - 4.0 * sc - 20.0 * sc, 12.0 * sc, 20.0 * sc, 20.0 * sc);
    let master = Rect::new(hide.x - 8.0 * sc - 72.0 * sc, 11.0 * sc, 72.0 * sc, 22.0 * sc);
    [master, hide, close]
}
