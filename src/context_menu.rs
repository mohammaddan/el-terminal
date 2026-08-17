use gtk4::prelude::*;
use gtk4::{gio, PopoverMenu};

/// Right-click context menu for a terminal pane.
pub fn attach_context_menu(
    terminal: &impl IsA<gtk4::Widget>,
    on_open: impl Fn() + 'static,
) {
    let menu = gio::Menu::new();
    menu.append(Some("Copy"), Some("win.copy"));
    menu.append(Some("Paste"), Some("win.paste"));
    menu.append(Some("Select All"), Some("win.select-all"));

    let tab_section = gio::Menu::new();
    tab_section.append(Some("New Tab"), Some("win.new-tab"));
    tab_section.append(Some("Previous Tab"), Some("win.prev-tab"));
    tab_section.append(Some("Next Tab"), Some("win.next-tab"));
    tab_section.append(Some("Close Tab"), Some("win.close-tab"));
    menu.append_section(None, &tab_section);

    let split_section = gio::Menu::new();
    split_section.append(Some("Split Right"), Some("win.split-right"));
    split_section.append(Some("Split Down"), Some("win.split-down"));
    split_section.append(Some("Close Pane"), Some("win.close-pane"));
    menu.append_section(None, &split_section);

    let pane_section = gio::Menu::new();
    pane_section.append(Some("Focus Left"), Some("win.focus-pane-left"));
    pane_section.append(Some("Focus Right"), Some("win.focus-pane-right"));
    pane_section.append(Some("Focus Up"), Some("win.focus-pane-up"));
    pane_section.append(Some("Focus Down"), Some("win.focus-pane-down"));
    menu.append_section(None, &pane_section);

    let popover = PopoverMenu::from_model(Some(&menu));
    popover.set_parent(terminal);
    popover.set_has_arrow(false);
    popover.add_css_class("menu");

    let gesture = gtk4::GestureClick::new();
    gesture.set_button(3);
    gesture.set_propagation_phase(gtk4::PropagationPhase::Capture);

    let popover_c = popover.clone();
    gesture.connect_pressed(move |gesture, _n, x, y| {
        gesture.set_state(gtk4::EventSequenceState::Claimed);
        on_open();
        popover_c.set_pointing_to(Some(&gtk4::gdk::Rectangle::new(
            x as i32,
            y as i32,
            1,
            1,
        )));
        popover_c.popup();
    });

    terminal.add_controller(gesture);
    // Gesture closure holds the popover; keep an extra strong ref on the widget tree.
    std::mem::forget(popover);
}

pub fn install_window_actions(
    window: &gtk4::ApplicationWindow,
    on_copy: impl Fn() + 'static,
    on_paste: impl Fn() + 'static,
    on_select_all: impl Fn() + 'static,
    on_new_tab: impl Fn() + 'static,
    on_close_tab: impl Fn() + 'static,
    on_split_right: impl Fn() + 'static,
    on_split_down: impl Fn() + 'static,
    on_close_pane: impl Fn() + 'static,
) {
    let copy = gio::SimpleAction::new("copy", None);
    copy.connect_activate(move |_, _| on_copy());
    window.add_action(&copy);

    let paste = gio::SimpleAction::new("paste", None);
    paste.connect_activate(move |_, _| on_paste());
    window.add_action(&paste);

    let select_all = gio::SimpleAction::new("select-all", None);
    select_all.connect_activate(move |_, _| on_select_all());
    window.add_action(&select_all);

    let new_tab = gio::SimpleAction::new("new-tab", None);
    new_tab.connect_activate(move |_, _| on_new_tab());
    window.add_action(&new_tab);

    let close_tab = gio::SimpleAction::new("close-tab", None);
    close_tab.connect_activate(move |_, _| on_close_tab());
    window.add_action(&close_tab);

    let split_right = gio::SimpleAction::new("split-right", None);
    split_right.connect_activate(move |_, _| on_split_right());
    window.add_action(&split_right);

    let split_down = gio::SimpleAction::new("split-down", None);
    split_down.connect_activate(move |_, _| on_split_down());
    window.add_action(&split_down);

    let close_pane = gio::SimpleAction::new("close-pane", None);
    close_pane.connect_activate(move |_, _| on_close_pane());
    window.add_action(&close_pane);
}

pub fn install_navigation_actions(
    window: &gtk4::ApplicationWindow,
    on_prev_tab: impl Fn() + 'static,
    on_next_tab: impl Fn() + 'static,
    on_focus_left: impl Fn() + 'static,
    on_focus_right: impl Fn() + 'static,
    on_focus_up: impl Fn() + 'static,
    on_focus_down: impl Fn() + 'static,
) {
    let prev_tab = gio::SimpleAction::new("prev-tab", None);
    prev_tab.connect_activate(move |_, _| on_prev_tab());
    window.add_action(&prev_tab);

    let next_tab = gio::SimpleAction::new("next-tab", None);
    next_tab.connect_activate(move |_, _| on_next_tab());
    window.add_action(&next_tab);

    let focus_left = gio::SimpleAction::new("focus-pane-left", None);
    focus_left.connect_activate(move |_, _| on_focus_left());
    window.add_action(&focus_left);

    let focus_right = gio::SimpleAction::new("focus-pane-right", None);
    focus_right.connect_activate(move |_, _| on_focus_right());
    window.add_action(&focus_right);

    let focus_up = gio::SimpleAction::new("focus-pane-up", None);
    focus_up.connect_activate(move |_, _| on_focus_up());
    window.add_action(&focus_up);

    let focus_down = gio::SimpleAction::new("focus-pane-down", None);
    focus_down.connect_activate(move |_, _| on_focus_down());
    window.add_action(&focus_down);
}
