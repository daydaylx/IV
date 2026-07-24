use crate::settings::{AppSettings, Theme};
use crate::ui::window;
use crate::ui::window::UiState;
use gtk::glib;
use std::rc::Rc; // to call show_settings_warnings

pub(super) fn load_pane_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(".active-pane { border: 2px solid @accent_color; }");

    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

pub(super) fn make_font_desc(settings: &AppSettings) -> gtk::pango::FontDescription {
    let mut desc = gtk::pango::FontDescription::from_string(&settings.font.family);
    desc.set_size((settings.font.size * gtk::pango::SCALE as f64) as i32);
    desc
}

pub(super) fn apply_theme(settings: &AppSettings) {
    let manager = adw::StyleManager::default();
    let color_scheme = match settings.theme {
        Theme::System => adw::ColorScheme::Default,
        Theme::Light => adw::ColorScheme::ForceLight,
        Theme::Dark => adw::ColorScheme::ForceDark,
    };
    manager.set_color_scheme(color_scheme);
}

pub(super) fn load_settings(state: &Rc<UiState>) {
    let state = Rc::downgrade(state);
    glib::MainContext::default().spawn_local(async move {
        let outcome = AppSettings::load_async().await;
        let Some(state) = state.upgrade() else {
            return;
        };

        apply_theme(&outcome.settings);
        let font_desc = make_font_desc(&outcome.settings);
        *state.font_desc.borrow_mut() = font_desc.clone();
        for terminal in state
            .tab_entries
            .borrow()
            .values()
            .flat_map(|entry| entry.terminals.values())
        {
            terminal.set_font(&font_desc);
        }
        window::show_settings_warnings(&state, &outcome.warnings);
    });
}
