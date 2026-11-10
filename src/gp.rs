//! GDI+ bindings, loaded dynamically so the binary links against nothing but
//! the core system libraries.

#![allow(non_snake_case, non_camel_case_types, dead_code)]

use std::ffi::c_void;
use std::sync::OnceLock;

use crate::win::{self, HBITMAP, HDC, HGDIOBJ};

#[repr(C)]
pub struct GdiplusStartupInput {
    pub version: u32,
    pub debug_callback: usize,
    pub suppress_background: i32,
    pub suppress_external_codecs: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RectF {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl RectF {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> RectF {
        RectF { x, y, w, h }
    }
}

pub type Gp = *mut c_void;

pub const SMOOTHING_ANTIALIAS: i32 = 4;
pub const PIXEL_OFFSET_HALF: i32 = 4;
pub const TEXT_ANTIALIAS_GRIDFIT: i32 = 3;
pub const TEXT_CLEARTYPE_GRIDFIT: i32 = 5;
pub const COMPOSITING_HIGH_QUALITY: i32 = 2;

pub const UNIT_PIXEL: i32 = 2;
pub const FONT_REGULAR: i32 = 0;
pub const FONT_BOLD: i32 = 1;

pub const ALIGN_NEAR: i32 = 0;
pub const ALIGN_CENTER: i32 = 1;
pub const ALIGN_FAR: i32 = 2;

pub const FILL_WINDING: i32 = 0;
pub const LINECAP_ROUND: i32 = 2;
pub const CLIP_REPLACE: i32 = 0;

/// Name of the entry point that failed to resolve (diagnostics).
pub static MISSING: std::sync::Mutex<Option<&'static str>> = std::sync::Mutex::new(None);

pub fn missing() -> String {
    match MISSING.lock() {
        Ok(g) => g.map(|s| s.to_string()).unwrap_or_else(|| "startup".to_string()),
        Err(_) => "mutex".to_string(),
    }
}

macro_rules! gdip_api {
    ($($name:ident($($a:ty),*) -> $r:ty;)*) => {
        pub struct Gdip {
            $(pub $name: unsafe extern "system" fn($($a),*) -> $r,)*
        }

        impl Gdip {
            unsafe fn bind(dll: win::HMODULE) -> Option<Gdip> {
                $(let $name = {
                    let p = win::GetProcAddress(dll, concat!(stringify!($name), "\0").as_ptr());
                    if p == 0 {
                        if let Ok(mut m) = MISSING.lock() { *m = Some(stringify!($name)); }
                        return None;
                    }
                    std::mem::transmute::<usize, unsafe extern "system" fn($($a),*) -> $r>(p)
                };)*
                Some(Gdip { $($name,)* })
            }
        }
    };
}

gdip_api! {
    GdiplusStartup(*mut usize, *const GdiplusStartupInput, *mut c_void) -> i32;
    GdiplusShutdown(usize) -> ();
    GdipCreateFromHDC(HDC, *mut Gp) -> i32;
    GdipDeleteGraphics(Gp) -> i32;
    GdipSetSmoothingMode(Gp, i32) -> i32;
    GdipSetPixelOffsetMode(Gp, i32) -> i32;
    GdipSetTextRenderingHint(Gp, i32) -> i32;
    GdipSetCompositingQuality(Gp, i32) -> i32;
    GdipCreateSolidFill(u32, *mut Gp) -> i32;
    GdipSetSolidFillColor(Gp, u32) -> i32;
    GdipDeleteBrush(Gp) -> i32;
    GdipFillRectangle(Gp, Gp, f32, f32, f32, f32) -> i32;
    GdipFillPath(Gp, Gp, Gp) -> i32;
    GdipFillEllipse(Gp, Gp, f32, f32, f32, f32) -> i32;
    GdipDrawPath(Gp, Gp, Gp) -> i32;
    GdipDrawLine(Gp, Gp, f32, f32, f32, f32) -> i32;
    GdipCreatePath(i32, *mut Gp) -> i32;
    GdipResetPath(Gp) -> i32;
    GdipDeletePath(Gp) -> i32;
    GdipAddPathArc(Gp, f32, f32, f32, f32, f32, f32) -> i32;
    GdipAddPathLine(Gp, f32, f32, f32, f32) -> i32;
    GdipAddPathBezier(Gp, f32, f32, f32, f32, f32, f32, f32, f32) -> i32;
    GdipAddPathEllipse(Gp, f32, f32, f32, f32) -> i32;
    GdipAddPathRectangle(Gp, f32, f32, f32, f32) -> i32;
    GdipClosePathFigure(Gp) -> i32;
    GdipSetClipRect(Gp, f32, f32, f32, f32, i32) -> i32;
    GdipResetClip(Gp) -> i32;
    GdipCreatePen1(u32, f32, i32, *mut Gp) -> i32;
    GdipDeletePen(Gp) -> i32;
    GdipSetPenColor(Gp, u32) -> i32;
    GdipSetPenWidth(Gp, f32) -> i32;
    GdipSetPenLineCap197819(Gp, i32, i32, i32) -> i32;
    GdipCreateFontFamilyFromName(*const u16, Gp, *mut Gp) -> i32;
    GdipDeleteFontFamily(Gp) -> i32;
    GdipCreateFont(Gp, f32, i32, i32, *mut Gp) -> i32;
    GdipDeleteFont(Gp) -> i32;
    GdipCreateStringFormat(i32, u16, *mut Gp) -> i32;
    GdipDeleteStringFormat(Gp) -> i32;
    GdipSetStringFormatAlign(Gp, i32) -> i32;
    GdipSetStringFormatLineAlign(Gp, i32) -> i32;
    GdipSetStringFormatFlags(Gp, i32) -> i32;
    GdipSetStringFormatTrimming(Gp, i32) -> i32;
    GdipDrawString(Gp, *const u16, i32, Gp, *const RectF, Gp, Gp) -> i32;
    GdipMeasureString(Gp, *const u16, i32, Gp, *const RectF, Gp, *mut RectF, *mut i32, *mut i32) -> i32;
}

static GDIP: OnceLock<Gdip> = OnceLock::new();
static mut TOKEN: usize = 0;

/// Initialise GDI+. Returns `None` when the DLL is unavailable (never on a
/// supported Windows install) so the process can bail out gracefully.
pub fn init() -> Option<&'static Gdip> {
    if let Some(g) = GDIP.get() {
        return Some(g);
    }
    unsafe {
        let dll = win::dyn_load("gdiplus.dll");
        if dll == 0 {
            return None;
        }
        let api = Gdip::bind(dll)?;
        let input = GdiplusStartupInput {
            version: 1,
            debug_callback: 0,
            suppress_background: 0,
            suppress_external_codecs: 0,
        };
        let mut token: usize = 0;
        if (api.GdiplusStartup)(&mut token, &input, std::ptr::null_mut()) != 0 {
            return None;
        }
        TOKEN = token;
        let _ = GDIP.set(api);
        GDIP.get()
    }
}

pub fn api() -> &'static Gdip {
    GDIP.get().expect("gdiplus not initialised")
}

pub fn shutdown() {
    if let Some(api) = GDIP.get() {
        unsafe { (api.GdiplusShutdown)(TOKEN) };
    }
}

// ------------------------------------------------------------------ drawing

/// A thin, allocation-light painting surface around a GDI+ Graphics object.
pub struct Canvas {
    pub g: Gp,
    brush: Gp,
    pen: Gp,
    path: Gp,
    color: u32,
    pen_color: u32,
    pen_width: f32,
    utf16: Vec<u16>,
}

impl Canvas {
    pub fn new(hdc: HDC) -> Option<Canvas> {
        let api = api();
        unsafe {
            let mut g: Gp = std::ptr::null_mut();
            if (api.GdipCreateFromHDC)(hdc, &mut g) != 0 {
                return None;
            }
            (api.GdipSetSmoothingMode)(g, SMOOTHING_ANTIALIAS);
            (api.GdipSetPixelOffsetMode)(g, PIXEL_OFFSET_HALF);
            (api.GdipSetTextRenderingHint)(g, TEXT_ANTIALIAS_GRIDFIT);
            (api.GdipSetCompositingQuality)(g, COMPOSITING_HIGH_QUALITY);

            let mut brush: Gp = std::ptr::null_mut();
            (api.GdipCreateSolidFill)(0xFFFFFFFF, &mut brush);
            let mut pen: Gp = std::ptr::null_mut();
            (api.GdipCreatePen1)(0xFFFFFFFF, 1.0, UNIT_PIXEL, &mut pen);
            (api.GdipSetPenLineCap197819)(pen, LINECAP_ROUND, LINECAP_ROUND, LINECAP_ROUND);
            let mut path: Gp = std::ptr::null_mut();
            if (api.GdipCreatePath)(FILL_WINDING, &mut path) != 0 {
                (api.GdipDeleteBrush)(brush);
                (api.GdipDeletePen)(pen);
                (api.GdipDeleteGraphics)(g);
                return None;
            }

            Some(Canvas {
                g,
                brush,
                pen,
                path,
                color: 0xFFFFFFFF,
                pen_color: 0xFFFFFFFF,
                pen_width: 1.0,
                utf16: Vec::with_capacity(96),
            })
        }
    }

    #[inline]
    fn fill_color(&mut self, color: u32) {
        if color != self.color {
            unsafe { (api().GdipSetSolidFillColor)(self.brush, color) };
            self.color = color;
        }
    }

    #[inline]
    fn reset_path(&mut self) -> Gp {
        unsafe { (api().GdipResetPath)(self.path) };
        self.path
    }

    #[inline]
    fn stroke(&mut self, color: u32, width: f32) {
        unsafe {
            if color != self.pen_color {
                (api().GdipSetPenColor)(self.pen, color);
                self.pen_color = color;
            }
            if width != self.pen_width {
                (api().GdipSetPenWidth)(self.pen, width);
                self.pen_width = width;
            }
        }
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: u32) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        self.fill_color(color);
        unsafe { (api().GdipFillRectangle)(self.g, self.brush, x, y, w, h) };
    }

    pub fn round_rect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, color: u32) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let r = r.min(w * 0.5).min(h * 0.5).max(0.0);
        if r < 0.6 {
            self.rect(x, y, w, h, color);
            return;
        }
        let api = api();
        let p = self.reset_path();
        unsafe {
            let d = r * 2.0;
            (api.GdipAddPathArc)(p, x, y, d, d, 180.0, 90.0);
            (api.GdipAddPathArc)(p, x + w - d, y, d, d, 270.0, 90.0);
            (api.GdipAddPathArc)(p, x + w - d, y + h - d, d, d, 0.0, 90.0);
            (api.GdipAddPathArc)(p, x, y + h - d, d, d, 90.0, 90.0);
            (api.GdipClosePathFigure)(p);
            self.fill_color(color);
            (api.GdipFillPath)(self.g, self.brush, p);
        }
    }

    /// 1px hairline border drawn as an antialiased outline path.
    pub fn round_stroke(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, color: u32, width: f32) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let r = r.min(w * 0.5).min(h * 0.5).max(0.0);
        let api = api();
        let p = self.reset_path();
        unsafe {
            if r < 0.6 {
                (api.GdipAddPathRectangle)(p, x, y, w, h);
            } else {
                let d = r * 2.0;
                (api.GdipAddPathArc)(p, x, y, d, d, 180.0, 90.0);
                (api.GdipAddPathArc)(p, x + w - d, y, d, d, 270.0, 90.0);
                (api.GdipAddPathArc)(p, x + w - d, y + h - d, d, d, 0.0, 90.0);
                (api.GdipAddPathArc)(p, x, y + h - d, d, d, 90.0, 90.0);
                (api.GdipClosePathFigure)(p);
            }
            self.stroke(color, width);
            (api.GdipDrawPath)(self.g, self.pen, p);
        }
    }

    pub fn circle(&mut self, cx: f32, cy: f32, radius: f32, color: u32) {
        self.fill_color(color);
        unsafe { (api().GdipFillEllipse)(self.g, self.brush, cx - radius, cy - radius, radius * 2.0, radius * 2.0) };
    }

    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: u32, width: f32) {
        self.stroke(color, width);
        unsafe { (api().GdipDrawLine)(self.g, self.pen, x1, y1, x2, y2) };
    }

    /// Filled diamond (the wordmark glyph).
    pub fn diamond(&mut self, cx: f32, cy: f32, radius: f32, color: u32) {
        let api = api();
        let p = self.reset_path();
        unsafe {
            (api.GdipAddPathLine)(p, cx, cy - radius, cx + radius * 0.78, cy);
            (api.GdipAddPathLine)(p, cx + radius * 0.78, cy, cx, cy + radius);
            (api.GdipAddPathLine)(p, cx, cy + radius, cx - radius * 0.78, cy);
            (api.GdipAddPathLine)(p, cx - radius * 0.78, cy, cx, cy - radius);
            (api.GdipClosePathFigure)(p);
            self.fill_color(color);
            (api.GdipFillPath)(self.g, self.brush, p);
        }
    }

    /// Filled water droplet: apex at (`cx`, `top`), bowl `w` wide and `h` tall.
    pub fn drop(&mut self, cx: f32, top: f32, w: f32, h: f32, color: u32) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let hw = w * 0.5;
        let api = api();
        let p = self.reset_path();
        unsafe {
            (api.GdipAddPathBezier)(
                p, cx, top,
                cx - w * 0.06, top + h * 0.26,
                cx - hw, top + h * 0.44,
                cx - hw, top + h * 0.64,
            );
            (api.GdipAddPathBezier)(
                p, cx - hw, top + h * 0.64,
                cx - hw, top + h * 1.08,
                cx + hw, top + h * 1.08,
                cx + hw, top + h * 0.64,
            );
            (api.GdipAddPathBezier)(
                p, cx + hw, top + h * 0.64,
                cx + hw, top + h * 0.44,
                cx + w * 0.06, top + h * 0.26,
                cx, top,
            );
            (api.GdipClosePathFigure)(p);
            self.fill_color(color);
            (api.GdipFillPath)(self.g, self.brush, p);
        }
    }

    /// Tinted water droplet with a crisp outline: the quiet panel mark.
    /// Shares the silhouette of [`Canvas::drop`].
    pub fn drop_mark(&mut self, cx: f32, top: f32, w: f32, h: f32, fill: u32, line: u32, width: f32) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let hw = w * 0.5;
        let api = api();
        let p = self.reset_path();
        unsafe {
            (api.GdipAddPathBezier)(
                p, cx, top,
                cx - w * 0.06, top + h * 0.26,
                cx - hw, top + h * 0.44,
                cx - hw, top + h * 0.64,
            );
            (api.GdipAddPathBezier)(
                p, cx - hw, top + h * 0.64,
                cx - hw, top + h * 1.08,
                cx + hw, top + h * 1.08,
                cx + hw, top + h * 0.64,
            );
            (api.GdipAddPathBezier)(
                p, cx + hw, top + h * 0.64,
                cx + hw, top + h * 0.44,
                cx + w * 0.06, top + h * 0.26,
                cx, top,
            );
            (api.GdipClosePathFigure)(p);
            self.fill_color(fill);
            (api.GdipFillPath)(self.g, self.brush, p);
            self.stroke(line, width.max(0.5));
            (api.GdipDrawPath)(self.g, self.pen, p);
        }
    }

    pub fn text(&mut self, s: &str, x: f32, y: f32, w: f32, h: f32, font: Gp, color: u32, align: i32) {
        if s.is_empty() {
            return;
        }
        self.utf16.clear();
        self.utf16.extend(s.encode_utf16());
        self.fill_color(color);
        let rect = RectF::new(x, y, w, h);
        let f = fmt(1, align);
        unsafe {
            (api().GdipDrawString)(
                self.g,
                self.utf16.as_mut_ptr(),
                self.utf16.len() as i32,
                font,
                &rect,
                f,
                self.brush,
            );
        }
    }

    pub fn measure(&mut self, s: &str, font: Gp) -> f32 {
        if s.is_empty() {
            return 0.0;
        }
        self.utf16.clear();
        self.utf16.extend(s.encode_utf16());
        let rect = RectF::new(0.0, 0.0, 4096.0, 4096.0);
        let mut bounds = RectF::default();
        let mut fitted = 0i32;
        let mut lines = 0i32;
        unsafe {
            (api().GdipMeasureString)(
                self.g,
                self.utf16.as_mut_ptr(),
                self.utf16.len() as i32,
                font,
                &rect,
                fmt(1, 0),
                &mut bounds,
                &mut fitted,
                &mut lines,
            );
        }
        bounds.w
    }

    pub fn clip(&mut self, x: f32, y: f32, w: f32, h: f32) {
        unsafe { (api().GdipSetClipRect)(self.g, x, y, w, h, CLIP_REPLACE) };
    }

    pub fn reset_clip(&mut self) {
        unsafe { (api().GdipResetClip)(self.g) };
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        let api = api();
        unsafe {
            (api.GdipDeleteBrush)(self.brush);
            (api.GdipDeletePen)(self.pen);
            (api.GdipDeletePath)(self.path);
            (api.GdipDeleteGraphics)(self.g);
        }
    }
}

// --------------------------------------------------------------- text setup

static mut FMT: [[Gp; 3]; 3] = [[std::ptr::null_mut(); 3]; 3];

/// `vertical` 0=top 1=center 2=bottom, `horizontal` 0=left 1=center 2=right.
fn fmt(vertical: usize, horizontal: i32) -> Gp {
    unsafe { FMT[vertical.min(2)][(horizontal as usize).min(2)] }
}

/// Builds the reusable string formats. Vertical alignment is encoded in the
/// slot we hand out; horizontal alignment defaults to near.
pub fn init_formats() {
    let api = api();
    unsafe {
        for (vi, flag) in [ALIGN_NEAR, ALIGN_CENTER, ALIGN_FAR].iter().enumerate() {
            for (hi, ha) in [ALIGN_NEAR, ALIGN_CENTER, ALIGN_FAR].iter().enumerate() {
                let mut fmt: Gp = std::ptr::null_mut();
                (api.GdipCreateStringFormat)(0, 0x0409, &mut fmt);
                (api.GdipSetStringFormatFlags)(fmt, 0x1000 | 0x4000); // NoWrap | NoClip
                (api.GdipSetStringFormatTrimming)(fmt, 1);
                (api.GdipSetStringFormatLineAlign)(fmt, *flag);
                (api.GdipSetStringFormatAlign)(fmt, *ha);
                FMT[vi][hi] = fmt;
            }
        }
    }
}

// ------------------------------------------------------------------- fonts

pub struct Fonts {
    pub family: Gp,
    cache: Vec<(u32, Gp)>,
}

impl Fonts {
    pub fn new(name: &str) -> Option<Fonts> {
        let api = api();
        let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            let mut fam: Gp = std::ptr::null_mut();
            if (api.GdipCreateFontFamilyFromName)(wide.as_ptr(), std::ptr::null_mut(), &mut fam) != 0 {
                return None;
            }
            Some(Fonts { family: fam, cache: Vec::new() })
        }
    }

    pub fn font(&mut self, px: f32, bold: bool) -> Gp {
        // Whole-pixel sizes keep GDI+ glyphs on the pixel grid, which is what
        // makes small text read crisply on a dark surface.
        let px = px.round().max(1.0);
        let key = ((px * 10.0) as u32) << 1 | bold as u32;
        for (k, f) in &self.cache {
            if *k == key {
                return *f;
            }
        }
        let api = api();
        unsafe {
            let mut f: Gp = std::ptr::null_mut();
            if (api.GdipCreateFont)(self.family, px, if bold { FONT_BOLD } else { FONT_REGULAR }, UNIT_PIXEL, &mut f) != 0 {
                return std::ptr::null_mut();
            }
            self.cache.push((key, f));
            f
        }
    }
}

impl Drop for Fonts {
    fn drop(&mut self) {
        let api = api();
        unsafe {
            for (_, f) in &self.cache {
                (api.GdipDeleteFont)(*f);
            }
            (api.GdipDeleteFontFamily)(self.family);
        }
    }
}

// ------------------------------------------------------------------- bitmap

/// A shared 32bpp top-down DIB used as back buffer for painting.
pub struct BackBuffer {
    pub dc: HDC,
    pub bmp: HBITMAP,
    pub old: HGDIOBJ,
    pub bits: *mut u32,
    pub w: i32,
    pub h: i32,
}

impl BackBuffer {
    pub fn new(w: i32, h: i32) -> Option<BackBuffer> {
        if w <= 0 || h <= 0 {
            return None;
        }
        unsafe {
            let screen = win::GetDC(0);
            let dc = win::CreateCompatibleDC(screen);
            win::ReleaseDC(0, screen);
            if dc == 0 {
                return None;
            }
            let mut bmi = win::BITMAPINFO::default();
            bmi.bmiHeader.biSize = std::mem::size_of::<win::BITMAPINFOHEADER>() as u32;
            bmi.bmiHeader.biWidth = w;
            bmi.bmiHeader.biHeight = -h;
            bmi.bmiHeader.biPlanes = 1;
            bmi.bmiHeader.biBitCount = 32;
            bmi.bmiHeader.biCompression = 0;
            let mut bits: *mut c_void = std::ptr::null_mut();
            let bmp = win::CreateDIBSection(dc, &bmi, win::DIB_RGB_COLORS, &mut bits, 0, 0);
            if bmp == 0 {
                win::DeleteDC(dc);
                return None;
            }
            let old = win::SelectObject(dc, bmp);
            Some(BackBuffer { dc, bmp, old, bits: bits as *mut u32, w, h })
        }
    }

    pub fn clear(&self, argb: u32) {
        // The DIB section's `bits` pointer is direct access to this app's own
        // pixel memory. A slice fill lets LLVM vectorize it and avoids a GDI
        // call (and any foreign-process memory access).
        let n = (self.w * self.h) as usize;
        unsafe {
            std::slice::from_raw_parts_mut(self.bits, n).fill(argb);
        }
    }
}

impl Drop for BackBuffer {
    fn drop(&mut self) {
        unsafe {
            win::SelectObject(self.dc, self.old);
            win::DeleteObject(self.bmp);
            win::DeleteDC(self.dc);
        }
    }
}
