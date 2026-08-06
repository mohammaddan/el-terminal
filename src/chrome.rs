//! Rounded translucent chrome with procedural noise, drawn behind the UI.

use gtk4::cairo::{Context, Format as CairoFormat, ImageSurface};
use gtk4::prelude::*;
use gtk4::DrawingArea;
use std::cell::RefCell;
use std::rc::Rc;

struct ChromePaint {
    radius: f64,
    fill: (f64, f64, f64, f64),
    border: (f64, f64, f64, f64),
}

impl Default for ChromePaint {
    fn default() -> Self {
        Self {
            radius: 12.0,
            fill: (0.051, 0.059, 0.071, 0.93),
            border: (1.0, 1.0, 1.0, 0.20),
        }
    }
}

struct NoiseCache {
    width: i32,
    height: i32,
    surface: ImageSurface,
}

/// Handle to update chrome fill / border / radius at runtime.
#[derive(Clone)]
pub struct ChromeBackground {
    area: DrawingArea,
    paint: Rc<RefCell<ChromePaint>>,
}

impl ChromeBackground {
    pub fn widget(&self) -> &DrawingArea {
        &self.area
    }

    pub fn apply_style(
        &self,
        radius: f64,
        fill: [f64; 4],
        border: [f64; 4],
    ) {
        {
            let mut paint = self.paint.borrow_mut();
            paint.radius = radius;
            paint.fill = (fill[0], fill[1], fill[2], fill[3]);
            paint.border = (border[0], border[1], border[2], border[3]);
        }
        self.area.queue_draw();
    }
}

pub fn build_chrome_background() -> ChromeBackground {
    let area = DrawingArea::new();
    area.set_hexpand(true);
    area.set_vexpand(true);
    area.set_can_target(false);
    area.add_css_class("window-chrome");

    let paint = Rc::new(RefCell::new(ChromePaint::default()));
    let cache: RefCell<Option<NoiseCache>> = RefCell::new(None);
    let paint_draw = paint.clone();

    area.set_draw_func(move |_area, cr, width, height| {
        let w = width as f64;
        let h = height as f64;
        if w <= 0.0 || h <= 0.0 {
            return;
        }

        let paint = paint_draw.borrow();
        let radius = paint.radius;
        let fill = paint.fill;
        let border = paint.border;
        drop(paint);

        cr.save().ok();
        rounded_rect(cr, 0.5, 0.5, w - 1.0, h - 1.0, radius);
        cr.clip();

        // Frosted glass fill — same under header and content.
        cr.set_source_rgba(fill.0, fill.1, fill.2, fill.3);
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
        rounded_rect(cr, 0.5, 0.5, w - 1.0, h - 1.0, radius);
        cr.set_source_rgba(border.0, border.1, border.2, border.3);
        cr.set_line_width(1.0);
        cr.stroke().ok();
    });

    ChromeBackground { area, paint }
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
                let a = 0.01 + ((n >> 8) & 0x3f) as f64 / 255.0 * 0.02;
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
