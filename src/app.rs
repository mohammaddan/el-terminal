use crate::ask::{self, AskPanel};
use crate::chrome::{self, ChromeBackground};
use crate::context_menu;
use crate::launch;
use crate::settings::{AppSettings, FONT_SIZE_MAX, FONT_SIZE_MIN};
use crate::settings_ui;
use crate::tab_bar;
use crate::terminal_links;
use crate::terminal_tab::{self, SpawnOpts};
use gtk4::gdk::{self, Key, ModifierType};
use gtk4::gio;
use gtk4::glib::{self, clone};
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, CssProvider, EventControllerKey,
    EventControllerMotion, EventControllerScroll, EventControllerScrollFlags, EventSequenceState,
    GestureClick, Orientation, Overlay, Paned, PropagationPhase, Stack, StackTransitionType,
    Widget, STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use std::cell::{Cell, RefCell};
use std::env;
use std::path::Path;
use std::rc::Rc;
use vte4::prelude::*;
use vte4::Terminal;

struct Pane {
    id: u32,
    terminal: Terminal,
}

struct Tab {
    name: String,
    root: Widget,
    panes: Vec<Pane>,
    focused: Cell<u32>,
    pill: GtkBox,
    label: gtk4::Label,
}

struct AppState {
    window: ApplicationWindow,
    tab_strip: GtkBox,
    stack: Stack,
    tabs: RefCell<Vec<Tab>>,
    next_id: Cell<u32>,
    active: Cell<usize>,
    settings: RefCell<AppSettings>,
    chrome: ChromeBackground,
    style_provider: CssProvider,
    ask_panel: RefCell<Option<Rc<AskPanel>>>,
    /// Consumed by the first tab only (`--command` / `-e`).
    pending_command: RefCell<Option<String>>,
    /// Absolute cwd for the first pane (`--working-directory` / `--dir`).
    pending_cwd: RefCell<Option<String>>,
}

enum PaneDir {
    Left,
    Right,
    Up,
    Down,
}

pub fn build_ui(app: &Application) {
    let launch_opts = launch::take();
    if let Some(dir) = launch_opts.working_directory.as_deref() {
        apply_working_directory(dir);
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title("El-Terminal")
        .default_width(960)
        .default_height(640)
        .decorated(false)
        .build();
    window.add_css_class("terminal-window");
    window.set_resizable(true);
    window.set_icon_name(Some("el-terminal"));

    // Transparent outer surface; rounded frosted chrome is painted underneath.
    let overlay = Overlay::new();
    overlay.set_hexpand(true);
    overlay.set_vexpand(true);

    let chrome_bg = chrome::build_chrome_background();
    overlay.set_child(Some(chrome_bg.widget()));

    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("window-chrome");
    root.set_hexpand(true);
    root.set_vexpand(true);

    let top_bar = GtkBox::new(Orientation::Horizontal, 6);
    top_bar.add_css_class("top-bar");
    top_bar.set_hexpand(true);

    let tab_strip = GtkBox::new(Orientation::Horizontal, 4);
    tab_strip.add_css_class("tab-strip");
    tab_strip.set_halign(gtk4::Align::Start);
    tab_strip.set_visible(false);

    let new_tab_btn = tab_bar::build_new_tab_button();
    let ask_btn = ask::build_ask_button();
    let menu_btn = tab_bar::build_menu_button();
    let status_dot = tab_bar::build_status_dot();
    let (window_controls, minimize_btn, maximize_btn, close_win_btn) =
        tab_bar::build_window_controls();

    let chrome_actions = GtkBox::new(Orientation::Horizontal, 6);
    chrome_actions.set_valign(gtk4::Align::Center);
    chrome_actions.append(&new_tab_btn);
    chrome_actions.append(&ask_btn);
    chrome_actions.append(&menu_btn);
    chrome_actions.append(&status_dot);
    chrome_actions.append(&window_controls);

    // GTK4 Box packs start→end; Align::End does not push widgets right.
    // An expanding spacer keeps chrome actions on the right when the tab strip is hidden.
    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    top_bar.append(&tab_strip);
    top_bar.append(&spacer);
    top_bar.append(&chrome_actions);

    let host = GtkBox::new(Orientation::Vertical, 0);
    host.add_css_class("terminal-host");
    host.set_hexpand(true);
    host.set_vexpand(true);

    let stack = Stack::new();
    stack.set_transition_type(StackTransitionType::None);
    stack.set_hexpand(true);
    stack.set_vexpand(true);
    host.append(&stack);

    // Content row: terminal host + Ask side panel
    let content_row = GtkBox::new(Orientation::Horizontal, 0);
    content_row.set_hexpand(true);
    content_row.set_vexpand(true);
    content_row.append(&host);

    let ask_panel = AskPanel::build();
    content_row.append(ask_panel.widget());

    root.append(&top_bar);
    root.append(&content_row);
    overlay.add_overlay(&root);

    window.set_child(Some(&overlay));

    // Ensure the surface stays clear once mapped (needed for rounded corners).
    window.connect_realize(|window| {
        if let Some(surface) = window.surface() {
            // No opaque region — let compositor see alpha outside the rounded clip.
            surface.set_opaque_region(None);
        }
    });

    // Ask the compositor for backdrop blur when the protocol is available.
    crate::blur::install(&window);

    let settings = AppSettings::load();

    let style_provider = CssProvider::new();
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("display"),
        &style_provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
    );

    let state = Rc::new(AppState {
        window: window.clone(),
        tab_strip,
        stack,
        tabs: RefCell::new(Vec::new()),
        next_id: Cell::new(0),
        active: Cell::new(0),
        settings: RefCell::new(settings),
        chrome: chrome_bg,
        style_provider,
        ask_panel: RefCell::new(Some(ask_panel.clone())),
        pending_command: RefCell::new(launch_opts.command),
        pending_cwd: RefCell::new(launch_opts.working_directory),
    });
    apply_theme_style(&state);

    AskPanel::connect_ask(
        &ask_panel,
        clone!(
            #[strong]
            state,
            move || state.settings.borrow().clone()
        ),
        clone!(
            #[strong]
            state,
            move || active_terminal(&state)
        ),
        clone!(
            #[strong]
            state,
            move || open_settings(&state)
        ),
    );

    install_drag(&top_bar, &state.window);
    install_resize(&overlay, &state.window);
    install_menu(&menu_btn, &state);
    install_window_controls(
        &state.window,
        &minimize_btn,
        &maximize_btn,
        &close_win_btn,
    );
    install_actions(&state);
    install_shortcuts(app, &state);

    {
        let s = state.clone();
        new_tab_btn.connect_clicked(move |_| add_tab(&s));
    }
    {
        let s = state.clone();
        ask_btn.connect_clicked(move |_| toggle_ask(&s));
    }

    add_tab(&state);
    state.window.present();
}

fn apply_working_directory(dir: &str) {
    let path = Path::new(dir);
    if let Err(err) = env::set_current_dir(path) {
        eprintln!("el-terminal: failed to set working directory to {dir}: {err}");
    }
}

const RESIZE_MARGIN: f64 = 8.0;

fn install_resize(overlay: &Overlay, window: &ApplicationWindow) {
    let motion = EventControllerMotion::new();
    motion.set_propagation_phase(PropagationPhase::Capture);
    motion.connect_motion(clone!(
        #[weak]
        overlay,
        #[weak]
        window,
        move |_, x, y| {
            let cursor = if window.is_maximized() {
                None
            } else {
                resize_edge_at(&overlay, x, y).map(resize_cursor_name)
            };
            overlay.set_cursor_from_name(cursor);
        }
    ));
    motion.connect_leave(clone!(
        #[weak]
        overlay,
        move |_| {
            overlay.set_cursor_from_name(None);
        }
    ));
    overlay.add_controller(motion);

    let gesture = GestureClick::new();
    gesture.set_button(1);
    gesture.set_propagation_phase(PropagationPhase::Capture);
    gesture.connect_pressed(clone!(
        #[weak]
        overlay,
        #[weak]
        window,
        move |gesture, n_press, x, y| {
            if n_press != 1 || window.is_maximized() {
                return;
            }
            let Some(edge) = resize_edge_at(&overlay, x, y) else {
                return;
            };
            let Some(event) = gesture.current_event() else {
                return;
            };
            let Some(device) = event.device() else {
                return;
            };
            let Some(native) = window.native() else {
                return;
            };
            let Some(surface) = native.surface() else {
                return;
            };
            let Ok(toplevel) = surface.downcast::<gdk::Toplevel>() else {
                return;
            };
            let (sx, sy) = overlay
                .compute_point(&window, &gtk4::graphene::Point::new(x as f32, y as f32))
                .map(|p| (f64::from(p.x()), f64::from(p.y())))
                .unwrap_or((x, y));
            toplevel.begin_resize(edge, Some(&device), 1, sx, sy, event.time());
            gesture.set_state(EventSequenceState::Claimed);
        }
    ));
    overlay.add_controller(gesture);
}

fn resize_edge_at(overlay: &Overlay, x: f64, y: f64) -> Option<gdk::SurfaceEdge> {
    if hit_interactive_child(overlay, x, y) {
        return None;
    }
    let width = f64::from(overlay.width());
    let height = f64::from(overlay.height());
    if width < RESIZE_MARGIN * 2.0 || height < RESIZE_MARGIN * 2.0 {
        return None;
    }
    let left = x <= RESIZE_MARGIN;
    let right = x >= width - RESIZE_MARGIN;
    let top = y <= RESIZE_MARGIN;
    let bottom = y >= height - RESIZE_MARGIN;
    match (top, bottom, left, right) {
        (true, false, true, false) => Some(gdk::SurfaceEdge::NorthWest),
        (true, false, false, true) => Some(gdk::SurfaceEdge::NorthEast),
        (false, true, true, false) => Some(gdk::SurfaceEdge::SouthWest),
        (false, true, false, true) => Some(gdk::SurfaceEdge::SouthEast),
        (true, false, false, false) => Some(gdk::SurfaceEdge::North),
        (false, true, false, false) => Some(gdk::SurfaceEdge::South),
        (false, false, true, false) => Some(gdk::SurfaceEdge::West),
        (false, false, false, true) => Some(gdk::SurfaceEdge::East),
        _ => None,
    }
}

fn resize_cursor_name(edge: gdk::SurfaceEdge) -> &'static str {
    match edge {
        gdk::SurfaceEdge::NorthWest => "nw-resize",
        gdk::SurfaceEdge::North => "n-resize",
        gdk::SurfaceEdge::NorthEast => "ne-resize",
        gdk::SurfaceEdge::West => "w-resize",
        gdk::SurfaceEdge::East => "e-resize",
        gdk::SurfaceEdge::SouthWest => "sw-resize",
        gdk::SurfaceEdge::South => "s-resize",
        gdk::SurfaceEdge::SouthEast => "se-resize",
        _ => "default",
    }
}

fn install_drag(top_bar: &GtkBox, window: &ApplicationWindow) {
    let gesture = GestureClick::new();
    gesture.set_button(1);
    gesture.connect_pressed(clone!(
        #[weak]
        window,
        move |gesture, n_press, x, y| {
            if n_press != 1 {
                return;
            }
            // Ignore presses that landed on interactive children (tabs/buttons).
            if let Some(widget) = gesture.widget() {
                if hit_interactive_child(&widget, x, y) {
                    return;
                }
            }
            let Some(event) = gesture.current_event() else {
                return;
            };
            let Some(device) = event.device() else {
                return;
            };
            let Some(native) = window.native() else {
                return;
            };
            let Some(surface) = native.surface() else {
                return;
            };
            let Ok(toplevel) = surface.downcast::<gdk::Toplevel>() else {
                return;
            };
            toplevel.begin_move(&device, 1, x, y, event.time());
        }
    ));
    top_bar.add_controller(gesture);
}

fn hit_interactive_child(top_bar: &impl IsA<gtk4::Widget>, x: f64, y: f64) -> bool {
    let Some(picked) = top_bar.pick(x, y, gtk4::PickFlags::DEFAULT) else {
        return false;
    };
    let top = top_bar.upcast_ref::<gtk4::Widget>();
    let mut current = Some(picked);
    while let Some(widget) = current {
        if widget == *top {
            break;
        }
        if widget.has_css_class("tab-pill")
            || widget.has_css_class("tab-close")
            || widget.has_css_class("new-tab-btn")
            || widget.has_css_class("menu-btn")
            || widget.has_css_class("ask-btn")
            || widget.has_css_class("status-dot")
            || widget.has_css_class("window-control")
            || widget.has_css_class("window-controls")
            || widget.is::<gtk4::Button>()
        {
            return true;
        }
        current = widget.parent();
    }
    false
}

fn install_window_controls(
    window: &ApplicationWindow,
    minimize_btn: &gtk4::Button,
    maximize_btn: &gtk4::Button,
    close_btn: &gtk4::Button,
) {
    minimize_btn.connect_clicked(clone!(
        #[weak]
        window,
        move |_| {
            window.minimize();
        }
    ));

    maximize_btn.connect_clicked(clone!(
        #[weak]
        window,
        #[weak]
        maximize_btn,
        move |_| {
            if window.is_maximized() {
                window.unmaximize();
                maximize_btn.set_icon_name("window-maximize-symbolic");
                maximize_btn.set_tooltip_text(Some("Maximize"));
            } else {
                window.maximize();
                maximize_btn.set_icon_name("window-restore-symbolic");
                maximize_btn.set_tooltip_text(Some("Restore"));
            }
        }
    ));

    // Keep maximize icon in sync if the window is toggled externally.
    window.connect_maximized_notify(clone!(
        #[weak]
        maximize_btn,
        move |window| {
            if window.is_maximized() {
                maximize_btn.set_icon_name("window-restore-symbolic");
                maximize_btn.set_tooltip_text(Some("Restore"));
            } else {
                maximize_btn.set_icon_name("window-maximize-symbolic");
                maximize_btn.set_tooltip_text(Some("Maximize"));
            }
        }
    ));

    close_btn.connect_clicked(clone!(
        #[weak]
        window,
        move |_| {
            window.close();
        }
    ));
}

fn install_menu(menu_btn: &gtk4::Button, state: &Rc<AppState>) {
    let menu = gio::Menu::new();
    menu.append(Some("New Tab"), Some("win.new-tab"));
    menu.append(Some("Previous Tab"), Some("win.prev-tab"));
    menu.append(Some("Next Tab"), Some("win.next-tab"));
    menu.append(Some("Close Tab"), Some("win.close-tab"));
    menu.append(Some("Split Right"), Some("win.split-right"));
    menu.append(Some("Split Down"), Some("win.split-down"));
    menu.append(Some("Ask…"), Some("win.ask"));
    menu.append(Some("Settings…"), Some("win.settings"));
    menu.append(Some("Quit"), Some("win.quit"));

    let popover = gtk4::PopoverMenu::from_model(Some(&menu));
    popover.set_parent(menu_btn);
    popover.set_has_arrow(false);
    popover.add_css_class("menu");

    menu_btn.connect_clicked(clone!(
        #[strong]
        popover,
        move |_| {
            popover.popup();
        }
    ));

    let quit = gio::SimpleAction::new("quit", None);
    let window = state.window.clone();
    quit.connect_activate(move |_, _| {
        window.close();
    });
    state.window.add_action(&quit);

    let settings_action = gio::SimpleAction::new("settings", None);
    settings_action.connect_activate(clone!(
        #[strong]
        state,
        move |_, _| open_settings(&state)
    ));
    state.window.add_action(&settings_action);

    let ask_action = gio::SimpleAction::new("ask", None);
    ask_action.connect_activate(clone!(
        #[strong]
        state,
        move |_, _| toggle_ask(&state)
    ));
    state.window.add_action(&ask_action);
}

fn install_actions(state: &Rc<AppState>) {
    context_menu::install_window_actions(
        &state.window,
        clone!(
            #[strong]
            state,
            move || {
                if let Some(term) = active_terminal(&state) {
                    terminal_tab::copy_selection(&term);
                }
            }
        ),
        clone!(
            #[strong]
            state,
            move || {
                if let Some(term) = active_terminal(&state) {
                    terminal_tab::paste_clipboard(&term);
                }
            }
        ),
        clone!(
            #[strong]
            state,
            move || {
                if let Some(term) = active_terminal(&state) {
                    terminal_tab::select_all(&term);
                }
            }
        ),
        clone!(
            #[strong]
            state,
            move || add_tab(&state)
        ),
        clone!(
            #[strong]
            state,
            move || close_active_tab(&state)
        ),
        clone!(
            #[strong]
            state,
            move || split_active(&state, Orientation::Horizontal)
        ),
        clone!(
            #[strong]
            state,
            move || split_active(&state, Orientation::Vertical)
        ),
        clone!(
            #[strong]
            state,
            move || close_active_pane(&state)
        ),
    );

    context_menu::install_navigation_actions(
        &state.window,
        clone!(
            #[strong]
            state,
            move || cycle_tab(&state, -1)
        ),
        clone!(
            #[strong]
            state,
            move || cycle_tab(&state, 1)
        ),
        clone!(
            #[strong]
            state,
            move || focus_pane_in_direction(&state, PaneDir::Left)
        ),
        clone!(
            #[strong]
            state,
            move || focus_pane_in_direction(&state, PaneDir::Right)
        ),
        clone!(
            #[strong]
            state,
            move || focus_pane_in_direction(&state, PaneDir::Up)
        ),
        clone!(
            #[strong]
            state,
            move || focus_pane_in_direction(&state, PaneDir::Down)
        ),
    );
}

fn install_shortcuts(app: &Application, state: &Rc<AppState>) {
    app.set_accels_for_action("win.copy", &["<Control><Shift>c"]);
    app.set_accels_for_action("win.paste", &["<Control><Shift>v"]);
    app.set_accels_for_action("win.select-all", &["<Control><Shift>a"]);
    app.set_accels_for_action("win.new-tab", &["<Control><Shift>t"]);
    app.set_accels_for_action("win.split-right", &["<Control><Shift>r"]);
    app.set_accels_for_action("win.split-down", &["<Control><Shift>d"]);
    // Closes the focused pane; closes the tab when only one pane remains.
    app.set_accels_for_action("win.close-pane", &["<Control><Shift>w"]);
    app.set_accels_for_action("win.quit", &["<Control><Shift>q"]);
    app.set_accels_for_action("win.settings", &["<Control><Shift>comma"]);
    app.set_accels_for_action("win.ask", &["<Control><Shift>slash"]);
    app.set_accels_for_action("win.prev-tab", &["<Control>Page_Up", "<Control>KP_Page_Up"]);
    app.set_accels_for_action(
        "win.next-tab",
        &["<Control>Page_Down", "<Control>KP_Page_Down"],
    );
    app.set_accels_for_action("win.focus-pane-left", &["<Alt>Left", "<Alt>KP_Left"]);
    app.set_accels_for_action("win.focus-pane-right", &["<Alt>Right", "<Alt>KP_Right"]);
    app.set_accels_for_action("win.focus-pane-up", &["<Alt>Up", "<Alt>KP_Up"]);
    app.set_accels_for_action("win.focus-pane-down", &["<Alt>Down", "<Alt>KP_Down"]);

    // Ensure VTE doesn't swallow window shortcuts before actions.
    let controller = EventControllerKey::new();
    controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    controller.connect_key_pressed(clone!(
        #[strong]
        state,
        move |_, key, _code, mods| {
            let ctrl = mods.contains(ModifierType::CONTROL_MASK);
            let shift = mods.contains(ModifierType::SHIFT_MASK);
            let alt = mods.contains(ModifierType::ALT_MASK);

            if ctrl && !shift && !alt {
                match key {
                    Key::Page_Up | Key::KP_Page_Up => {
                        cycle_tab(&state, -1);
                        return glib::Propagation::Stop;
                    }
                    Key::Page_Down | Key::KP_Page_Down => {
                        cycle_tab(&state, 1);
                        return glib::Propagation::Stop;
                    }
                    _ => {}
                }
            }

            if alt && !ctrl {
                match key {
                    Key::Left | Key::KP_Left => {
                        focus_pane_in_direction(&state, PaneDir::Left);
                        return glib::Propagation::Stop;
                    }
                    Key::Right | Key::KP_Right => {
                        focus_pane_in_direction(&state, PaneDir::Right);
                        return glib::Propagation::Stop;
                    }
                    Key::Up | Key::KP_Up => {
                        focus_pane_in_direction(&state, PaneDir::Up);
                        return glib::Propagation::Stop;
                    }
                    Key::Down | Key::KP_Down => {
                        focus_pane_in_direction(&state, PaneDir::Down);
                        return glib::Propagation::Stop;
                    }
                    _ => {}
                }
            }

            let ctrl_shift = ctrl && shift;
            if !ctrl_shift {
                return glib::Propagation::Proceed;
            }
            match key {
                Key::c | Key::C => {
                    if let Some(term) = active_terminal(&state) {
                        terminal_tab::copy_selection(&term);
                    }
                    glib::Propagation::Stop
                }
                Key::v | Key::V => {
                    if let Some(term) = active_terminal(&state) {
                        terminal_tab::paste_clipboard(&term);
                    }
                    glib::Propagation::Stop
                }
                Key::a | Key::A => {
                    if let Some(term) = active_terminal(&state) {
                        terminal_tab::select_all(&term);
                    }
                    glib::Propagation::Stop
                }
                Key::t | Key::T => {
                    add_tab(&state);
                    glib::Propagation::Stop
                }
                Key::w | Key::W => {
                    close_active_pane(&state);
                    glib::Propagation::Stop
                }
                Key::r | Key::R => {
                    split_active(&state, Orientation::Horizontal);
                    glib::Propagation::Stop
                }
                Key::d | Key::D => {
                    split_active(&state, Orientation::Vertical);
                    glib::Propagation::Stop
                }
                Key::q | Key::Q => {
                    state.window.close();
                    glib::Propagation::Stop
                }
                Key::comma => {
                    open_settings(&state);
                    glib::Propagation::Stop
                }
                Key::slash | Key::question => {
                    toggle_ask(&state);
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        }
    ));
    state.window.add_controller(controller);
}

fn open_settings(state: &Rc<AppState>) {
    let current = state.settings.borrow().clone();
    settings_ui::open_settings_dialog(
        &state.window,
        &current,
        clone!(
            #[strong]
            state,
            move |preview| {
                *state.settings.borrow_mut() = preview;
                apply_settings_to_all(&state);
            }
        ),
        clone!(
            #[strong]
            state,
            move |new_settings| {
                if let Err(err) = new_settings.save() {
                    eprintln!("failed to save settings: {err}");
                }
                *state.settings.borrow_mut() = new_settings;
                apply_settings_to_all(&state);
            }
        ),
    );
}

fn toggle_ask(state: &AppState) {
    if let Some(panel) = state.ask_panel.borrow().as_ref() {
        panel.toggle();
    }
}

fn apply_theme_style(state: &AppState) {
    let settings = state.settings.borrow();
    state
        .style_provider
        .load_from_data(&settings.style.to_css());
    state.chrome.apply_style(
        settings.style.window_radius,
        settings.style.chrome_fill,
        settings.style.window_border,
    );
}

fn apply_settings_to_all(state: &AppState) {
    apply_theme_style(state);
    let settings = state.settings.borrow().clone();
    let tabs = state.tabs.borrow();
    for tab in tabs.iter() {
        for pane in &tab.panes {
            terminal_tab::apply_font(&pane.terminal, &settings);
            terminal_tab::apply_palette(&pane.terminal, &settings.style);
            terminal_links::set_link_color(&pane.terminal, &settings.style.accent_blue);
        }
    }
}

fn adjust_font_size(state: &AppState, delta: i32) {
    let mut settings = state.settings.borrow_mut();
    let next = (settings.font_size as i32 + delta).clamp(FONT_SIZE_MIN as i32, FONT_SIZE_MAX as i32)
        as u32;
    if next == settings.font_size {
        return;
    }
    settings.font_size = next;
    let snapshot = settings.clone();
    drop(settings);
    if let Err(err) = snapshot.save() {
        eprintln!("failed to save settings: {err}");
    }
    *state.settings.borrow_mut() = snapshot.clone();
    let tabs = state.tabs.borrow();
    for tab in tabs.iter() {
        for pane in &tab.panes {
            terminal_tab::apply_font(&pane.terminal, &snapshot);
        }
    }
}

fn active_terminal(state: &AppState) -> Option<Terminal> {
    let tabs = state.tabs.borrow();
    let tab = tabs.get(state.active.get())?;
    let focused = tab.focused.get();
    tab.panes
        .iter()
        .find(|p| p.id == focused)
        .map(|p| p.terminal.clone())
}

fn next_id(state: &AppState) -> u32 {
    let id = state.next_id.get();
    state.next_id.set(id + 1);
    id
}

fn update_tabs_visibility(state: &AppState) {
    let show = state.tabs.borrow().len() > 1;
    state.tab_strip.set_visible(show);
}

fn add_tab(state: &Rc<AppState>) {
    let id = next_id(state);
    let name = format!("tab-{id}");
    let accent = terminal_tab::ACCENT_CLASSES[(id as usize) % terminal_tab::ACCENT_CLASSES.len()];

    let pane_id = next_id(state);
    let command = state.pending_command.borrow_mut().take();
    let working_directory = state.pending_cwd.borrow_mut().take();
    let terminal = terminal_tab::create_terminal_with(
        &state.settings.borrow(),
        SpawnOpts {
            working_directory,
            command,
        },
    );
    let title = terminal_tab::default_title();

    let (pill, label, close_btn) = tab_bar::build_tab_pill(&title, accent, false);
    state.stack.add_named(&terminal, Some(&name));
    state.tab_strip.append(&pill);

    wire_pane(state, &name, pane_id, &terminal);

    {
        let click = GestureClick::new();
        click.set_button(1);
        click.connect_pressed(clone!(
            #[strong]
            state,
            #[strong]
            name,
            move |gesture, _, x, y| {
                // Don't switch when clicking the close button.
                if let Some(widget) = gesture.widget() {
                    if let Some(picked) = widget.pick(x, y, gtk4::PickFlags::DEFAULT) {
                        let mut cur = Some(picked);
                        while let Some(w) = cur {
                            if w.is::<gtk4::Button>() || w.has_css_class("tab-close") {
                                return;
                            }
                            if w.has_css_class("tab-pill") {
                                break;
                            }
                            cur = w.parent();
                        }
                    }
                }
                select_tab_by_name(&state, &name);
            }
        ));
        pill.add_controller(click);
    }

    {
        let s = state.clone();
        let name_c = name.clone();
        close_btn.connect_clicked(move |_| close_tab_by_name(&s, &name_c));
    }

    state.tabs.borrow_mut().push(Tab {
        name: name.clone(),
        root: terminal.clone().upcast(),
        panes: vec![Pane {
            id: pane_id,
            terminal: terminal.clone(),
        }],
        focused: Cell::new(pane_id),
        pill,
        label,
    });

    update_tabs_visibility(state);
    select_tab_by_name(state, &name);
    terminal.grab_focus();
}

fn wire_pane(state: &Rc<AppState>, tab_name: &str, pane_id: u32, terminal: &Terminal) {
    terminal_links::wire_clicks(terminal, &state.window);

    context_menu::attach_context_menu(
        terminal,
        clone!(
            #[strong]
            state,
            #[strong]
            terminal,
            move || focus_pane_terminal(&state, &terminal)
        ),
    );

    {
        let s = state.clone();
        let name_c = tab_name.to_string();
        let pid = pane_id;
        terminal.connect_notify_local(
            Some("has-focus"),
            clone!(
                #[strong]
                s,
                move |term, _| {
                    if term.has_focus() {
                        focus_pane_in_tab(&s, &name_c, pid);
                    }
                }
            ),
        );
    }

    {
        let s = state.clone();
        let name_c = tab_name.to_string();
        let pid = pane_id;
        terminal.connect_window_title_changed(move |term| {
            let title = terminal_tab::title_from_terminal(term);
            let tabs = s.tabs.borrow();
            if let Some(tab) = tabs.iter().find(|t| t.name == name_c) {
                if tab.focused.get() == pid {
                    tab.label.set_text(&title);
                }
            }
        });
    }

    {
        let s = state.clone();
        let name_c = tab_name.to_string();
        let pid = pane_id;
        terminal.connect_child_exited(move |_, _| {
            // Ctrl+D / shell EOF: quit when this was the last live session
            // (no other panes, tabs, or nested shells left in the window).
            let alone = {
                let tabs = s.tabs.borrow();
                tabs.len() == 1
                    && tabs
                        .first()
                        .is_some_and(|t| t.panes.len() == 1 && t.panes[0].id == pid)
            };
            if alone {
                s.window.close();
            } else {
                close_pane_by_id(&s, &name_c, pid);
            }
        });
    }

    // Ctrl+scroll adjusts font size without stealing normal scrollback.
    let scroll = EventControllerScroll::new(
        EventControllerScrollFlags::VERTICAL | EventControllerScrollFlags::DISCRETE,
    );
    scroll.set_propagation_phase(gtk4::PropagationPhase::Capture);
    scroll.connect_scroll(clone!(
        #[strong]
        state,
        move |controller, _dx, dy| {
            let Some(event) = controller.current_event() else {
                return glib::Propagation::Proceed;
            };
            if !event.modifier_state().contains(ModifierType::CONTROL_MASK) {
                return glib::Propagation::Proceed;
            }
            let delta = if dy < 0.0 {
                1
            } else if dy > 0.0 {
                -1
            } else {
                return glib::Propagation::Proceed;
            };
            adjust_font_size(&state, delta);
            glib::Propagation::Stop
        }
    ));
    terminal.add_controller(scroll);

    // In-shell Ask: type `<prefix> question` (default `??`) and press Enter.
    let ask_keys = EventControllerKey::new();
    ask_keys.set_propagation_phase(gtk4::PropagationPhase::Capture);
    ask_keys.connect_key_pressed(clone!(
        #[strong]
        state,
        #[strong]
        terminal,
        move |_, key, _code, _mods| {
            if key != Key::Return && key != Key::KP_Enter {
                return glib::Propagation::Proceed;
            }

            let Some(line) = ask::current_input_line(&terminal) else {
                return glib::Propagation::Proceed;
            };
            let prefix = state.settings.borrow().ask_prefix.clone();
            let Some(question) = ask::extract_shell_ask_query(&line, &prefix) else {
                return glib::Propagation::Proceed;
            };

            // Don't send Enter to the shell — clear the typed Ask line instead.
            ask::clear_shell_input_line(&terminal);

            let settings = state.settings.borrow().clone();
            ask::run_shell_ask(
                &terminal,
                &question,
                &settings,
                clone!(
                    #[strong]
                    state,
                    move || open_settings(&state)
                ),
            );

            glib::Propagation::Stop
        }
    ));
    terminal.add_controller(ask_keys);
}

fn focus_pane_terminal(state: &AppState, terminal: &Terminal) {
    let tabs = state.tabs.borrow();
    for (i, tab) in tabs.iter().enumerate() {
        if let Some(pane) = tab.panes.iter().find(|p| p.terminal == *terminal) {
            tab.focused.set(pane.id);
            let name = tab.name.clone();
            let title = terminal_tab::title_from_terminal(terminal);
            tab.label.set_text(&title);
            drop(tabs);
            if state.active.get() != i {
                select_tab_by_name(state, &name);
            }
            return;
        }
    }
}

fn focus_pane_in_tab(state: &AppState, tab_name: &str, pane_id: u32) {
    let tabs = state.tabs.borrow();
    let Some(tab) = tabs.iter().find(|t| t.name == tab_name) else {
        return;
    };
    tab.focused.set(pane_id);
    if let Some(pane) = tab.panes.iter().find(|p| p.id == pane_id) {
        tab.label
            .set_text(&terminal_tab::title_from_terminal(&pane.terminal));
    }
}

fn select_tab_by_name(state: &AppState, name: &str) {
    let tabs = state.tabs.borrow();
    let Some(index) = tabs.iter().position(|t| t.name == name) else {
        return;
    };
    for (i, tab) in tabs.iter().enumerate() {
        if i == index {
            tab.pill.add_css_class("active");
        } else {
            tab.pill.remove_css_class("active");
        }
    }
    state.stack.set_visible_child_name(name);
    state.active.set(index);
    if let Some(tab) = tabs.get(index) {
        let focused = tab.focused.get();
        if let Some(pane) = tab.panes.iter().find(|p| p.id == focused) {
            pane.terminal.grab_focus();
        }
    }
}

fn cycle_tab(state: &AppState, delta: isize) {
    let tabs = state.tabs.borrow();
    let n = tabs.len() as isize;
    if n <= 1 {
        return;
    }
    let next = (state.active.get() as isize + delta).rem_euclid(n) as usize;
    let name = tabs[next].name.clone();
    drop(tabs);
    select_tab_by_name(state, &name);
}

fn focus_pane_in_direction(state: &AppState, dir: PaneDir) {
    let tabs = state.tabs.borrow();
    let Some(tab) = tabs.get(state.active.get()) else {
        return;
    };
    if tab.panes.len() <= 1 {
        return;
    }
    let focused = tab.focused.get();
    let Some(current) = tab.panes.iter().find(|p| p.id == focused) else {
        return;
    };
    let root = tab.root.clone();
    let Some(origin) = current.terminal.compute_bounds(&root) else {
        return;
    };
    let ox = origin.x() + origin.width() / 2.0;
    let oy = origin.y() + origin.height() / 2.0;

    let mut best: Option<(f32, Terminal)> = None;
    for pane in &tab.panes {
        if pane.id == focused {
            continue;
        }
        let Some(bounds) = pane.terminal.compute_bounds(&root) else {
            continue;
        };
        let px = bounds.x() + bounds.width() / 2.0;
        let py = bounds.y() + bounds.height() / 2.0;
        let dx = px - ox;
        let dy = py - oy;
        let on_side = match dir {
            PaneDir::Right => dx > 0.0,
            PaneDir::Left => dx < 0.0,
            PaneDir::Down => dy > 0.0,
            PaneDir::Up => dy < 0.0,
        };
        if !on_side {
            continue;
        }
        let (primary, ortho) = match dir {
            PaneDir::Left | PaneDir::Right => (dx.abs(), dy.abs()),
            PaneDir::Up | PaneDir::Down => (dy.abs(), dx.abs()),
        };
        let score = primary + 2.0 * ortho;
        if best.as_ref().map_or(true, |(s, _)| score < *s) {
            best = Some((score, pane.terminal.clone()));
        }
    }
    drop(tabs);
    if let Some((_, terminal)) = best {
        terminal.grab_focus();
    }
}

fn split_active(state: &Rc<AppState>, orientation: Orientation) {
    let (tab_name, pane_id, terminal) = {
        let tabs = state.tabs.borrow();
        let Some(tab) = tabs.get(state.active.get()) else {
            return;
        };
        let focused = tab.focused.get();
        let Some(pane) = tab.panes.iter().find(|p| p.id == focused) else {
            return;
        };
        (tab.name.clone(), pane.id, pane.terminal.clone())
    };

    let new_pane_id = next_id(state);
    let new_terminal = terminal_tab::create_terminal(&state.settings.borrow());

    let paned = Paned::new(orientation);
    paned.add_css_class("terminal-paned");
    paned.set_hexpand(true);
    paned.set_vexpand(true);
    paned.set_wide_handle(true);
    paned.set_resize_start_child(true);
    paned.set_resize_end_child(true);
    paned.set_shrink_start_child(false);
    paned.set_shrink_end_child(false);

    let Some(parent) = terminal.parent() else {
        return;
    };

    // GtkPaned refuses children that still have a parent — unparent first.
    if parent == state.stack {
        state.stack.remove(&terminal);
        paned.set_start_child(Some(&terminal));
        paned.set_end_child(Some(&new_terminal));
        state.stack.add_named(&paned, Some(&tab_name));
        state.stack.set_visible_child_name(&tab_name);
        if let Some(tab) = state
            .tabs
            .borrow_mut()
            .iter_mut()
            .find(|t| t.name == tab_name)
        {
            tab.root = paned.clone().upcast();
        }
    } else if let Ok(parent_paned) = parent.downcast::<Paned>() {
        let was_start = parent_paned
            .start_child()
            .is_some_and(|c| c == terminal.clone().upcast::<Widget>());
        if was_start {
            parent_paned.set_start_child(Widget::NONE);
        } else {
            parent_paned.set_end_child(Widget::NONE);
        }
        paned.set_start_child(Some(&terminal));
        paned.set_end_child(Some(&new_terminal));
        if was_start {
            parent_paned.set_start_child(Some(&paned));
        } else {
            parent_paned.set_end_child(Some(&paned));
        }
    } else {
        return;
    }

    balance_paned(&paned, orientation);

    wire_pane(state, &tab_name, new_pane_id, &new_terminal);

    if let Some(tab) = state
        .tabs
        .borrow_mut()
        .iter_mut()
        .find(|t| t.name == tab_name)
    {
        tab.panes.push(Pane {
            id: new_pane_id,
            terminal: new_terminal.clone(),
        });
        tab.focused.set(new_pane_id);
    }

    let _ = pane_id;
    new_terminal.grab_focus();
}

fn balance_paned(paned: &Paned, orientation: Orientation) {
    // Nested splits are often already mapped by the time we run, so connect_map
    // can miss. Retry briefly until the paned has a real size.
    let paned = paned.clone();
    let attempts = Rc::new(Cell::new(0u32));
    glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
        let mid = match orientation {
            Orientation::Horizontal => paned.width() / 2,
            Orientation::Vertical => paned.height() / 2,
            _ => return glib::ControlFlow::Break,
        };
        if mid > 0 {
            paned.set_position(mid);
            return glib::ControlFlow::Break;
        }
        let n = attempts.get();
        if n >= 60 {
            return glib::ControlFlow::Break;
        }
        attempts.set(n + 1);
        glib::ControlFlow::Continue
    });
}

fn close_active_pane(state: &Rc<AppState>) {
    let (tab_name, pane_id) = {
        let tabs = state.tabs.borrow();
        let Some(tab) = tabs.get(state.active.get()) else {
            return;
        };
        (tab.name.clone(), tab.focused.get())
    };
    close_pane_by_id(state, &tab_name, pane_id);
}

fn close_pane_by_id(state: &Rc<AppState>, tab_name: &str, pane_id: u32) {
    let pane_count = {
        let tabs = state.tabs.borrow();
        let Some(tab) = tabs.iter().find(|t| t.name == tab_name) else {
            return;
        };
        tab.panes.len()
    };

    if pane_count <= 1 {
        close_tab_by_name(state, tab_name);
        return;
    }

    let terminal = {
        let tabs = state.tabs.borrow();
        let Some(tab) = tabs.iter().find(|t| t.name == tab_name) else {
            return;
        };
        let Some(pane) = tab.panes.iter().find(|p| p.id == pane_id) else {
            return;
        };
        pane.terminal.clone()
    };

    let Some(parent) = terminal.parent() else {
        return;
    };
    let Ok(parent_paned) = parent.downcast::<Paned>() else {
        return;
    };

    let start = parent_paned.start_child();
    let end = parent_paned.end_child();
    let terminal_widget = terminal.clone().upcast::<Widget>();
    let sibling = if start.as_ref() == Some(&terminal_widget) {
        end
    } else if end.as_ref() == Some(&terminal_widget) {
        start
    } else {
        return;
    };
    let Some(sibling) = sibling else {
        return;
    };

    let Some(grandparent) = parent_paned.parent() else {
        return;
    };

    // Detach both children before reparenting the survivor.
    parent_paned.set_start_child(gtk4::Widget::NONE);
    parent_paned.set_end_child(gtk4::Widget::NONE);

    if grandparent == state.stack {
        state.stack.remove(&parent_paned);
        state.stack.add_named(&sibling, Some(tab_name));
        state.stack.set_visible_child_name(tab_name);
        if let Some(tab) = state
            .tabs
            .borrow_mut()
            .iter_mut()
            .find(|t| t.name == tab_name)
        {
            tab.root = sibling;
        }
    } else if let Ok(gp) = grandparent.downcast::<Paned>() {
        let was_start = gp
            .start_child()
            .is_some_and(|c| c == parent_paned.clone().upcast::<Widget>());
        if was_start {
            gp.set_start_child(Some(&sibling));
        } else {
            gp.set_end_child(Some(&sibling));
        }
    }

    let next_focus = {
        let mut tabs = state.tabs.borrow_mut();
        let Some(tab) = tabs.iter_mut().find(|t| t.name == tab_name) else {
            return;
        };
        tab.panes.retain(|p| p.id != pane_id);
        if tab.focused.get() == pane_id {
            if let Some(first) = tab.panes.first() {
                tab.focused.set(first.id);
                Some(first.terminal.clone())
            } else {
                None
            }
        } else {
            tab.panes
                .iter()
                .find(|p| p.id == tab.focused.get())
                .map(|p| p.terminal.clone())
        }
    };

    if let Some(term) = next_focus {
        let title = terminal_tab::title_from_terminal(&term);
        if let Some(tab) = state.tabs.borrow().iter().find(|t| t.name == tab_name) {
            tab.label.set_text(&title);
        }
        term.grab_focus();
    }
}

fn close_tab_by_name(state: &Rc<AppState>, name: &str) {
    let index = {
        let tabs = state.tabs.borrow();
        tabs.iter().position(|t| t.name == name)
    };
    let Some(index) = index else {
        return;
    };

    {
        let tabs = state.tabs.borrow();
        if tabs.len() == 1 {
            drop(tabs);
            // Keep at least one tab: collapse splits and respawn shell.
            reset_last_tab(state);
            return;
        }
    }

    let tab = state.tabs.borrow_mut().remove(index);
    state.tab_strip.remove(&tab.pill);
    state.stack.remove(&tab.root);

    let new_index = if index >= state.tabs.borrow().len() {
        state.tabs.borrow().len().saturating_sub(1)
    } else {
        index
    };
    let next_name = state.tabs.borrow().get(new_index).map(|t| t.name.clone());
    if let Some(name) = next_name {
        select_tab_by_name(state, &name);
    }
    update_tabs_visibility(state);
}

fn reset_last_tab(state: &Rc<AppState>) {
    let (name, old_root, keep_terminal, keep_id) = {
        let tabs = state.tabs.borrow();
        let Some(tab) = tabs.first() else {
            return;
        };
        let focused = tab.focused.get();
        let pane = tab
            .panes
            .iter()
            .find(|p| p.id == focused)
            .or_else(|| tab.panes.first());
        let Some(pane) = pane else {
            return;
        };
        (
            tab.name.clone(),
            tab.root.clone(),
            pane.terminal.clone(),
            pane.id,
        )
    };

    // If already a single pane, just respawn.
    let pane_count = state.tabs.borrow().first().map(|t| t.panes.len()).unwrap_or(0);
    if pane_count <= 1 {
        terminal_tab::spawn_shell(&keep_terminal, &SpawnOpts::default());
        keep_terminal.grab_focus();
        return;
    }

    // Collapse to the focused terminal as the sole root.
    if let Some(parent) = keep_terminal.parent() {
        if let Ok(paned) = parent.downcast::<Paned>() {
            paned.set_start_child(gtk4::Widget::NONE);
            paned.set_end_child(gtk4::Widget::NONE);
        }
    }
    state.stack.remove(&old_root);
    state.stack.add_named(&keep_terminal, Some(&name));
    state.stack.set_visible_child_name(&name);

    if let Some(tab) = state.tabs.borrow_mut().first_mut() {
        tab.root = keep_terminal.clone().upcast();
        tab.panes.retain(|p| p.id == keep_id);
        tab.focused.set(keep_id);
    }

    terminal_tab::spawn_shell(&keep_terminal, &SpawnOpts::default());
    keep_terminal.grab_focus();
}

fn close_active_tab(state: &Rc<AppState>) {
    let name = state
        .tabs
        .borrow()
        .get(state.active.get())
        .map(|t| t.name.clone());
    if let Some(name) = name {
        close_tab_by_name(state, &name);
    }
}
