//! Compositor-driven backdrop blur (frosted glass).
//!
//! Prefers the portable `ext-background-effect-v1` protocol (GNOME 51+,
//! Plasma 6.7+, recent Hyprland, etc.). Falls back to KWin's
//! `org_kde_kwin_blur`. Silently no-ops when the compositor has no blur global
//! (e.g. Ubuntu GNOME 46 / Mutter 46).

use gdk4_wayland::prelude::*;
use gdk4_wayland::{WaylandDisplay, WaylandToplevel};
use gtk4::glib::{self, translate::ToGlibPtr};
use gtk4::prelude::*;
use gtk4::ApplicationWindow;
use std::cell::RefCell;
use std::rc::Rc;
use wayland_client::protocol::wl_compositor::WlCompositor;
use wayland_client::{
    delegate_noop,
    globals::{registry_queue_init, GlobalListContents},
    protocol::wl_registry,
    Connection, Dispatch, Proxy, QueueHandle,
};
use wayland_protocols::ext::background_effect::v1::client::ext_background_effect_manager_v1::ExtBackgroundEffectManagerV1;
use wayland_protocols::ext::background_effect::v1::client::ext_background_effect_surface_v1::ExtBackgroundEffectSurfaceV1;
use wayland_protocols_plasma::blur::client::org_kde_kwin_blur::OrgKdeKwinBlur;
use wayland_protocols_plasma::blur::client::org_kde_kwin_blur_manager::OrgKdeKwinBlurManager;

enum BlurHandle {
    Ext(ExtBackgroundEffectSurfaceV1),
    Kwin(OrgKdeKwinBlur),
}

struct BlurState {
    handle: BlurHandle,
    compositor: WlCompositor,
    /// Kept alive so protocol objects stay registered on the shared display.
    _conn: Connection,
    queue: RefCell<wayland_client::EventQueue<BlurDispatch>>,
}

struct BlurDispatch;

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for BlurDispatch {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtBackgroundEffectManagerV1, ()> for BlurDispatch {
    fn event(
        _state: &mut Self,
        _proxy: &ExtBackgroundEffectManagerV1,
        _event: <ExtBackgroundEffectManagerV1 as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Capabilities are informational; we still request blur when the global exists.
    }
}

delegate_noop!(BlurDispatch: ignore ExtBackgroundEffectSurfaceV1);
delegate_noop!(BlurDispatch: ignore OrgKdeKwinBlurManager);
delegate_noop!(BlurDispatch: ignore OrgKdeKwinBlur);
delegate_noop!(BlurDispatch: ignore WlCompositor);
delegate_noop!(BlurDispatch: ignore wayland_client::protocol::wl_region::WlRegion);

fn wayland_connection(display: &WaylandDisplay) -> Option<Connection> {
    unsafe {
        let display_ptr =
            gdk4_wayland::ffi::gdk_wayland_display_get_wl_display(display.to_glib_none().0);
        if display_ptr.is_null() {
            return None;
        }
        let backend =
            wayland_backend::sys::client::Backend::from_foreign_display(display_ptr as *mut _);
        Some(Connection::from_backend(backend))
    }
}

fn set_full_blur_region(state: &BlurState, width: i32, height: i32) {
    if width <= 0 || height <= 0 {
        return;
    }

    let qh = state.queue.borrow().handle();
    let region = state.compositor.create_region(&qh, ());
    region.add(0, 0, width, height);

    match &state.handle {
        BlurHandle::Ext(effect) => {
            effect.set_blur_region(Some(&region));
        }
        BlurHandle::Kwin(blur) => {
            blur.set_region(Some(&region));
            blur.commit();
        }
    }

    region.destroy();

    let mut dispatch = BlurDispatch;
    let _ = state.queue.borrow_mut().dispatch_pending(&mut dispatch);
}

/// Request compositor backdrop blur behind the undecorated window.
pub fn install(window: &ApplicationWindow) {
    let blur_slot: Rc<RefCell<Option<Rc<BlurState>>>> = Rc::new(RefCell::new(None));

    window.connect_realize(glib::clone!(
        #[weak]
        window,
        #[strong]
        blur_slot,
        move |_| {
            let Some(state) = try_enable(&window) else {
                return;
            };
            set_full_blur_region(&state, window.width(), window.height());
            *blur_slot.borrow_mut() = Some(state.clone());

            if let Some(surface) = window.surface() {
                let slot = blur_slot.clone();
                surface.connect_layout(move |_surface, width, height| {
                    if let Some(state) = slot.borrow().as_ref() {
                        set_full_blur_region(state, width, height);
                    }
                });
            }
        }
    ));
}

fn try_enable(window: &ApplicationWindow) -> Option<Rc<BlurState>> {
    let gdk_surface = window.surface()?;
    let display = gdk_surface.display().downcast::<WaylandDisplay>().ok()?;

    let has_ext = display.query_registry("ext_background_effect_manager_v1");
    let has_kwin = display.query_registry("org_kde_kwin_blur_manager");
    if !has_ext && !has_kwin {
        eprintln!(
            "backdrop blur unavailable: compositor has no ext-background-effect-v1 or KWin blur"
        );
        return None;
    }

    let toplevel = gdk_surface.downcast_ref::<WaylandToplevel>()?;
    let wl_surface = toplevel.wl_surface()?;
    let compositor = display.wl_compositor()?;
    let conn = wayland_connection(&display)?;

    let (globals, mut queue) = registry_queue_init::<BlurDispatch>(&conn).ok()?;
    let qh = queue.handle();

    let handle = if has_ext {
        let manager = globals
            .bind::<ExtBackgroundEffectManagerV1, _, _>(&qh, 1..=1, ())
            .ok()?;
        let mut dispatch = BlurDispatch;
        let _ = queue.roundtrip(&mut dispatch);
        BlurHandle::Ext(manager.get_background_effect(&wl_surface, &qh, ()))
    } else {
        let manager = globals
            .bind::<OrgKdeKwinBlurManager, _, _>(&qh, 1..=1, ())
            .ok()?;
        BlurHandle::Kwin(manager.create(&wl_surface, &qh, ()))
    };

    Some(Rc::new(BlurState {
        handle,
        compositor,
        _conn: conn,
        queue: RefCell::new(queue),
    }))
}
