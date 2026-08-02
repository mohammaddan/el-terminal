//! Rounded translucent chrome with procedural noise, drawn behind the UI.

use gtk4::cairo::{Context, Format as CairoFormat, ImageSurface};
use gtk4::prelude::*;
use gtk4::DrawingArea;
use std::cell::RefCell;

const RADIUS: f64 = 16.0;
// Low enough alpha that compositor blur reads through as frosted glass.
const FILL: (f64, f64, f64, f64) = (0.051, 0.059, 0.071, 0.95);
const BORDER: (f64, f64, f64, f64) = (1.0, 1.0, 1.0, 0.10);

struct NoiseCache {
    width: i32,
    height: i32,
    surface: ImageSurface,
}

pub fn build_chrome_background() -> DrawingArea {
    let area = DrawingArea::new();
    area.set_hexpand(true);
    area.set_vexpand(true);
    area.set_can_target(false);
    area.add_css_class("window-chrome");

    let cache: RefCell<Option<NoiseCache>> = RefCell::new(None);

    area.set_draw_func(move |_area, cr, width, height| {
        let w = width as f64;
        let h = height as f64;
        if w <= 0.0 || h <= 0.0 {
            return;
        }

        cr.save().ok();
        rounded_rect(cr, 0.5, 0.5, w - 1.0, h - 1.0, RADIUS);
        cr.clip();

        // Frosted glass fill — same under header and content.
        cr.set_source_rgba(FILL.0, FILL.1, FILL.2, FILL.3);
        cr.paint().ok();

        // Cached grain overlay.
        {
            let mut slot = cache.borrow_mut();
            let needs_rebuild = slot
                .as_ref()
                .map(|c| c.width != width || c.height != height)
                .unwrap_or(true);
            if needs_rebuild {
                if let Some(surface) = build_noise_surface(width, height) {
                    *slot = Some(NoiseCache {
                        width,
                        height,
                        surface,
                    });
                }
            }
            if let Some(cached) = slot.as_ref() {
                cr.set_source_surface(&cached.surface, 0.0, 0.0).ok();
                cr.paint().ok();
            }
        }

        cr.restore().ok();

        // Soft rim so the rounded edge reads against the desktop.
        rounded_rect(cr, 0.5, 0.5, w - 1.0, h - 1.0, RADIUS);
        cr.set_source_rgba(BORDER.0, BORDER.1, BORDER.2, BORDER.3);
        cr.set_line_width(1.0);
        cr.stroke().ok();
    });

    area
}

fn build_noise_surface(width: i32, height: i32) -> Option<ImageSurface> {
    let surface = ImageSurface::create(CairoFormat::ARgb32, width, height).ok()?;
    {
        let cr = Context::new(&surface).ok()?;
        let step = 2;
        let mut y = 0;
        while y < height {
            let mut x = 0;
            while x < width {
                let n = hash2(x as u32, y as u32);
                let v = (n & 0xff) as f64 / 255.0;
                // Visible film grain without washing out the glass.
                let a = 0.01 + ((n >> 8) & 0x3f) as f64 / 255.0 * 0.03;
                cr.set_source_rgba(v, v, v, a);
                cr.rectangle(x as f64, y as f64, step as f64, step as f64);
                cr.fill().ok();
                x += step;
            }
            y += step;
        }
    }
    surface.flush();
    Some(surface)
}

fn hash2(x: u32, y: u32) -> u32 {
    let mut n = x
        .wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263));
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    n ^ (n >> 16)
}

fn rounded_rect(cr: &Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    let r = r.min(w * 0.5).min(h * 0.5);
    cr.new_sub_path();
    cr.arc(x + w - r, y + r, r, -std::f64::consts::FRAC_PI_2, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, std::f64::consts::FRAC_PI_2);
    cr.arc(
        x + r,
        y + h - r,
        r,
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
    );
    cr.arc(
        x + r,
        y + r,
        r,
        std::f64::consts::PI,
        3.0 * std::f64::consts::FRAC_PI_2,
    );
    cr.close_path();
}
