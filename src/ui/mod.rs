use std::rc::Rc;

use adw::prelude::*;
use gtk::{gio, glib};

use crate::terminal::{StartupWarning, Terminal, TerminalEvent};

const DEFAULT_WIDTH: i32 = 900;
const DEFAULT_HEIGHT: i32 = 600;
const MINIMUM_WIDTH: i32 = 480;
const MINIMUM_HEIGHT: i32 = 320;

pub(crate) fn build_main_window(application: &adw::Application) {
    let terminal = Terminal::new();
    let terminal_widget = terminal.widget();

    let status_label = gtk::Label::builder()
        .halign(gtk::Align::Start)
        .margin_start(12)
        .margin_end(12)
        .margin_top(6)
        .margin_bottom(6)
        .wrap(true)
        .build();
    let status_revealer = gtk::Revealer::builder()
        .transition_type(gtk::RevealerTransitionType::SlideDown)
        .child(&status_label)
        .build();

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&adw::HeaderBar::new());
    content.append(&status_revealer);
    content.append(&terminal_widget);

    let window = adw::ApplicationWindow::builder()
        .application(application)
        .title("IV")
        .default_width(DEFAULT_WIDTH)
        .default_height(DEFAULT_HEIGHT)
        .content(&content)
        .build();
    window.set_size_request(MINIMUM_WIDTH, MINIMUM_HEIGHT);

    install_clipboard_actions(
        application,
        &window,
        &terminal,
        &status_label,
        &status_revealer,
    );
    connect_terminal_events(&window, &terminal, &status_label, &status_revealer);
    connect_close_request(&window, &terminal, &status_label, &status_revealer);

    window.present();

    match terminal.start() {
        Ok(warnings) => show_warnings(&status_label, &status_revealer, &warnings),
        Err(error) => show_status(&status_label, &status_revealer, error.user_message()),
    }

    let terminal = terminal.clone();
    glib::idle_add_local_once(move || terminal.focus());
}

fn install_clipboard_actions(
    application: &adw::Application,
    window: &adw::ApplicationWindow,
    terminal: &Terminal,
    status_label: &gtk::Label,
    status_revealer: &gtk::Revealer,
) {
    let copy_action = gio::SimpleAction::new("copy", None);
    let terminal_copy = terminal.clone();
    let label_weak = status_label.downgrade();
    let revealer_weak = status_revealer.downgrade();
    copy_action.connect_activate(move |_, _| {
        if let Err(error) = terminal_copy.copy()
            && let (Some(label), Some(revealer)) = (label_weak.upgrade(), revealer_weak.upgrade())
        {
            show_status(&label, &revealer, error.user_message());
        }
    });
    window.add_action(&copy_action);

    let paste_action = gio::SimpleAction::new("paste", None);
    let terminal_paste = terminal.clone();
    let label_weak = status_label.downgrade();
    let revealer_weak = status_revealer.downgrade();
    paste_action.connect_activate(move |_, _| {
        if let Err(error) = terminal_paste.paste()
            && let (Some(label), Some(revealer)) = (label_weak.upgrade(), revealer_weak.upgrade())
        {
            show_status(&label, &revealer, error.user_message());
        }
    });
    window.add_action(&paste_action);

    application.set_accels_for_action("win.copy", &["<Control><Shift>c"]);
    application.set_accels_for_action("win.paste", &["<Control><Shift>v"]);
}

fn connect_terminal_events(
    window: &adw::ApplicationWindow,
    terminal: &Terminal,
    status_label: &gtk::Label,
    status_revealer: &gtk::Revealer,
) {
    let window_weak = window.downgrade();
    let label_weak = status_label.downgrade();
    let revealer_weak = status_revealer.downgrade();

    terminal.set_event_handler(Rc::new(move |event| match event {
        TerminalEvent::Started => {}
        TerminalEvent::SpawnFailed(error) => {
            if let (Some(label), Some(revealer)) = (label_weak.upgrade(), revealer_weak.upgrade()) {
                show_status(&label, &revealer, error.user_message());
            }
        }
        TerminalEvent::Exited(exit) if exit.successful() => {
            if let Some(window) = window_weak.upgrade() {
                window.close();
            }
        }
        TerminalEvent::Exited(exit) => {
            if let (Some(label), Some(revealer)) = (label_weak.upgrade(), revealer_weak.upgrade()) {
                show_status(&label, &revealer, &exit.user_message());
            }
        }
    }));
}

fn connect_close_request(
    window: &adw::ApplicationWindow,
    terminal: &Terminal,
    status_label: &gtk::Label,
    status_revealer: &gtk::Revealer,
) {
    let terminal = terminal.clone();
    let label_weak = status_label.downgrade();
    let revealer_weak = status_revealer.downgrade();

    window.connect_close_request(move |window| {
        let window_weak = window.downgrade();
        let close_immediately = terminal.request_close(move || {
            if let Some(window) = window_weak.upgrade() {
                window.close();
            }
        });

        if close_immediately {
            glib::Propagation::Proceed
        } else {
            if let (Some(label), Some(revealer)) = (label_weak.upgrade(), revealer_weak.upgrade()) {
                show_status(&label, &revealer, "Die Shell wird beendet …");
            }
            glib::Propagation::Stop
        }
    });
}

fn show_warnings(
    status_label: &gtk::Label,
    status_revealer: &gtk::Revealer,
    warnings: &[StartupWarning],
) {
    if warnings.is_empty() {
        return;
    }

    let message = warnings
        .iter()
        .map(|warning| warning.user_message())
        .collect::<Vec<_>>()
        .join(" ");
    show_status(status_label, status_revealer, &message);
}

fn show_status(status_label: &gtk::Label, status_revealer: &gtk::Revealer, message: &str) {
    status_label.set_label(message);
    status_revealer.set_reveal_child(true);
}
