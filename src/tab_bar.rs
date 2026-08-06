use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation};

/// Builds a single slim pill-shaped tab widget.
pub fn build_tab_pill(title: &str, accent: &str, active: bool) -> (GtkBox, Label, Button) {
    let pill = GtkBox::new(Orientation::Horizontal, 5);
    pill.add_css_class("tab-pill");
    if active {
        pill.add_css_class("active");
    }
    pill.set_cursor_from_name(Some("pointer"));

    let dot = GtkBox::new(Orientation::Horizontal, 0);
    dot.add_css_class("tab-dot");
    dot.add_css_class(accent);
    dot.set_valign(gtk4::Align::Center);
    dot.set_halign(gtk4::Align::Center);

    let label = Label::new(Some(title));
    label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    label.set_max_width_chars(20);
    label.set_xalign(0.0);

    let close = Button::from_icon_name("window-close-symbolic");
    close.add_css_class("tab-close");
    close.set_focus_on_click(false);
    close.set_tooltip_text(Some("Close tab"));

    pill.append(&dot);
    pill.append(&label);
    pill.append(&close);

    (pill, label, close)
}

pub fn build_new_tab_button() -> Button {
    let btn = Button::from_icon_name("list-add-symbolic");
    btn.add_css_class("new-tab-btn");
    btn.set_tooltip_text(Some("New tab"));
    btn.set_focus_on_click(false);
    btn
}

pub fn build_menu_button() -> Button {
    let btn = Button::from_icon_name("open-menu-symbolic");
    btn.add_css_class("menu-btn");
    btn.set_tooltip_text(Some("Menu"));
    btn.set_focus_on_click(false);
    btn
}

pub fn build_status_dot() -> GtkBox {
    let dot = GtkBox::new(Orientation::Horizontal, 0);
    dot.add_css_class("status-dot");
    dot.set_valign(gtk4::Align::Center);
    dot.set_halign(gtk4::Align::Center);
    dot.set_tooltip_text(Some("Connected"));
    dot
}

/// Minimize / maximize / close window controls (right side of title bar).
pub fn build_window_controls() -> (GtkBox, Button, Button, Button) {
    let row = GtkBox::new(Orientation::Horizontal, 2);
    row.add_css_class("window-controls");
    row.set_halign(gtk4::Align::End);
    row.set_valign(gtk4::Align::Center);

    let minimize = Button::from_icon_name("window-minimize-symbolic");
    minimize.add_css_class("window-control");
    minimize.add_css_class("window-minimize");
    minimize.set_tooltip_text(Some("Minimize"));
    minimize.set_focus_on_click(false);

    let maximize = Button::from_icon_name("window-maximize-symbolic");
    maximize.add_css_class("window-control");
    maximize.add_css_class("window-maximize");
    maximize.set_tooltip_text(Some("Maximize"));
    maximize.set_focus_on_click(false);

    let close = Button::from_icon_name("window-close-symbolic");
    close.add_css_class("window-control");
    close.add_css_class("window-close-btn");
    close.set_tooltip_text(Some("Close"));
    close.set_focus_on_click(false);

    row.append(&minimize);
    row.append(&maximize);
    row.append(&close);

    (row, minimize, maximize, close)
}
