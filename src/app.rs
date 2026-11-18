//! Application shell: windows, global input hooks, focus watching, rendering
//! and the event loop.

#![allow(dead_code)]

use std::collections::HashMap;
use std::ffi::c_void;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::thread;
use std::time::Duration;

use crate::config::{self, Settings};
use crate::gp::{self, BackBuffer, Canvas, Fonts};
use crate::input::{self, Probe};
use crate::keys::{self, Btn, Ev};
use crate::macros;
use crate::ui::{self, Act, Shared, Theme, Ui};
use crate::win::{self, *};

// --------------------------------------------------------------- engine

pub struct Engine {
    st: RwLock<Arc<Settings>>,
    triggers: RwLock<Arc<macros::TriggerMap>>,
    pub focus: AtomicBool,
    pub present: AtomicBool,
    pub gate: AtomicBool,
    focus_only: AtomicBool,
    sprint_on: AtomicBool,
    totem_on: AtomicBool,
    pub capture_key: Mutex<Option<&'static str>>,
    pub captured: Mutex<Option<(&'static str, String)>>,
    pub quitting: AtomicBool,
}

impl Engine {
    fn new(s: &Settings) -> Engine {
        Engine {
            st: RwLock::new(Arc::new(s.clone())),
            triggers: RwLock::new(Arc::new(macros::TriggerMap::build(s))),
            focus: AtomicBool::new(false),
            present: AtomicBool::new(false),
            gate: AtomicBool::new(false),
            focus_only: AtomicBool::new(s.focus_only),
            sprint_on: AtomicBool::new(s.master && s.sprint),
            totem_on: AtomicBool::new(s.master && s.totem),
            capture_key: Mutex::new(None),
            captured: Mutex::new(None),
            quitting: AtomicBool::new(false),
        }
    }
    pub fn settings(&self) -> Arc<Settings> {
        self.st.read().unwrap().clone()
    }
    pub fn triggers(&self) -> Arc<macros::TriggerMap> {
        self.triggers.read().unwrap().clone()
    }
    /// Cheap hit test for the hook path. Most key events do not match a
    /// binding, so resolve that before cloning the settings Arc for a macro.
    #[inline]
    pub fn has_trigger(&self, ev: Ev) -> bool {
        self.triggers.read().unwrap().get(ev).is_some()
    }
    fn publish(&self, s: Settings) {
        // Bindings can change from the panel. Rebuild the event index once per
        // settings publish, never on the input-hook hot path.
        *self.triggers.write().unwrap() = Arc::new(macros::TriggerMap::build(&s));
        self.focus_only.store(s.focus_only, Ordering::Relaxed);
        self.sprint_on
            .store(s.master && s.sprint, Ordering::Relaxed);
        self.totem_on.store(s.master && s.totem, Ordering::Relaxed);
        wake_sprint();
        *self.st.write().unwrap() = Arc::new(s);
    }
}

static ENGINE: OnceLock<Arc<Engine>> = OnceLock::new();
static mut APP: *mut App = std::ptr::null_mut();
static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);
static GAME: AtomicIsize = AtomicIsize::new(0);
static HOOK_TID: AtomicU32 = AtomicU32::new(0);
static SPRINT_WAIT: OnceLock<thread::Thread> = OnceLock::new();

fn wake_sprint() {
    if let Some(t) = SPRINT_WAIT.get() {
        t.unpark();
    }
}

pub fn main_hwnd() -> HWND {
    MAIN_HWND.load(Ordering::Relaxed)
}

static RENDER_US: AtomicU32 = AtomicU32::new(0);
static SD_ACTIVE: AtomicBool = AtomicBool::new(false);
static SD_PROGRESS: AtomicU32 = AtomicU32::new(0);
static TIMER_100NS: AtomicU32 = AtomicU32::new(156_250);
static TIMER_ON: AtomicBool = AtomicBool::new(false);
static TIMER_GUARD_STARTED: AtomicBool = AtomicBool::new(false);
static REALTIME_ACTIVE: AtomicBool = AtomicBool::new(false);

thread_local! {
    static DOWN_KEYS: std::cell::RefCell<[bool; 256]> = std::cell::RefCell::new([false; 256]);
}

// --------------------------------------------------------------- app state

struct App {
    hwnd: HWND,
    dpi: f32,
    engine: Arc<Engine>,
    settings: Settings,
    fonts: Option<Fonts>,

    tab: usize,
    mx: f32,
    my: f32,
    inside: bool,
    down: bool,
    pressed: bool,
    rpressed: bool,

    anims: HashMap<u64, f32>,
    shared: Shared,
    capture: Option<&'static str>,
    capture_at: u64,
    anim_until: u64,
    last_frame: i64,
    last_move: u64,
    anim_timer: bool,

    drag: Option<(i32, i32)>,
    back: Option<BackBuffer>,
    canvas: Option<Canvas>,
    shown: bool,
    user_hidden: bool,
    destructed: bool,
    applied_low: bool,
}

impl App {
    fn new(hwnd: HWND, dpi: f32, engine: Arc<Engine>, settings: Settings, fonts: Fonts) -> App {
        App {
            hwnd,
            dpi,
            applied_low: settings.low_latency,
            engine,
            settings,
            fonts: Some(fonts),
            tab: 0,
            mx: -1.0,
            my: -1.0,
            inside: false,
            down: false,
            pressed: false,
            rpressed: false,
            anims: HashMap::new(),
            shared: Shared::default(),
            capture: None,
            capture_at: 0,
            anim_until: 0,
            last_frame: input::now(),
            last_move: 0,
            anim_timer: false,
            drag: None,
            back: None,
            canvas: None,
            shown: false,
            user_hidden: false,
            destructed: false,
        }
    }

    fn ticks() -> u64 {
        unsafe { win::GetTickCount64() }
    }

    fn client(&self) -> (i32, i32) {
        let mut r = RECT::default();
        unsafe { win::GetClientRect(self.hwnd, &mut r) };
        (r.width().max(0), r.height().max(0))
    }

    fn invalidate(&self) {
        unsafe { win::InvalidateRect(self.hwnd, std::ptr::null(), 0) };
    }

    fn kick_anim(&mut self) {
        self.anim_until = Self::ticks() + 460;
        if !self.anim_timer {
            self.anim_timer = unsafe { win::SetTimer(self.hwnd, 1, 16, 0) } != 0;
        }
    }

    fn dt_value(&mut self) -> f32 {
        let now = input::now();
        let dt = (now - self.last_frame) as f32 / input::qpf() as f32;
        self.last_frame = now;
        dt.clamp(0.0, 0.08)
    }

    // ------------------------------------------------------------- drawing

    fn draw(&mut self, hdc: HDC, interact: bool) {
        // Kept for the BeginPaint/GetDC callers; the layered update below owns
        // the visible pixels.
        let _ = hdc;
        let (cw, ch) = self.client();
        if cw <= 8 || ch <= 8 {
            return;
        }
        if self.back.as_ref().map(|b| (b.w, b.h)) != Some((cw, ch)) {
            // The Graphics object is bound to the DIB's HDC; drop it before the
            // old buffer. Reusing it across paints removes three GDI+ object
            // allocations and their cleanup from every frame.
            self.canvas = None;
            self.back = BackBuffer::new(cw, ch);
        }
        let bdc = match self.back.as_ref() {
            Some(b) => b.dc,
            None => return,
        };
        if self.canvas.is_none() {
            self.canvas = Canvas::new(bdc);
        }
        let render_start = input::now();
        let dt = self.dt_value();
        let scale = self.dpi;
        let time = (Self::ticks() as f32) / 1000.0;
        let sd = SD_PROGRESS.load(Ordering::Relaxed) as f32 / 1000.0;
        let acts;

        {
            let theme = Theme::new(&self.settings);
            let fonts = match self.fonts.as_mut() {
                Some(f) => f,
                None => return,
            };
            let canvas = match self.canvas.as_mut() {
                Some(c) => c,
                None => return,
            };
            let mut ui = Ui {
                c: canvas,
                f: fonts,
                th: theme,
                s: &self.settings,
                tab: self.tab,
                mx: self.mx,
                my: self.my,
                inside: self.inside,
                down: self.down,
                pressed: self.pressed,
                rpressed: self.rpressed,
                dt,
                scale,
                time,
                capture: self.capture,
                sd,
                game: self.engine.present.load(Ordering::Relaxed),
                focused: self.engine.focus.load(Ordering::Relaxed),
                activations: macros::RUNS.load(Ordering::Relaxed),
                render_us: RENDER_US.load(Ordering::Relaxed),
                anims: &mut self.anims,
                shared: &mut self.shared,
                acts: Vec::new(),
                timer_100ns: TIMER_100NS.load(Ordering::Relaxed),
                priority: if REALTIME_ACTIVE.load(Ordering::Relaxed) {
                    "Realtime"
                } else {
                    "High"
                },
            };
            ui.frame();
            acts = std::mem::take(&mut ui.acts);
        }

        unsafe {
            // A layered window owns its pixels through UpdateLayeredWindow. The
            // old BitBlt path would draw a square, region-clipped surface.
            if let Some(back) = self.back.as_ref() {
                crate::layered::prepare(
                    back,
                    (ui::CORNER_RADIUS * scale).round() as i32,
                    ACRYLIC_ALPHA,
                );
                crate::layered::update(self.hwnd, back);
            }
        }
        let elapsed = input::now().wrapping_sub(render_start).max(0) as f64;
        let qpf = input::qpf().max(1) as f64;
        let us = (elapsed * 1_000_000.0 / qpf).min(u32::MAX as f64) as u32;
        RENDER_US.store(us, Ordering::Relaxed);

        if interact {
            self.pressed = false;
            self.rpressed = false;
            self.apply(acts);
        }
    }

    fn paint_main(&mut self) {
        unsafe {
            let mut ps = PAINTSTRUCT::default();
            let hdc = win::BeginPaint(self.hwnd, &mut ps);
            self.draw(hdc, false);
            win::EndPaint(self.hwnd, &ps);
        }
    }

    fn interact(&mut self) {
        unsafe {
            let hdc = win::GetDC(self.hwnd);
            if hdc != 0 {
                self.draw(hdc, true);
                win::ReleaseDC(self.hwnd, hdc);
            }
        }
    }

    // ------------------------------------------------------------- actions

    fn changed(&mut self) {
        self.engine.publish(self.settings.clone());
        unsafe { win::SetTimer(self.hwnd, 2, 600, 0) };
        self.kick_anim();
        self.invalidate();
    }

    fn apply(&mut self, acts: Vec<Act>) {
        let listening = self.capture.is_some();
        for a in acts {
            // While listening, only another bind chip may act on the panel.
            // The mouse button being bound must reach the global hook instead
            // of toggling a control or cancelling the capture.
            if listening && !matches!(a, Act::Capture(_)) {
                continue;
            }
            match a {
                Act::Tab(i) => self.tab = i,
                Act::Toggle(k) => {
                    let v = !self.settings.bool_at(k);
                    self.settings.set_bool(k, v);
                    if !v {
                        if k == "v0" {
                            macros::stop_all_toggles();
                        } else {
                            macros::stop_toggle(k);
                        }
                    }
                    if k == "vP" && !v && !self.engine.present.load(Ordering::Relaxed) {
                        self.hide_panel();
                    }
                    self.changed();
                }
                Act::Mode(k) => {
                    macros::stop_toggle(macros::mode_module(k));
                    let next = if self.settings.mode_is_toggle(k) {
                        "hold"
                    } else {
                        "toggle"
                    };
                    self.settings.set_str(k, next);
                    self.changed();
                }
                Act::Theme(ch, v) => {
                    let (mut r, mut g, mut b) = self.settings.theme_rgb();
                    match ch {
                        0 => r = v.clamp(0, 255) as u8,
                        1 => g = v.clamp(0, 255) as u8,
                        _ => b = v.clamp(0, 255) as u8,
                    }
                    self.settings.set_theme_rgb(r as i32, g as i32, b as i32);
                    self.changed();
                }
                Act::Preset(r, g, b) => {
                    self.settings.set_theme_rgb(r, g, b);
                    self.changed();
                }
                Act::Reset => {
                    macros::stop_all_toggles();
                    // Reset means the shipped defaults, including keeping the
                    // panel open. Carrying the old toggle forward made a reset
                    // leave the panel able to disappear on the next focus tick.
                    self.settings = Settings::default();
                    self.changed();
                }
                Act::Capture(k) => {
                    if self.capture == Some(k) {
                        self.cancel_capture();
                    } else {
                        self.capture = Some(k);
                        self.capture_at = Self::ticks();
                        *self.engine.capture_key.lock().unwrap() = Some(k);
                        self.kick_anim();
                    }
                }
                Act::Clear(k) => {
                    self.settings.set_str(k, "...");
                    self.changed();
                }
                Act::SetInt(k, v) => {
                    self.settings.set_int(k, v);
                    // Keep Min <= Max and keep all fast timings inside their
                    // 47-97 / 53-103 slider domains while the user drags.
                    self.settings.normalize_timing();
                    self.changed();
                }
                Act::SetStr(k, v) => {
                    self.settings.set_str(k, v);
                    self.changed();
                }
                Act::Discord => unsafe { open_url("https://discord.gg/fvrdYzrQjq") },
                Act::Destruct => self.self_destruct(),
                Act::Hide => {
                    self.user_hidden = true;
                    self.hide_panel();
                    self.settings.save();
                }
                Act::Close => unsafe {
                    win::DestroyWindow(self.hwnd);
                },
            }
        }
        if self.settings.low_latency != self.applied_low {
            self.applied_low = self.settings.low_latency;
            apply_low_latency(self.applied_low);
        }
    }

    fn cancel_capture(&mut self) {
        self.capture = None;
        *self.engine.capture_key.lock().unwrap() = None;
        self.invalidate();
    }

    fn take_capture(&mut self) {
        let got = self.engine.captured.lock().unwrap().take();
        if let Some((key, name)) = got {
            if self.capture == Some(key) {
                self.settings.set_binding(key, &name);
                self.capture = None;
                *self.engine.capture_key.lock().unwrap() = None;
                self.changed();
            }
        }
        self.kick_anim();
        self.invalidate();
    }

    fn toggle_master(&mut self) {
        let v = !self.settings.bool_at("v0");
        self.settings.set_bool("v0", v);
        if !v {
            macros::stop_all_toggles();
        }
        self.changed();
    }

    fn self_destruct(&mut self) {
        if self.destructed {
            return;
        }
        self.destructed = true;

        let packaged = std::env::current_exe()
            .map(|p| {
                p.to_string_lossy()
                    .to_ascii_lowercase()
                    .contains("\\windowsapps\\")
            })
            .unwrap_or(false);
        if packaged {
            // MSIX install files live in a read-only WindowsApps directory and
            // cannot be recycled by the app. Clear only Droplet's own data and
            // open the Windows app-removal surface instead of pretending the
            // package was deleted.
            unsafe { open_url("ms-settings:appsfeatures") };
        } else if let Ok(exe) = std::env::current_exe() {
            if exe.is_absolute() && exe.is_file() {
                // A normal file goes straight to the Recycle Bin. Windows
                // locks a running image, so that call can return 124; in
                // that case a no-file waiter finishes the job after exit.
                let _ = win::recycle(&exe);
                if exe.exists() {
                    spawn_recycle_waiter(&exe);
                }
            }
        }

        crate::config::purge_data();
        unsafe { win::DestroyWindow(self.hwnd) };
    }

    // ------------------------------------------------------------ geometry

    fn pixel_size(&self) -> (i32, i32) {
        ((ui::WIN_W * self.dpi) as i32, (ui::WIN_H * self.dpi) as i32)
    }

    fn on_pointer(&mut self, lp: LPARAM) {
        // Synthetic macro input must never operate Droplet's own controls.
        // If the panel is open while a macro clicks, the click would otherwise
        // hover a chip and start a rebind, stealing the next trigger key.
        if unsafe { win::GetMessageExtraInfo() as usize } == input::INJ {
            return;
        }
        let x = ((lp & 0xFFFF) as u16 as i16) as i32;
        let y = (((lp >> 16) & 0xFFFF) as u16 as i16) as i32;
        let (mx, my, inside) = pointer_state(x, y, self.dpi);
        self.mx = mx;
        self.my = my;
        self.inside = inside;
    }

    /// Per-monitor DPI changes arrive with a suggested physical rectangle.
    /// Scale, window size, rounded clip, pointer mapping, and hit testing must
    /// move together; otherwise the panel draws one monitor's layout into the
    /// previous monitor's window and every control lands in the wrong place.
    fn apply_dpi(&mut self, dpi: u32, suggested: *const RECT) {
        if dpi == 0 {
            return;
        }
        // Finish any gesture before the geometry moves. A drag carries
        // physical coordinates and a char offset from the old scale; keeping
        // it would make the next mouse move jump the slider to a stale
        // position after the panel has been rescaled.
        self.release_pointer();
        self.dpi = dpi as f32 / 96.0;
        self.mx = -1.0;
        self.my = -1.0;
        self.inside = false;

        let mut rect = RECT::default();
        if !suggested.is_null() {
            rect = unsafe { *suggested };
        }
        if rect.width() > 0 && rect.height() > 0 {
            unsafe {
                win::SetWindowPos(
                    self.hwnd,
                    0,
                    rect.left,
                    rect.top,
                    rect.width(),
                    rect.height(),
                    win::SWP_NOZORDER | win::SWP_NOACTIVATE,
                );
            }
        } else {
            let (w, h) = self.pixel_size();
            unsafe {
                win::SetWindowPos(
                    self.hwnd,
                    0,
                    0,
                    0,
                    w,
                    h,
                    win::SWP_NOMOVE | win::SWP_NOZORDER | win::SWP_NOACTIVATE,
                );
            }
        }
        let (cw, ch) = self.client();
        unsafe { apply_window_shape(self.hwnd, cw, ch, self.dpi) };
        self.kick_anim();
        self.invalidate();
    }

    fn show_panel(&mut self) {
        if self.shown {
            return;
        }
        let (w, h) = self.pixel_size();
        let mut wa = RECT::default();
        unsafe {
            win::SystemParametersInfoW(
                win::SPI_GETWORKAREA,
                0,
                &mut wa as *mut RECT as *mut c_void,
                0,
            )
        };
        let (mut x, mut y) = if self.settings.win_x >= 0 && self.settings.win_y >= 0 {
            (self.settings.win_x, self.settings.win_y)
        } else {
            (
                wa.left + (wa.width() - w) / 2,
                wa.top + (wa.height() - h) / 2,
            )
        };
        // A saved position can fall outside the work area after a monitor,
        // resolution or taskbar change. Keep a usable strip reachable.
        if wa.width() > 0 && wa.height() > 0 {
            let vis_x = (80.0 * self.dpi) as i32;
            let vis_y = (40.0 * self.dpi) as i32;
            x = x.clamp(wa.left - w + vis_x, wa.right - vis_x);
            y = y.clamp(wa.top - h + vis_y, wa.bottom - vis_y);
        }
        unsafe {
            win::SetWindowPos(
                self.hwnd,
                win::HWND_TOPMOST,
                x,
                y,
                w,
                h,
                win::SWP_NOACTIVATE | win::SWP_SHOWWINDOW,
            );
        }
        self.shown = true;
        self.user_hidden = false;
        self.kick_anim();
        self.invalidate();
    }

    /// Drop every pointer gesture, including a slider drag that lives in the
    /// UI shared state rather than in the window-move slot. Used by mouse-up,
    /// lost capture, and the defensive auto-end path in WM_MOUSEMOVE.
    fn release_pointer(&mut self) {
        self.down = false;
        self.pressed = false;
        self.rpressed = false;
        self.shared.drag = None;
        self.end_drag();
    }

    fn end_drag(&mut self) {
        if self.drag.take().is_none() {
            return;
        }
        let mut r = RECT::default();
        unsafe {
            win::ReleaseCapture();
            win::GetWindowRect(self.hwnd, &mut r);
        }
        self.settings.win_x = r.left;
        self.settings.win_y = r.top;
        unsafe { win::SetTimer(self.hwnd, 2, 400, 0) };
    }

    fn hide_panel(&mut self) {
        if !self.shown {
            return;
        }
        let mut r = RECT::default();
        unsafe {
            win::GetWindowRect(self.hwnd, &mut r);
            win::ShowWindow(self.hwnd, win::SW_HIDE);
        }
        self.settings.win_x = r.left;
        self.settings.win_y = r.top;
        self.shown = false;
    }

    fn toggle_panel(&mut self) {
        if self.shown {
            self.user_hidden = true;
            self.hide_panel();
            self.settings.save();
        } else {
            self.show_panel();
        }
    }

    fn on_state(&mut self) {
        // A manual hide is sticky. Focus changes from alt-tabbing must not
        // resurrect the panel; only the toggle hotkey or a direct show clears
        // this flag. This is the bug where the panel reappeared after being
        // minimized to the tray when the user switched tabs.
        if self.user_hidden {
            return;
        }
        let present = self.engine.present.load(Ordering::Relaxed);
        let persist = self.settings.bool_at("vP");
        if present || persist {
            self.show_panel();
        } else {
            self.hide_panel();
        }
    }

    fn on_timer(&mut self, id: usize) {
        match id {
            1 => {
                let now = Self::ticks();
                if let Some(_) = self.capture {
                    if now.wrapping_sub(self.capture_at) > 6000 {
                        self.cancel_capture();
                    }
                }
                let alive = now < self.anim_until
                    || self.capture.is_some()
                    || SD_PROGRESS.load(Ordering::Relaxed) > 0;
                if alive {
                    self.invalidate();
                } else {
                    unsafe { win::KillTimer(self.hwnd, 1) };
                    self.anim_timer = false;
                }
            }
            2 => {
                unsafe { win::KillTimer(self.hwnd, 2) };
                let mut r = RECT::default();
                if self.shown {
                    unsafe { win::GetWindowRect(self.hwnd, &mut r) };
                    self.settings.win_x = r.left;
                    self.settings.win_y = r.top;
                }
                self.settings.save();
            }
            _ => {}
        }
    }
}

/// Windows keeps a running image locked. This helper waits for this process
/// to exit, then sends the exe to the Recycle Bin. It writes no script, log
/// or temp file and is used only when the direct recycle call is refused.
fn spawn_recycle_waiter(exe: &std::path::Path) {
    // A single-quoted PowerShell string takes the path literally; doubling
    // apostrophes is the only escape needed, so no shell path injection.
    let path = exe.to_string_lossy().replace('\'', "''");
    let script = format!(
        "$p=Get-Process -Id {} -ErrorAction SilentlyContinue; if($p){{$null=$p.WaitForExit(5000)}}; Add-Type -AssemblyName Microsoft.VisualBasic; [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile('{}',2,3)",
        std::process::id(),
        path
    );
    let params = format!(
        "-NoLogo -NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -Command \"{}\"",
        script
    );
    unsafe { shell_exec_hidden("powershell.exe", &params) };
}

/// Launch a helper through the shell without a console window. The app already
/// relies on ShellExecuteW for links, so this avoids linking std's process
/// spawner for the one self-destruct waiter.
unsafe fn shell_exec_hidden(file: &str, params: &str) {
    let dll = win::dyn_load("shell32.dll");
    if dll == 0 {
        return;
    }
    let p = win::load_proc(dll, "ShellExecuteW");
    if p == 0 {
        return;
    }
    let f: unsafe extern "system" fn(
        HWND,
        *const u16,
        *const u16,
        *const u16,
        *const u16,
        i32,
    ) -> isize = std::mem::transmute(p);
    let op = win::wide("open");
    let program = win::wide(file);
    let args = win::wide(params);
    f(
        0,
        op.as_ptr(),
        program.as_ptr(),
        args.as_ptr(),
        std::ptr::null(),
        0,
    );
}

// --------------------------------------------------------------- wndproc

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == win::WM_NCCREATE || msg == win::WM_CREATE {
        return win::DefWindowProcW(hwnd, msg, wp, lp);
    }
    if APP.is_null() {
        return win::DefWindowProcW(hwnd, msg, wp, lp);
    }
    let app = &mut *APP;
    match msg {
        win::WM_PAINT => {
            if hwnd == app.hwnd {
                app.paint_main();
            } else {
                return win::DefWindowProcW(hwnd, msg, wp, lp);
            }
            0
        }
        win::WM_ERASEBKGND => 1,
        win::WM_MOUSEACTIVATE => win::WM_MOUSEACTIVATE_NOACTIVATE,
        win::WM_NCHITTEST => {
            // The layered alpha mask makes the quarter-rounds transparent.
            // Keep the matching hit test so a click there never starts a drag
            // on a square edge.
            if corner_hit(hwnd, lp, app.dpi) {
                win::HTTRANSPARENT
            } else {
                win::HTCLIENT
            }
        }
        win::WM_DPICHANGED => {
            app.apply_dpi((wp & 0xFFFF) as u32, lp as *const RECT);
            0
        }
        win::WM_MOUSEMOVE => {
            if unsafe { win::GetMessageExtraInfo() as usize } == input::INJ {
                return 0;
            }
            let now = App::ticks();
            if let Some((ox, oy)) = app.drag {
                if win::is_down(0x01) {
                    let mut p = POINT::default();
                    win::GetCursorPos(&mut p);
                    win::SetWindowPos(
                        hwnd,
                        0,
                        p.x - ox,
                        p.y - oy,
                        0,
                        0,
                        win::SWP_NOSIZE | win::SWP_NOZORDER | win::SWP_NOACTIVATE,
                    );
                    return 0;
                }
            }
            // A button-up can be lost when capture changes or the pointer
            // leaves the window. A stale slider drag would otherwise keep
            // following the mouse with no button held.
            if (app.drag.is_some() || app.down || app.shared.drag.is_some()) && !win::is_down(0x01)
            {
                app.release_pointer();
            }
            app.on_pointer(lp);
            app.kick_anim();
            if now.wrapping_sub(app.last_move) >= 10 {
                app.last_move = now;
                app.interact();
            }
            0
        }
        win::WM_LBUTTONDOWN => {
            if unsafe { win::GetMessageExtraInfo() as usize } == input::INJ {
                return 0;
            }
            app.on_pointer(lp);
            let on_header = app.my >= 0.0
                && app.my < ui::HEADER_H * app.dpi
                && !ui::header_buttons(ui::WIN_W, app.dpi)
                    .iter()
                    .any(|r| r.has(app.mx, app.my));
            if on_header {
                let mut p = POINT::default();
                let mut wr = RECT::default();
                win::GetCursorPos(&mut p);
                win::GetWindowRect(hwnd, &mut wr);
                app.drag = Some((p.x - wr.left, p.y - wr.top));
                unsafe { win::SetCapture(hwnd) };
            } else {
                app.down = true;
                app.pressed = true;
                app.kick_anim();
                app.interact();
            }
            0
        }
        win::WM_LBUTTONUP => {
            if unsafe { win::GetMessageExtraInfo() as usize } == input::INJ {
                return 0;
            }
            app.on_pointer(lp);
            app.release_pointer();
            app.interact();
            0
        }
        win::WM_CAPTURECHANGED => {
            app.release_pointer();
            0
        }
        win::WM_RBUTTONDOWN => {
            if unsafe { win::GetMessageExtraInfo() as usize } == input::INJ {
                return 0;
            }
            app.on_pointer(lp);
            app.rpressed = true;
            app.interact();
            0
        }
        win::WM_TIMER => {
            app.on_timer(wp);
            0
        }
        win::WM_APP_CAPTURED => {
            app.take_capture();
            0
        }
        win::WM_APP_STATE => {
            app.on_state();
            0
        }
        win::WM_APP_TICK => {
            app.kick_anim();
            app.invalidate();
            0
        }
        win::WM_APP_PANEL => {
            if app.shown {
                app.hide_panel();
            } else {
                app.show_panel();
            }
            0
        }
        win::WM_APP_MODULE_TOGGLE => {
            app.invalidate();
            0
        }
        win::WM_APP_MASTER => {
            app.toggle_master();
            0
        }
        win::WM_APP_DESTRUCT => {
            app.self_destruct();
            0
        }
        win::WM_CLOSE => {
            // Close means quit, not hide: stop the watchers and tear the
            // window down immediately. The message loop then saves and exits.
            app.engine.quitting.store(true, Ordering::Relaxed);
            win::DestroyWindow(app.hwnd);
            0
        }
        win::WM_DESTROY => {
            win::PostQuitMessage(0);
            0
        }
        _ => win::DefWindowProcW(hwnd, msg, wp, lp),
    }
}

// ---------------------------------------------------------------- hooks

unsafe extern "system" fn kb_hook(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code < 0 {
        return win::CallNextHookEx(0, code, wp, lp);
    }
    let engine = match ENGINE.get() {
        Some(e) => e,
        None => return win::CallNextHookEx(0, code, wp, lp),
    };
    let info = &*(lp as *const win::KBDLLHOOKSTRUCT);
    if info.dwExtraInfo == input::INJ {
        return win::CallNextHookEx(0, code, wp, lp);
    }
    let vk = info.vkCode & 0xFF;
    let up = wp == win::WM_KEYUP as usize || wp == win::WM_SYSKEYUP as usize;
    input::set_physical(Ev::K(vk), !up);

    if up {
        input::signal_release(Ev::K(vk));
        DOWN_KEYS.with(|d| d.borrow_mut()[vk as usize] = false);
        return win::CallNextHookEx(0, code, wp, lp);
    }

    let fresh = DOWN_KEYS.with(|d| {
        let mut b = d.borrow_mut();
        if b[vk as usize] {
            false
        } else {
            b[vk as usize] = true;
            true
        }
    });
    if !fresh {
        // A W-Tap releases W while the user is still physically holding it.
        // Drop the OS auto-repeat of that held W for the tap window so it
        // cannot re-assert forward and cancel the S-Tap's backward step.
        if vk == 0x57 && input::w_repeat_suppressed() {
            return 1;
        }
        return win::CallNextHookEx(0, code, wp, lp);
    }

    let hwnd = MAIN_HWND.load(Ordering::Relaxed);

    // Rebinding capture takes precedence over everything else. There is no
    // debounce: the click that opened the capture was already processed by
    // the mouse hook, so a fast keypress must be accepted immediately.
    if let Some(key) = *engine.capture_key.lock().unwrap() {
        *engine.captured.lock().unwrap() = Some((key, keys::name_of_vk(vk)));
        win::PostMessageW(hwnd, win::WM_APP_CAPTURED, 0, 0);
        return win::CallNextHookEx(0, code, wp, lp);
    }

    // Ctrl + right shift toggles the panel
    if vk == 0xA1 && (win::GetAsyncKeyState(0x11) as u16 & 0x8000) != 0 {
        win::PostMessageW(hwnd, win::WM_APP_PANEL, 0, 0);
        return 1;
    }

    if !engine.gate.load(Ordering::Relaxed) {
        return win::CallNextHookEx(0, code, wp, lp);
    }

    // backslash: tap toggles the master switch, one second hold self-destructs
    if vk == 0xDC {
        if !SD_ACTIVE.swap(true, Ordering::SeqCst) {
            let _ = std::thread::Builder::new()
                .stack_size(64 * 1024)
                .spawn(move || {
                    let start = input::now();
                    let freq = input::qpf() as f64;
                    let mut progress: u32;
                    loop {
                        let elapsed = (input::now() - start) as f64 / freq;
                        progress = ((elapsed / 1.0) * 1000.0).clamp(0.0, 1000.0) as u32;
                        SD_PROGRESS.store(progress, Ordering::Relaxed);
                        win::PostMessageW(hwnd, win::WM_APP_TICK, 0, 0);
                        if elapsed >= 1.0 || !input::raw_down(Ev::K(0xDC)) {
                            break;
                        }
                        input::sleep_ms(40.0);
                    }
                    SD_PROGRESS.store(0, Ordering::Relaxed);
                    SD_ACTIVE.store(false, Ordering::SeqCst);
                    win::PostMessageW(hwnd, win::WM_APP_TICK, 0, 0);
                    if progress >= 1000 {
                        win::PostMessageW(hwnd, win::WM_APP_DESTRUCT, 0, 0);
                    } else if progress < 400 {
                        win::PostMessageW(hwnd, win::WM_APP_MASTER, 0, 0);
                    }
                });
        }
        return win::CallNextHookEx(0, code, wp, lp);
    }

    if !engine.has_trigger(Ev::K(vk)) {
        return win::CallNextHookEx(0, code, wp, lp);
    }
    let s = engine.settings();
    let map = engine.triggers();
    macros::trigger(Ev::K(vk), &s, &map);
    win::CallNextHookEx(0, code, wp, lp)
}

unsafe extern "system" fn ms_hook(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code < 0 {
        return win::CallNextHookEx(0, code, wp, lp);
    }
    let engine = match ENGINE.get() {
        Some(e) => e,
        None => return win::CallNextHookEx(0, code, wp, lp),
    };
    let info = &*(lp as *const win::MSLLHOOKSTRUCT);
    if info.dwExtraInfo == input::INJ {
        return win::CallNextHookEx(0, code, wp, lp);
    }
    let msg = wp as u32;
    let (btn, down) = match msg {
        win::WM_LBUTTONDOWN => (Btn::Left, true),
        win::WM_LBUTTONUP => (Btn::Left, false),
        win::WM_RBUTTONDOWN => (Btn::Right, true),
        win::WM_RBUTTONUP => (Btn::Right, false),
        win::WM_MBUTTONDOWN => (Btn::Middle, true),
        win::WM_MBUTTONUP => (Btn::Middle, false),
        0x020B => (
            if (info.mouseData >> 16) == 1 {
                Btn::X1
            } else {
                Btn::X2
            },
            true,
        ),
        0x020C => (
            if (info.mouseData >> 16) == 1 {
                Btn::X1
            } else {
                Btn::X2
            },
            false,
        ),
        _ => return win::CallNextHookEx(0, code, wp, lp),
    };

    input::set_physical(Ev::M(btn), down);

    if !down {
        input::signal_release(Ev::M(btn));
        return win::CallNextHookEx(0, code, wp, lp);
    }

    let hwnd = MAIN_HWND.load(Ordering::Relaxed);
    if let Some(key) = *engine.capture_key.lock().unwrap() {
        *engine.captured.lock().unwrap() = Some((key, btn.name().to_string()));
        win::PostMessageW(hwnd, win::WM_APP_CAPTURED, 0, 0);
        return win::CallNextHookEx(0, code, wp, lp);
    }
    if !engine.gate.load(Ordering::Relaxed) {
        return win::CallNextHookEx(0, code, wp, lp);
    }
    if !engine.has_trigger(Ev::M(btn)) {
        return win::CallNextHookEx(0, code, wp, lp);
    }
    let s = engine.settings();
    let map = engine.triggers();
    macros::trigger(Ev::M(btn), &s, &map);
    win::CallNextHookEx(0, code, wp, lp)
}

// -------------------------------------------------------------- discovery

fn process_leaf(image: &str) -> &str {
    image
        .rsplit(|c| c == '\\' || c == '/')
        .next()
        .unwrap_or(image)
}

/// Keep the pointer in the same physical space as the painted rects. Dividing
/// logical coordinates here shifts every hit target by one row/item on a 125%
/// or 150% display, which is exactly the GUI misplacement this fixes.
fn pointer_state(x: i32, y: i32, dpi: f32) -> (f32, f32, bool) {
    let w = (ui::WIN_W * dpi) as i32;
    let h = (ui::WIN_H * dpi) as i32;
    (x as f32, y as f32, x >= 0 && y >= 0 && x < w && y < h)
}

/// True only in the transparent quarter-rounds at the panel edge. Keeping this
/// in sync with `ui::CORNER_RADIUS` makes the visible corners click-through;
/// the layered alpha mask removes these pixels from the visible surface, while
/// the same geometry defines the hit target.
fn in_rounded_corner(x: f32, y: f32, w: f32, h: f32, radius: f32) -> bool {
    let r = radius.max(1.0);
    if x < 0.0 || y < 0.0 || x >= w || y >= h {
        return false;
    }
    let cx = if x < r {
        r
    } else if x > w - r {
        w - r
    } else {
        return false;
    };
    let cy = if y < r {
        r
    } else if y > h - r {
        h - r
    } else {
        return false;
    };
    let dx = x - cx;
    let dy = y - cy;
    dx * dx + dy * dy > r * r
}

fn corner_hit(hwnd: HWND, lp: LPARAM, dpi: f32) -> bool {
    let mut wr = RECT::default();
    if unsafe { win::GetWindowRect(hwnd, &mut wr) } == 0 {
        return false;
    }
    let x = (lp & 0xFFFF) as u16 as i16 as f32 - wr.left as f32;
    let y = ((lp >> 16) & 0xFFFF) as u16 as i16 as f32 - wr.top as f32;
    let w = (wr.right - wr.left) as f32;
    let h = (wr.bottom - wr.top) as f32;
    in_rounded_corner(x, y, w, h, (ui::CORNER_RADIUS * dpi).round().max(1.0))
}

/// Match the window to an actual Minecraft process, not just a title.
///
/// Browsers, Discord, editors, and the launcher can all carry "Minecraft" in
/// their title. A title is only a candidate hint; the process image decides.
/// Bedrock owns `Minecraft.Windows.exe`; Java clients run inside `javaw.exe`
/// (or a console `java.exe`) and expose a GLFW/LWJGL window class or a known
/// client title. Launcher windows are deliberately rejected.
fn game_window_matches(image: &str, class: &str, title: &str) -> bool {
    let exe = process_leaf(image).to_ascii_lowercase();
    let class = class.to_ascii_lowercase();
    let title = title.to_ascii_lowercase();

    if exe == "minecraft.windows.exe" {
        return true;
    }
    if exe != "javaw.exe" && exe != "java.exe" {
        return false;
    }
    if title.contains("launcher") || class.contains("launcher") {
        return false;
    }

    let game_class = class.starts_with("glfw") || class == "lwjgl" || class.contains("minecraft");
    let game_title =
        title.contains("minecraft") || title.contains("lunar client") || title.contains("badlion");
    game_class || game_title
}

fn is_game(hwnd: HWND) -> bool {
    if hwnd == 0 || unsafe { win::IsWindowVisible(hwnd) } == 0 {
        return false;
    }
    let cls = unsafe { win::get_class(hwnd) }.to_ascii_lowercase();
    let title = unsafe { win::get_title(hwnd) }.to_ascii_lowercase();
    let candidate = cls.starts_with("glfw")
        || cls == "lwjgl"
        || cls.contains("minecraft")
        || title.contains("minecraft")
        || title.contains("lunar client")
        || title.contains("badlion");
    if !candidate {
        return false;
    }

    let pid = unsafe { win::window_pid(hwnd) };
    if pid == 0 || pid == std::process::id() {
        return false;
    }
    let image = unsafe { win::process_image(pid) };
    if image.is_empty() {
        return false;
    }
    game_window_matches(&image, &cls, &title)
}

unsafe extern "system" fn enum_proc(hwnd: HWND, lp: LPARAM) -> BOOL {
    if win::IsWindowVisible(hwnd) == 0 {
        return 1;
    }
    if is_game(hwnd) {
        *(lp as *mut HWND) = hwnd;
        return 0;
    }
    1
}

fn find_game() -> HWND {
    let mut hwnd: HWND = 0;
    unsafe { win::EnumWindows(enum_proc, &mut hwnd as *mut HWND as LPARAM) };
    hwnd
}

fn spawn_watchers(engine: Arc<Engine>, hwnd: HWND) {
    // presence + focus
    let e = engine.clone();
    thread::spawn(move || {
        let mut last: Option<(bool, bool)> = None;
        let mut misses = 0u32;
        let mut hits = 0u32;
        while !e.quitting.load(Ordering::Relaxed) {
            // One foreground lookup per tick; only enumerate when the cache is cold.
            let fg = unsafe { win::GetForegroundWindow() };
            let root = if fg != 0 {
                unsafe { win::GetAncestor(fg, win::GA_ROOT) }
            } else {
                0
            };
            let mut game = GAME.load(Ordering::Relaxed);
            if game != 0 && unsafe { win::IsWindow(game) } == 0 {
                game = 0;
            }
            // Once the game window is verified, the foreground HWND cannot
            // change process identity. Skip a second process-image query on
            // every 160 ms tick and only revalidate when focus moves.
            if root != 0 {
                if root == game {
                    misses = 0;
                } else if is_game(root) {
                    game = root;
                    misses = 0;
                }
            }
            if game == 0 {
                misses += 1;
                if misses % 8 == 0 {
                    game = find_game();
                }
            }
            GAME.store(game, Ordering::Relaxed);
            // Confirm two samples (about 320 ms) before showing a running
            // game. Clear on the first miss so a closed client cannot leave
            // a stale badge, while a transient foreground change cannot
            // flicker the status.
            if game != 0 {
                hits = (hits + 1).min(2);
            } else {
                hits = 0;
            }
            let present = hits >= 2;
            let focus = present && root == game;

            // The presence loop only needs this one boolean; avoid cloning the
            // whole settings snapshot and taking the settings lock every tick.
            let gate = focus || !e.focus_only.load(Ordering::Relaxed);
            let was_gate = e.gate.swap(gate, Ordering::Relaxed);
            if gate != was_gate {
                wake_sprint();
            }
            if !gate && was_gate {
                // A latched Toggle worker must not outlive the foreground gate.
                macros::stop_all_toggles();
            }

            if last != Some((present, focus)) {
                e.present.store(present, Ordering::Relaxed);
                e.focus.store(focus, Ordering::Relaxed);
                unsafe { win::PostMessageW(hwnd, win::WM_APP_STATE, 0, 0) };
                last = Some((present, focus));
            }
            thread::sleep(Duration::from_millis(160));
        }
    });

    // crosshair pixel watch
    let e = engine.clone();
    thread::spawn(move || {
        let probe = Probe::new();
        while !e.quitting.load(Ordering::Relaxed) {
            let armed = e.totem_on.load(Ordering::Relaxed) && e.focus.load(Ordering::Relaxed);
            if armed {
                if let Some(p) = &probe {
                    // Sample first; the settings lock is only needed on an
                    // actual magenta flash, not on every crosshair tick.
                    if macros::totem_flash(p) {
                        let s = e.settings();
                        macros::totem_tap(&s);
                    }
                }
            }
            thread::sleep(Duration::from_millis(if armed { 16 } else { 100 }));
        }
    });

    // Sprint pulse: physical W only, 100 ms cadence. Poll at 8 ms while
    // armed so the 100 ms schedule is not rounded up; sleep at 100 ms while
    // idle. Focus/settings transitions unpark this thread immediately, so the
    // first pulse is not delayed by the idle wait.
    let e = engine.clone();
    thread::spawn(move || {
        let _ = SPRINT_WAIT.set(thread::current());
        while !e.quitting.load(Ordering::Relaxed) {
            let armed = e.gate.load(Ordering::Relaxed) && e.sprint_on.load(Ordering::Relaxed);
            if armed && input::physical_down(Ev::K(0x57)) {
                // Most 8 ms polls are not due. Check the deadline before
                // cloning the settings snapshot or taking its lock.
                if macros::sprint_due_now() {
                    let s = e.settings();
                    macros::sprint_tick(&s);
                }
            } else {
                macros::sprint_reset();
            }
            thread::park_timeout(Duration::from_millis(macros::sprint_wait_ms(armed)));
        }
    });
}

fn spawn_hooks(engine: Arc<Engine>) {
    thread::spawn(move || unsafe {
        HOOK_TID.store(win::GetCurrentThreadId(), Ordering::Relaxed);
        if engine.settings().low_latency {
            input::boost_thread();
        }
        // Low-level hooks are delivered through the installing thread's message
        // queue. Create that queue before SetWindowsHookEx; installing first can
        // succeed yet never reach the callback on a fresh worker thread.
        let mut msg = MSG::default();
        win::PeekMessageW(&mut msg, 0, 0, 0, 0);
        let kb = win::SetWindowsHookExW(win::WH_KEYBOARD_LL, kb_hook as *const () as usize, 0, 0);
        let ms = win::SetWindowsHookExW(win::WH_MOUSE_LL, ms_hook as *const () as usize, 0, 0);
        loop {
            let r = win::GetMessageW(&mut msg, 0, 0, 0);
            if r <= 0 || engine.quitting.load(Ordering::Relaxed) {
                break;
            }
            win::TranslateMessage(&msg);
            win::DispatchMessageW(&msg);
        }
        if kb != 0 {
            win::UnhookWindowsHookEx(kb);
        }
        if ms != 0 {
            win::UnhookWindowsHookEx(ms);
        }
        HOOK_TID.store(0, Ordering::Relaxed);
    });
}

// ------------------------------------------------------------- utilities

unsafe fn time_begin_period(ms: u32) {
    let dll = win::dyn_load("winmm.dll");
    if dll == 0 {
        return;
    }
    let p = win::load_proc(dll, "timeBeginPeriod");
    if p == 0 {
        return;
    }
    let f: unsafe extern "system" fn(u32) -> u32 = std::mem::transmute(p);
    f(ms);
}

unsafe fn time_end_period(ms: u32) {
    let dll = win::dyn_load("winmm.dll");
    if dll == 0 {
        return;
    }
    let p = win::load_proc(dll, "timeEndPeriod");
    if p == 0 {
        return;
    }
    let f: unsafe extern "system" fn(u32) -> u32 = std::mem::transmute(p);
    f(ms);
}

/// Ask for Realtime only when the low-latency switch is on. Windows normally
/// denies Realtime to an unelevated process; in that case fall back to High so
/// the panel remains usable rather than failing to schedule at all.
fn apply_priority(on: bool) {
    unsafe {
        let p = win::GetCurrentProcess();
        let ok = if on {
            win::SetPriorityClass(p, win::REALTIME_PRIORITY_CLASS)
        } else {
            win::SetPriorityClass(p, win::HIGH_PRIORITY_CLASS)
        };
        if on && ok == 0 {
            win::SetPriorityClass(p, win::HIGH_PRIORITY_CLASS);
        }
        REALTIME_ACTIVE.store(on && ok != 0, Ordering::Relaxed);
    }
}

type TimerQuery = unsafe extern "system" fn(*mut u32, *mut u32, *mut u32) -> i32;
type TimerSet = unsafe extern "system" fn(u32, u8, *mut u32) -> i32;

const TIMER_TARGET_100NS: u32 = 5_000;
const TIMER_IDLE_100NS: u32 = 156_250;
const ES_CONTINUOUS: u32 = 0x8000_0000;
const ES_SYSTEM_REQUIRED: u32 = 0x0000_0001;

fn timer_fns() -> (Option<TimerQuery>, Option<TimerSet>) {
    static QUERY: OnceLock<Option<TimerQuery>> = OnceLock::new();
    static SET: OnceLock<Option<TimerSet>> = OnceLock::new();
    let query = *QUERY.get_or_init(|| unsafe {
        let ntdll = win::dyn_load("ntdll.dll");
        if ntdll == 0 {
            None
        } else {
            let p = win::load_proc(ntdll, "NtQueryTimerResolution");
            if p == 0 {
                None
            } else {
                Some(std::mem::transmute(p))
            }
        }
    });
    let set = *SET.get_or_init(|| unsafe {
        let ntdll = win::dyn_load("ntdll.dll");
        if ntdll == 0 {
            None
        } else {
            let p = win::load_proc(ntdll, "NtSetTimerResolution");
            if p == 0 {
                None
            } else {
                Some(std::mem::transmute(p))
            }
        }
    });
    (query, set)
}

/// NtQueryTimerResolution returns the coarsest value in `min` and the finest
/// hardware value in `max` (the naming is counter-intuitive). The kernel's
/// finest value is normally 5,000 (0.5 ms). Request 0.5 ms when the hardware
/// supports it; if the finest available value is coarser, request that instead
/// of silently accepting the 15.625 ms idle default. The UI displays the value
/// the kernel actually granted.
fn timer_target_100ns(finest: u32) -> u32 {
    finest.max(TIMER_TARGET_100NS)
}

fn timer_query() -> (u32, u32) {
    let (query, _) = timer_fns();
    if let Some(q) = query {
        let mut min = 0u32;
        let mut max = 0u32;
        let mut current = 0u32;
        unsafe {
            if q(&mut min, &mut max, &mut current) == 0 {
                // `max` is the finest available resolution, not the coarsest.
                return (max, current);
            }
        }
    }
    (0, 0)
}

fn timer_set(target: u32, on: bool) -> u32 {
    let (_, set) = timer_fns();
    if let Some(s) = set {
        let mut actual = 0u32;
        unsafe {
            if s(target, on as u8, &mut actual) == 0 {
                return actual;
            }
        }
    }
    0
}

/// Keep a 0.5 ms timer request alive after startup. Windows can drop or raise
/// a process timer request after a power transition, a session event, or a
/// higher-resolution request from another app. A cheap one-second query-and-
/// reassert loop fixes the "re-enable low latency" workaround without polling
/// on the input path.
fn ensure_timer_guard() {
    if TIMER_GUARD_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    let spawned = thread::Builder::new()
        .name("droplet-timer-guard".to_string())
        .stack_size(32 * 1024)
        .spawn(|| loop {
            if !TIMER_ON.load(Ordering::SeqCst) {
                TIMER_GUARD_STARTED.store(false, Ordering::SeqCst);
                return;
            }
            let (finest, current) = timer_query();
            let target = timer_target_100ns(finest);
            if current == 0 || current > target {
                let actual = timer_set(target, true);
                // If low latency was switched off while the guard was in the
                // kernel call, release the request instead of leaving a
                // stale 0.5 ms hold behind.
                if !TIMER_ON.load(Ordering::SeqCst) {
                    let _ = timer_set(TIMER_TARGET_100NS, false);
                    TIMER_GUARD_STARTED.store(false, Ordering::SeqCst);
                    return;
                }
                if actual > 0 {
                    TIMER_100NS.store(actual, Ordering::Relaxed);
                }
            } else {
                TIMER_100NS.store(current, Ordering::Relaxed);
            }
            thread::sleep(Duration::from_millis(1000));
        });
    if spawned.is_err() {
        TIMER_GUARD_STARTED.store(false, Ordering::SeqCst);
    }
}

unsafe fn keep_awake(on: bool) {
    let flags = ES_CONTINUOUS | if on { ES_SYSTEM_REQUIRED } else { 0 };
    win::SetThreadExecutionState(flags);
}

/// Request the kernel's native 0.5 ms floor and keep it held. The multimedia
/// call is made first because the two APIs interact; NtSet records the
/// resolution the kernel actually granted, which the UI displays verbatim.
fn apply_timer(on: bool) {
    if on {
        TIMER_ON.store(true, Ordering::SeqCst);
        unsafe {
            time_begin_period(1);
        }
        let (finest, current) = timer_query();
        let target = timer_target_100ns(finest);
        let actual = timer_set(target, true);
        let shown = if actual > 0 {
            actual
        } else if current > 0 {
            current
        } else {
            TIMER_IDLE_100NS
        };
        TIMER_100NS.store(shown, Ordering::Relaxed);
        ensure_timer_guard();
    } else {
        TIMER_ON.store(false, Ordering::SeqCst);
        let _ = timer_set(TIMER_TARGET_100NS, false);
        unsafe {
            time_end_period(1);
        }
        let (_, current) = timer_query();
        TIMER_100NS.store(
            if current > 0 {
                current
            } else {
                TIMER_IDLE_100NS
            },
            Ordering::Relaxed,
        );
    }
}

fn apply_low_latency(on: bool) {
    apply_priority(on);
    apply_timer(on);
    unsafe {
        keep_awake(on);
    }
    unsafe {
        let tid = HOOK_TID.load(Ordering::Relaxed);
        if tid != 0 {
            let h = win::OpenThread(win::THREAD_SET_INFORMATION, 0, tid);
            if h != 0 {
                win::SetThreadPriority(
                    h,
                    if on {
                        win::THREAD_PRIORITY_TIME_CRITICAL
                    } else {
                        win::THREAD_PRIORITY_NORMAL
                    },
                );
                win::CloseHandle(h);
            }
        }
    }
}

unsafe fn open_url(url: &str) {
    let dll = win::dyn_load("shell32.dll");
    if dll == 0 {
        return;
    }
    let p = win::load_proc(dll, "ShellExecuteW");
    if p == 0 {
        return;
    }
    let f: unsafe extern "system" fn(
        HWND,
        *const u16,
        *const u16,
        *const u16,
        *const u16,
        i32,
    ) -> isize = std::mem::transmute(p);
    let op = win::wide("open");
    let u = win::wide(url);
    let opened = f(
        0,
        op.as_ptr(),
        u.as_ptr(),
        std::ptr::null(),
        std::ptr::null(),
        1,
    );
    if opened <= 32 {
        // A shell association can be missing or broken. Hand the URL to
        // Explorer explicitly so the click still reaches the default browser.
        let explorer = win::wide("explorer.exe");
        f(
            0,
            op.as_ptr(),
            explorer.as_ptr(),
            u.as_ptr(),
            std::ptr::null(),
            1,
        );
    }
}

unsafe fn native_icon() -> HICON {
    let s: i32 = 32;
    let mut xor = vec![0u32; (s * s) as usize];
    let and_mask = vec![0u8; ((s / 8) * s) as usize];
    // Flat monochrome mark, matching the header droplet. A mid-light neutral
    // stays readable on both dark and light taskbars; no rim or gradient.
    let fill = 0xFFF4_F3EFu32;
    for y in 0..s {
        for x in 0..s {
            // 3x3 supersampled teardrop: round bowl + taper to the apex.
            let mut hits = 0u32;
            for sy in 0..3 {
                for sx in 0..3 {
                    let px = (x as f32 + (sx as f32 + 0.5) / 3.0 - 16.0) / 15.0;
                    let py = (y as f32 + (sy as f32 + 0.5) / 3.0 - 16.0) / 15.0;
                    let cy = 0.24f32;
                    let r = 0.46f32;
                    let d2 = px * px + (py - cy) * (py - cy);
                    let t = (cy - py) / (cy + 1.0);
                    let in_tip = py < cy && py > -1.0 && px.abs() <= r * (1.0 - t);
                    if d2 <= r * r || in_tip {
                        hits += 1;
                    }
                }
            }
            xor[(y * s + x) as usize] = if hits == 0 {
                0
            } else {
                let a = (hits as f32 / 9.0 * 255.0) as u32;
                (fill & 0x00FF_FFFF) | (a << 24)
            };
        }
    }
    win::CreateIcon(0, s, s, 1, 32, and_mask.as_ptr(), xor.as_ptr() as *const u8)
}

// ------------------------------------------------------------------- run

pub fn run() {
    unsafe {
        win::SetProcessDpiAwarenessContext(win::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        let mutex_name = win::wide("DropletMacroSuite");
        win::SetLastError(0);
        let mtx = win::CreateMutexW(std::ptr::null_mut(), 1, mutex_name.as_ptr());
        if mtx != 0 && win::GetLastError() == win::ERROR_ALREADY_EXISTS {
            return;
        }

        // Keep Windows from popping error dialogs or doing crash bookkeeping.
        win::SetErrorMode(0x0001 | 0x0002 | 0x0003);

        input::init();

        if gp::init().is_none() {
            return;
        }
        gp::init_formats();

        let settings = Settings::load();
        // Realtime with low latency; High otherwise. apply_priority keeps a
        // High fallback when the token cannot request Realtime.
        apply_low_latency(settings.low_latency);
        let engine = Arc::new(Engine::new(&settings));
        let _ = ENGINE.set(engine.clone());

        let fonts = match Fonts::new("Segoe UI") {
            Some(f) => f,
            None => {
                return;
            }
        };

        let hinst = win::GetModuleHandleW(std::ptr::null());
        let class = win::wide("DropletOverlay");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0x0001 | 0x0002,
            lpfnWndProc: wndproc as *const () as usize,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinst,
            hIcon: 0,
            hCursor: win::LoadCursorW(0, win::IDC_ARROW as *const u16),
            hbrBackground: 0,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class.as_ptr(),
            hIconSm: 0,
        };
        if win::RegisterClassExW(&wc) == 0 {
            return;
        }

        let title = win::wide("Droplet");
        let ex_style =
            win::WS_EX_TOPMOST | win::WS_EX_TOOLWINDOW | win::WS_EX_NOACTIVATE | win::WS_EX_LAYERED;
        let hwnd = win::CreateWindowExW(
            ex_style,
            class.as_ptr(),
            title.as_ptr(),
            win::WS_POPUP,
            0,
            0,
            720,
            520,
            0,
            0,
            hinst,
            std::ptr::null_mut(),
        );
        if hwnd == 0 {
            return;
        }

        let dpi = {
            let d = win::GetDpiForWindow(hwnd);
            if d == 0 {
                1.0
            } else {
                d as f32 / 96.0
            }
        };
        let (w, h) = ((ui::WIN_W * dpi) as i32, (ui::WIN_H * dpi) as i32);
        win::SetWindowPos(
            hwnd,
            0,
            0,
            0,
            w,
            h,
            win::SWP_NOMOVE | win::SWP_NOZORDER | win::SWP_NOACTIVATE,
        );

        // Desktop acrylic is composited by DWM, so it stays off the app's
        // paint thread. The per-pixel alpha surface still owns the final
        // pixels; this call keeps the material available where the platform
        // supports it.
        let _ = apply_acrylic(hwnd);

        // The layered alpha surface is the primary silhouette on every Windows
        // version, including Windows 10 where DWM's newer corner attribute is
        // unsupported. The helper is reused by WM_DPICHANGED so a monitor
        // change cannot leave an old corner mask.
        apply_window_shape(hwnd, w, h, dpi);

        let icon = native_icon();
        if icon != 0 {
            win::SetClassLongPtrW(hwnd, -14, icon);
        }

        let app = Box::new(App::new(hwnd, dpi, engine.clone(), settings, fonts));
        APP = Box::into_raw(app);
        MAIN_HWND.store(hwnd, Ordering::Relaxed);

        spawn_hooks(engine.clone());
        spawn_watchers(engine.clone(), hwnd);

        let mut msg = MSG::default();
        loop {
            let r = win::GetMessageW(&mut msg, 0, 0, 0);
            if r <= 0 {
                break;
            }
            win::TranslateMessage(&msg);
            win::DispatchMessageW(&msg);
        }

        engine.quitting.store(true, Ordering::Relaxed);
        let app = &mut *APP;
        if !app.destructed {
            app.settings.save();
        }
        apply_timer(false);
        keep_awake(false);
        if icon != 0 {
            win::DestroyIcon(icon);
        }
        gp::shutdown();
        // Everything is flushed; leave at once instead of waiting on runtime
        // teardown. The close button and self-destruct both land here.
        std::process::exit(0);
    }
}

const ACRYLIC_ALPHA: u8 = 250;
const ACRYLIC_TINT: u32 = 0x5A0B_0A0C;

unsafe fn apply_acrylic(hwnd: HWND) -> &'static str {
    if system_backdrop(hwnd) {
        return "backdrop";
    }
    if accent_policy(hwnd) {
        return "accent";
    }
    "none"
}

/// Legacy Windows 10 accent path. Some newer builds no longer expose it, so
/// it is the fallback after the supported DWM system-backdrop attribute.
unsafe fn accent_policy(hwnd: HWND) -> bool {
    let dll = win::dyn_load("user32.dll");
    if dll == 0 {
        return false;
    }
    let p = win::load_proc(dll, "SetWindowCompositionAttribute");
    if p == 0 {
        return false;
    }
    let f: unsafe extern "system" fn(HWND, *mut win::WindowCompositionAttributeData) -> i32 =
        std::mem::transmute(p);
    let mut accent = win::AccentPolicy {
        state: win::ACCENT_ENABLE_ACRYLICBLURBEHIND,
        flags: 0,
        gradient: ACRYLIC_TINT,
        animation: 0,
    };
    let mut data = win::WindowCompositionAttributeData {
        attrib: win::WCA_ACCENT_POLICY,
        data: &mut accent as *mut _ as *mut std::ffi::c_void,
        size: std::mem::size_of::<win::AccentPolicy>(),
    };
    f(hwnd, &mut data) != 0
}

/// Windows 11 22H2+ system backdrop: DWMSBT_TRANSIENTWINDOW is the desktop
/// acrylic material. The app still owns every pixel; DWM only supplies the
/// blurred material behind the window, so this costs no app CPU.
unsafe fn system_backdrop(hwnd: HWND) -> bool {
    let dll = win::dyn_load("dwmapi.dll");
    if dll == 0 {
        return false;
    }
    let p = win::load_proc(dll, "DwmSetWindowAttribute");
    if p == 0 {
        return false;
    }
    let f: unsafe extern "system" fn(HWND, u32, *const std::ffi::c_void, u32) -> i32 =
        std::mem::transmute(p);
    let kind: i32 = 3; // DWMSBT_TRANSIENTWINDOW (desktop acrylic)
    f(hwnd, 38, &kind as *const i32 as *const std::ffi::c_void, 4) == 0
}

/// Append one local diagnostic line. This is the same `%LOCALAPPDATA%\Droplet`
/// directory as the config and goes nowhere else: no network, telemetry, or
/// external process. Cap it so a long-lived install cannot grow it forever.
fn dwm_log(line: &str) {
    let path = config::data_dir().join("dwm.log");
    if let Ok(meta) = std::fs::metadata(&path) {
        if meta.len() > 32 * 1024 {
            let _ = std::fs::remove_file(&path);
        }
    }
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(file, "{line}");
    }
}

/// The visible silhouette now comes from the per-pixel alpha mask in
/// `layered::prepare`, not a binary GDI region. Clear any region left by an
/// older DPI pass so it cannot cut a stair step through the antialiased edge.
/// The layered surface itself is the Windows 10 fallback: it does not depend on
/// the Windows 11 corner attribute, and DWM's square popup shadow is not part of
/// the per-pixel surface. Every DWM result is recorded in `dwm.log`.
unsafe fn apply_window_shape(hwnd: HWND, w: i32, h: i32, dpi: f32) {
    if w <= 0 || h <= 0 {
        return;
    }
    let round = set_dwm_frame(hwnd);
    let cleared = win::SetWindowRgn(hwnd, 0, 1);
    dwm_log(&format!(
        "shape size={w}x{h} dpi={dpi:.2} mode=layered-alpha attr33_ok={round} region_cleared={cleared}"
    ));
}

/// Configure DWM for a borderless edge and record every HRESULT. DWM is asked
/// to round on Windows 11, but the layered alpha surface owns the visible
/// silhouette on both Windows 10 and 11.
unsafe fn set_dwm_frame(hwnd: HWND) -> bool {
    let dll = win::dyn_load("dwmapi.dll");
    if dll == 0 {
        dwm_log("dwm dll=missing");
        return false;
    }
    let p = win::load_proc(dll, "DwmSetWindowAttribute");
    if p == 0 {
        dwm_log("dwm entry=missing");
        return false;
    }
    let f: unsafe extern "system" fn(HWND, u32, *const c_void, u32) -> i32 = std::mem::transmute(p);
    let pref: i32 = 2; // DWMWCP_ROUND
    let round = f(hwnd, 33, &pref as *const i32 as *const c_void, 4);
    let dark: i32 = 1; // DWMWA_USE_IMMERSIVE_DARK_MODE
    let dark_hr = f(hwnd, 20, &dark as *const i32 as *const c_void, 4);
    // Remove DWM's square border so the rounded surface owns the edge.
    let border: u32 = 0xFFFF_FFFE; // DWMWA_COLOR_NONE
    let border_hr = f(hwnd, 34, &border as *const u32 as *const c_void, 4);
    dwm_log(&format!(
        "DwmSetWindowAttribute attr33=0x{round:08X} attr20=0x{dark_hr:08X} attr34=0x{border_hr:08X}"
    ));
    round == 0
}

#[cfg(test)]
mod tests {
    use super::{
        game_window_matches, in_rounded_corner, pointer_state, process_leaf, timer_target_100ns,
    };
    use crate::ui;

    #[test]
    fn game_detection_rejects_browser_and_chat_titles() {
        assert!(!game_window_matches(
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            "Chrome_WidgetWin_1",
            "Minecraft - Google Chrome"
        ));
        assert!(!game_window_matches(
            r"C:\Users\me\AppData\Local\Discord\app-1.0\Discord.exe",
            "Chrome_WidgetWin_1",
            "Minecraft | Discord"
        ));
        assert!(!game_window_matches(
            r"C:\Windows\System32\notepad.exe",
            "Notepad",
            "Minecraft notes.txt"
        ));
    }

    #[test]
    fn game_detection_rejects_the_launcher() {
        assert!(!game_window_matches(
            r"C:\XboxGames\Minecraft Launcher\MinecraftLauncher.exe",
            "MinecraftLauncher",
            "Minecraft Launcher"
        ));
        assert!(!game_window_matches(
            r"C:\Java\bin\javaw.exe",
            "SunAwtFrame",
            "Minecraft Launcher"
        ));
    }

    #[test]
    fn game_detection_accepts_java_clients_only() {
        assert!(game_window_matches(
            r"C:\Java\bin\javaw.exe",
            "GLFW30",
            "Minecraft 1.8.9"
        ));
        assert!(game_window_matches(
            r"C:\Java\bin\java.exe",
            "LWJGL",
            "Lunar Client"
        ));
        assert!(game_window_matches(
            r"C:\Java\bin\javaw.exe",
            "SunAwtFrame",
            "Badlion Client"
        ));
    }

    #[test]
    fn game_detection_accepts_bedrock_and_reads_process_leaf() {
        assert!(game_window_matches(
            r"C:\Program Files\WindowsApps\Microsoft.MinecraftUWP\Minecraft.Windows.exe",
            "ApplicationFrameWindow",
            "Minecraft"
        ));
        assert_eq!(process_leaf(r"C:\Games\javaw.exe"), "javaw.exe");
        assert_eq!(process_leaf("C:/Games/javaw.exe"), "javaw.exe");
    }

    #[test]
    fn scaled_pointer_hits_the_scaled_rail() {
        // At 125%, Crystal occupies physical y=110..152 and Core y=65..107.
        // A pointer that was divided back to logical space (y=104) would pick
        // Core, which was the reported one-row-off bug.
        let scale = 1.25;
        let (mx, my, inside) = pointer_state(50, 131, scale);
        assert!(inside);
        assert_eq!((mx, my), (50.0, 131.0));
        assert!(ui::rail_rect(1, scale).has(mx, my));
        assert!(!ui::rail_rect(0, scale).has(mx, my));

        // At 150%, the same physical point must still hit Crystal and remain
        // inside the taller panel.
        let scale = 1.5;
        let (mx, my, inside) = pointer_state(60, 157, scale);
        assert!(inside);
        assert!(ui::rail_rect(1, scale).has(mx, my));
        assert!(!ui::rail_rect(0, scale).has(mx, my));

        // The right/bottom edge is exclusive, matching Rect::has.
        assert!(!pointer_state(780, 510, 1.25).2);
    }

    #[test]
    fn timer_target_never_requests_coarser_than_half_ms() {
        assert_eq!(timer_target_100ns(0), 5_000);
        assert_eq!(timer_target_100ns(5_000), 5_000);
        assert_eq!(timer_target_100ns(6_250), 6_250);
    }

    #[test]
    fn corner_hit_test_matches_the_rounded_edge_at_every_dpi() {
        for scale in [1.0f32, 1.25, 1.5] {
            let w = ui::WIN_W * scale;
            let h = ui::WIN_H * scale;
            let r = (ui::CORNER_RADIUS * scale).round().max(1.0);
            // Every outer corner is click-through, and the mirrored corners
            // agree; the straight edge stays inside the client area.
            assert!(in_rounded_corner(0.0, 0.0, w, h, r));
            assert!(in_rounded_corner(w - 1.0, 0.0, w, h, r));
            assert!(in_rounded_corner(0.0, h - 1.0, w, h, r));
            assert!(in_rounded_corner(w - 1.0, h - 1.0, w, h, r));
            assert!(!in_rounded_corner(r, r, w, h, r));
            assert!(!in_rounded_corner(w - r, r, w, h, r));
            assert!(!in_rounded_corner(r, h - r, w, h, r));
            assert!(!in_rounded_corner(w / 2.0, 2.0, w, h, r));
            assert!(!in_rounded_corner(-1.0, 4.0, w, h, r));
        }
    }
}
