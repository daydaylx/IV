//! Safe Ctrl+Click link launching.

use std::rc::Weak;

use adw::prelude::*;
use gtk::{gio, glib};

use crate::terminal::Terminal;

use super::window::{UiState, show_status};

pub(super) fn attach_url_click_handler(
    widget: &gtk::Widget,
    terminal: Terminal,
    state: Weak<UiState>,
) {
    let click = gtk::GestureClick::new();
    click.set_button(1);

    click.connect_pressed(move |gesture, _n_press, x, y| {
        let modifiers = gesture.current_event_state();
        if !modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
            return;
        }

        if let Some(uri) = terminal.hyperlink_at(x, y)
            && open_uri(&uri).is_err()
            && let Some(state) = state.upgrade()
        {
            show_status(
                &state,
                "Der Link konnte nicht mit der Standardanwendung geöffnet werden.",
            );
        }
    });

    widget.add_controller(click);
}

fn open_uri(uri: &str) -> Result<(), glib::Error> {
    gio::AppInfo::launch_default_for_uri(uri, gio::AppLaunchContext::NONE)
}
