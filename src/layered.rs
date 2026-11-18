//! Per-pixel layered-window composition. A hard GDI region is binary, so its
//! 12 px arc is a visible pixel staircase. This module publishes the same panel
//! through UpdateLayeredWindow with an antialiased alpha mask; the window still
//! paints into its existing 32bpp back buffer.

use crate::gp::BackBuffer;
use crate::win::{self, HWND};

/// Samples per pixel axis for the quarter-circle coverage. Four samples give
/// seventeen alpha levels across the rim, which removes the one-pixel spikes
/// that a single centre sample leaves on the right and bottom arcs.
const CORNER_SAMPLES: i32 = 4;

fn premultiply(pixel: u32, alpha: u32) -> u32 {
    let b = ((pixel & 0xFF) * alpha) / 255;
    let g = (((pixel >> 8) & 0xFF) * alpha) / 255;
    let r = (((pixel >> 16) & 0xFF) * alpha) / 255;
    (alpha << 24) | (r << 16) | (g << 8) | b
}

/// Fraction of one corner pixel covered by the rounded rectangle. `x` and `y`
/// are local to a `radius x radius` corner box whose outer corner is at
/// `(0, 0)` and whose circle centre is at `(radius, radius)`.
fn corner_coverage(radius: i32, x: i32, y: i32) -> f32 {
    let r = radius as f32;
    let samples = CORNER_SAMPLES as f32;
    let mut inside = 0u32;
    for sy in 0..CORNER_SAMPLES {
        for sx in 0..CORNER_SAMPLES {
            let fx = x as f32 + (sx as f32 + 0.5) / samples;
            let fy = y as f32 + (sy as f32 + 0.5) / samples;
            let dx = fx - r;
            let dy = fy - r;
            if dx * dx + dy * dy <= r * r {
                inside += 1;
            }
        }
    }
    inside as f32 / (samples * samples)
}

/// Alpha for a mirrored corner sample. `radius` is already clamped to the
/// surface; `mx`/`my` are local to the top-left corner box. Callers use this
/// same primitive at every DPI scale, so the mask geometry cannot drift
/// between paint, capture, and tests.
#[inline]
fn mirrored_alpha(radius: i32, mx: i32, my: i32, alpha: u8) -> u8 {
    if mx < 0 || my < 0 || mx >= radius || my >= radius {
        return alpha;
    }
    (alpha as f32 * corner_coverage(radius, mx, my)) as u8
}

/// Alpha of one pixel in the final layered surface. This is the testable
/// definition of the silhouette: mirror right/bottom pixels into one corner
/// box, then apply the same 4×4 coverage mask. The four outer corner pixels
/// are always zero and mirrored pixels always agree.
#[cfg(test)]
pub fn alpha_at(w: i32, h: i32, radius: i32, x: i32, y: i32, alpha: u8) -> u8 {
    if w <= 0 || h <= 0 || x < 0 || y < 0 || x >= w || y >= h {
        return 0;
    }
    let radius = radius.clamp(1, w.min(h) / 2);
    let mx = if x >= w - radius { w - 1 - x } else { x };
    let my = if y >= h - radius { h - 1 - y } else { y };
    mirrored_alpha(radius, mx, my, alpha)
}

/// Apply a rounded, antialiased alpha mask to the back buffer. Interior pixels
/// use `alpha`; the quarter-circles are coverage-sampled so the edge blends
/// into the desktop instead of showing a stair step or a corner spike.
pub unsafe fn prepare(back: &BackBuffer, radius: i32, alpha: u8) {
    let w = back.w.max(1);
    let h = back.h.max(1);
    let radius = radius.clamp(1, w.min(h) / 2);
    let panel_alpha = alpha as u32;
    let bits = std::slice::from_raw_parts_mut(back.bits, (w * h) as usize);

    // One pass: choose the alpha from the corner coverage, then premultiply
    // the existing GDI+ pixel. Mirroring each right/bottom pixel into the
    // top-left corner box makes all four arcs use the same centre and sample
    // grid; the previous centre calculation put the right/bottom circles on
    // the window edge and left visible spikes.
    for y in 0..h {
        let my = if y >= h - radius { h - 1 - y } else { y };
        let corner_y = my < radius;
        for x in 0..w {
            let mx = if x >= w - radius { w - 1 - x } else { x };
            let corner_x = mx < radius;
            let a = if corner_x && corner_y {
                mirrored_alpha(radius, mx, my, alpha) as u32
            } else {
                panel_alpha
            };
            let idx = (y * w + x) as usize;
            bits[idx] = premultiply(bits[idx] & 0x00FF_FFFF, a);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{alpha_at, corner_coverage};
    use crate::ui::{CORNER_RADIUS, WIN_H, WIN_W};

    #[test]
    fn corner_coverage_spans_empty_to_full() {
        assert_eq!(corner_coverage(12, 0, 0), 0.0);
        assert_eq!(corner_coverage(12, 11, 11), 1.0);
        assert!(corner_coverage(12, 3, 3) > 0.0);
        assert!(corner_coverage(12, 3, 3) < 1.0);
    }

    #[test]
    fn corner_coverage_is_symmetric() {
        for y in 0..12 {
            for x in 0..12 {
                assert!((corner_coverage(12, x, y) - corner_coverage(12, y, x)).abs() < 0.001);
            }
        }
    }

    #[test]
    fn all_dpi_corners_are_transparent_and_symmetric() {
        for scale in [1.0f32, 1.25, 1.5] {
            let w = (WIN_W * scale).round() as i32;
            let h = (WIN_H * scale).round() as i32;
            let radius = (CORNER_RADIUS * scale).round() as i32;
            let alpha = 250u8;

            // Every outer corner is fully transparent: no square pixel and no
            // upward spike can survive at any tested scale.
            assert_eq!(
                alpha_at(w, h, radius, 0, 0, alpha),
                0,
                "top-left at {scale}"
            );
            assert_eq!(
                alpha_at(w, h, radius, w - 1, 0, alpha),
                0,
                "top-right at {scale}"
            );
            assert_eq!(
                alpha_at(w, h, radius, 0, h - 1, alpha),
                0,
                "bottom-left at {scale}"
            );
            assert_eq!(
                alpha_at(w, h, radius, w - 1, h - 1, alpha),
                0,
                "bottom-right at {scale}"
            );

            // The whole corner box mirrors horizontally and vertically. This
            // is the regression that catches a right/bottom centre placed on
            // the window edge instead of one radius inside it.
            for y in 0..radius {
                for x in 0..radius {
                    let a = alpha_at(w, h, radius, x, y, alpha);
                    assert_eq!(a, alpha_at(w, h, radius, w - 1 - x, y, alpha));
                    assert_eq!(a, alpha_at(w, h, radius, x, h - 1 - y, alpha));
                    assert_eq!(a, alpha_at(w, h, radius, w - 1 - x, h - 1 - y, alpha));
                }
            }

            assert_eq!(alpha_at(w, h, radius, w / 2, h / 2, alpha), alpha);
        }
    }
}

/// Push the finished buffer to the compositor. Passing a null destination
/// keeps the existing window position; the size is the fixed 624x408 logical
/// surface scaled by DPI.
pub unsafe fn update(hwnd: HWND, back: &BackBuffer) {
    let size = win::SIZE {
        cx: back.w,
        cy: back.h,
    };
    let src = win::POINT { x: 0, y: 0 };
    let blend = win::BLENDFUNCTION {
        BlendOp: win::AC_SRC_OVER,
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: win::AC_SRC_ALPHA,
    };
    let screen = win::GetDC(0);
    win::UpdateLayeredWindow(
        hwnd,
        screen,
        std::ptr::null(),
        &size,
        back.dc,
        &src,
        0,
        &blend,
        win::ULW_ALPHA,
    );
    win::ReleaseDC(0, screen);
}
