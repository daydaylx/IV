//! Window actions and keyboard shortcuts.

use std::rc::Rc;

use adw::prelude::*;
use gtk::gio;

use crate::pane::{Direction, Orientation};

use super::window::{
    UiState, active_terminal, close_active_pane, close_active_tab, create_new_tab, move_pane_focus,
    next_tab, previous_tab, show_status, split_pane,
};

pub(super) fn install_tab_actions(application: &adw::Application, state: &Rc<UiState>) {
    let state_new = Rc::clone(state);
    let new_tab_action = gio::SimpleAction::new("new-tab", None);
    new_tab_action.connect_activate(move |_, _| create_new_tab(&state_new));
    state.window.add_action(&new_tab_action);

    let state_close = Rc::clone(state);
    let close_tab_action = gio::SimpleAction::new("close-tab", None);
    close_tab_action.connect_activate(move |_, _| close_active_tab(&state_close));
    state.window.add_action(&close_tab_action);

    let state_next = Rc::clone(state);
    let next_tab_action = gio::SimpleAction::new("next-tab", None);
    next_tab_action.connect_activate(move |_, _| next_tab(&state_next));
    state.window.add_action(&next_tab_action);

    let state_prev = Rc::clone(state);
    let prev_tab_action = gio::SimpleAction::new("prev-tab", None);
    prev_tab_action.connect_activate(move |_, _| previous_tab(&state_prev));
    state.window.add_action(&prev_tab_action);

    application.set_accels_for_action("win.new-tab", &["<Control><Shift>t"]);
    application.set_accels_for_action("win.close-tab", &["<Control><Shift>w"]);
    application.set_accels_for_action("win.next-tab", &["<Control>Page_Down"]);
    application.set_accels_for_action("win.prev-tab", &["<Control>Page_Up"]);
}

pub(super) fn install_pane_actions(application: &adw::Application, state: &Rc<UiState>) {
    let state_h = Rc::clone(state);
    let split_h = gio::SimpleAction::new("split-horizontal", None);
    split_h.connect_activate(move |_, _| split_pane(&state_h, Orientation::Horizontal));
    state.window.add_action(&split_h);

    let state_v = Rc::clone(state);
    let split_v = gio::SimpleAction::new("split-vertical", None);
    split_v.connect_activate(move |_, _| split_pane(&state_v, Orientation::Vertical));
    state.window.add_action(&split_v);

    let state_c = Rc::clone(state);
    let close_pane = gio::SimpleAction::new("close-pane", None);
    close_pane.connect_activate(move |_, _| close_active_pane(&state_c));
    state.window.add_action(&close_pane);

    for (name, direction) in [
        ("focus-left", Direction::Left),
        ("focus-right", Direction::Right),
        ("focus-up", Direction::Up),
        ("focus-down", Direction::Down),
    ] {
        let state_f = Rc::clone(state);
        let action = gio::SimpleAction::new(name, None);
        action.connect_activate(move |_, _| move_pane_focus(&state_f, direction));
        state.window.add_action(&action);
    }

    application.set_accels_for_action("win.split-horizontal", &["<Control><Shift>Right"]);
    application.set_accels_for_action("win.split-vertical", &["<Control><Shift>Down"]);
    application.set_accels_for_action("win.close-pane", &["<Control><Shift>q"]);
    application.set_accels_for_action("win.focus-left", &["<Alt>Left"]);
    application.set_accels_for_action("win.focus-right", &["<Alt>Right"]);
    application.set_accels_for_action("win.focus-up", &["<Alt>Up"]);
    application.set_accels_for_action("win.focus-down", &["<Alt>Down"]);
}

pub(super) fn install_clipboard_actions(application: &adw::Application, state: &Rc<UiState>) {
    let state_copy = Rc::clone(state);
    let copy_action = gio::SimpleAction::new("copy", None);
    copy_action.connect_activate(move |_, _| {
        if let Some(terminal) = active_terminal(&state_copy)
            && let Err(error) = terminal.copy()
        {
            show_status(&state_copy, error.user_message());
        }
    });
    state.window.add_action(&copy_action);

    let state_paste = Rc::clone(state);
    let paste_action = gio::SimpleAction::new("paste", None);
    paste_action.connect_activate(move |_, _| {
        if let Some(terminal) = active_terminal(&state_paste)
            && let Err(error) = terminal.paste()
        {
            show_status(&state_paste, error.user_message());
        }
    });
    state.window.add_action(&paste_action);

    application.set_accels_for_action("win.copy", &["<Control><Shift>c"]);
    application.set_accels_for_action("win.paste", &["<Control><Shift>v"]);
}

pub(super) fn install_font_actions(application: &adw::Application, state: &Rc<UiState>) {
    let state_zoom_in = Rc::clone(state);
    let zoom_in = gio::SimpleAction::new("zoom-in", None);
    zoom_in.connect_activate(move |_, _| {
        if let Some(terminal) = active_terminal(&state_zoom_in) {
            terminal.zoom_font(0.1);
        }
    });
    state.window.add_action(&zoom_in);

    let state_zoom_out = Rc::clone(state);
    let zoom_out = gio::SimpleAction::new("zoom-out", None);
    zoom_out.connect_activate(move |_, _| {
        if let Some(terminal) = active_terminal(&state_zoom_out) {
            terminal.zoom_font(-0.1);
        }
    });
    state.window.add_action(&zoom_out);

    let state_zoom_reset = Rc::clone(state);
    let zoom_reset = gio::SimpleAction::new("zoom-reset", None);
    zoom_reset.connect_activate(move |_, _| {
        if let Some(terminal) = active_terminal(&state_zoom_reset) {
            terminal.reset_font_zoom();
        }
    });
    state.window.add_action(&zoom_reset);

    application.set_accels_for_action("win.zoom-in", &["<Control>plus", "<Control>equal"]);
    application.set_accels_for_action("win.zoom-out", &["<Control>minus"]);
    application.set_accels_for_action("win.zoom-reset", &["<Control>0"]);
}
