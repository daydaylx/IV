//! Profilbezogene Aktionen: Anlegen, Auswählen, Anwenden.
//!
//! Tastenkürzel `Alt+1`…`Alt+9` wählen die ersten neun Profile
//! alphabetisch aus. `Alt+0` ist absichtlich unbelegt. Die
//! Profilanlage läuft über einen [`adw::AlertDialog`].

#![allow(dead_code, reason = "Profil-UI ist die Grundlage für T12-Iterationen")]

use std::rc::Rc;

use adw::prelude::*;
use gtk::{gio, glib};

use crate::ui::window::{UiState, show_status};
use crate::workspace::{ProfileId, StartConfig, StartProfile};

/// Installiert die Profil-Aktionen und die zugehörigen Tastenkürzel.
pub(super) fn install_profile_actions(application: &adw::Application, state: &Rc<UiState>) {
    // win.open-new-profile-dialog
    let state_for_new = Rc::clone(state);
    let new_action = gio::SimpleAction::new("open-new-profile-dialog", None);
    new_action.connect_activate(move |_, _| {
        open_new_profile_dialog(&state_for_new);
    });
    state.window.add_action(&new_action);
    application.set_accels_for_action("win.open-new-profile-dialog", &["<Control><Shift>n"]);

    // win.clear-active-profile
    let state_for_clear = Rc::clone(state);
    let clear_action = gio::SimpleAction::new("clear-active-profile", None);
    clear_action.connect_activate(move |_, _| {
        *state_for_clear.active_profile_id.borrow_mut() = None;
        show_status(&state_for_clear, "Kein Profil aktiv.");
    });
    state.window.add_action(&clear_action);

    // win.apply-profile-N (N = 1..=9)
    install_quick_pick_actions(application, state);
}

fn install_quick_pick_actions(application: &adw::Application, state: &Rc<UiState>) {
    for n in 1..=9u32 {
        let state_for_action = Rc::clone(state);
        let name = format!("apply-profile-{n}");
        let action = gio::SimpleAction::new(&name, None);
        action.connect_activate(move |_, _| {
            apply_nth_profile(&state_for_action, n as usize);
        });
        state.window.add_action(&action);
        // Alt+1..Alt+9
        let accel = format!("<Alt>{n}");
        application.set_accels_for_action(&format!("win.{name}"), &[accel.as_str()]);
    }
}

fn sorted_profile_ids(state: &UiState) -> Vec<ProfileId> {
    let binding = state.profiles.borrow();
    let mut profiles: Vec<&StartProfile> = binding.iter().collect();
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    profiles.into_iter().map(|p| p.id).collect()
}

fn apply_nth_profile(state: &Rc<UiState>, index_1_based: usize) {
    let ids = sorted_profile_ids(state);
    let Some(&id) = ids.get(index_1_based.saturating_sub(1)) else {
        show_status(state, "Kein Profil für diesen Schnellzugriff vorhanden.");
        return;
    };
    apply_profile(state, id);
}

fn apply_profile(state: &Rc<UiState>, id: ProfileId) {
    let profile = {
        let profiles = state.profiles.borrow();
        profiles.iter().find(|p| p.id == id).cloned()
    };
    let Some(profile) = profile else {
        show_status(state, "Das ausgewählte Profil wurde nicht gefunden.");
        return;
    };
    *state.active_profile_id.borrow_mut() = Some(id);
    show_status(
        state,
        &format!(
            "Profil „{}“ aktiv. Wende das Profil auf neue Panes an.",
            profile.name
        ),
    );
}

fn open_new_profile_dialog(state: &Rc<UiState>) {
    use gtk::ResponseType;

    // Use a modal gtk::Window since gtk::Dialog is deprecated since 4.10.
    let window = gtk::Window::builder()
        .title("Neues Profil")
        .modal(true)
        .transient_for(&state.window)
        .default_width(420)
        .default_height(320)
        .resizable(false)
        .build();

    let outer = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let name_entry = gtk::Entry::builder()
        .placeholder_text("Name (Pflicht)")
        .build();
    let dir_entry = gtk::Entry::builder()
        .placeholder_text("Startverzeichnis, absolut (Pflicht)")
        .build();
    let shell_entry = gtk::Entry::builder()
        .placeholder_text("Optionale Shell, z. B. /bin/zsh")
        .build();
    let command_entry = gtk::Entry::builder()
        .placeholder_text("Optionaler Startbefehl, z. B. /usr/bin/nvim")
        .build();
    outer.append(&name_entry);
    outer.append(&dir_entry);
    outer.append(&shell_entry);
    outer.append(&command_entry);

    let button_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .halign(gtk::Align::End)
        .build();
    let cancel_button = gtk::Button::with_label("Abbrechen");
    let save_button = gtk::Button::with_label("Speichern");
    save_button.add_css_class("suggested-action");
    button_box.append(&cancel_button);
    button_box.append(&save_button);
    outer.append(&button_box);

    window.set_child(Some(&outer));

    let state_for_cancel = Rc::clone(state);
    let window_for_cancel = window.clone();
    cancel_button.connect_clicked(move |_| {
        let _ = state_for_cancel;
        window_for_cancel.close();
    });

    let state_for_save = Rc::clone(state);
    let window_for_save = window.clone();
    let name_for_save = name_entry.clone();
    let dir_for_save = dir_entry.clone();
    let shell_for_save = shell_entry.clone();
    let command_for_save = command_entry.clone();
    save_button.connect_clicked(move |_| {
        handle_save(
            &state_for_save,
            &name_for_save,
            &dir_for_save,
            &shell_for_save,
            &command_for_save,
        );
        window_for_save.close();
    });

    // Suppress an unused-variable warning while keeping the API symbol.
    let _ = (ResponseType::Accept, ResponseType::Cancel);

    window.present();
}

fn handle_save(
    state: &Rc<UiState>,
    name_entry: &gtk::Entry,
    dir_entry: &gtk::Entry,
    shell_entry: &gtk::Entry,
    command_entry: &gtk::Entry,
) {
    let name = name_entry.text().trim().to_owned();
    let dir_text = dir_entry.text().trim().to_owned();
    if name.is_empty() {
        show_status(state, "Profil-Name darf nicht leer sein.");
        return;
    }
    if dir_text.is_empty() {
        show_status(state, "Startverzeichnis darf nicht leer sein.");
        return;
    }
    let shell = if shell_entry.text().trim().is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(shell_entry.text().trim()))
    };
    let command = if command_entry.text().trim().is_empty() {
        None
    } else {
        Some(vec![command_entry.text().trim().to_owned()])
    };
    let dir_path = std::path::PathBuf::from(&dir_text);
    let start_config = match StartConfig::new(&dir_path, shell, command) {
        Ok(config) => config,
        Err(error) => {
            show_status(state, &error.user_message());
            return;
        }
    };
    persist_new_profile(state, name, start_config);
}

fn persist_new_profile(state: &Rc<UiState>, name: String, start_config: StartConfig) {
    let Some(storage) = state.workspace_storage.borrow().clone() else {
        show_status(state, "Workspace-Storage ist nicht initialisiert.");
        return;
    };
    let mut profiles = state.profiles.borrow().clone();
    let new_id = profiles.len() as u64;
    let profile =
        match StartProfile::new(crate::workspace::ProfileId::new(new_id), name, start_config) {
            Ok(profile) => profile,
            Err(error) => {
                show_status(state, &error.user_message());
                return;
            }
        };
    profiles.push(profile.clone());
    *state.profiles.borrow_mut() = profiles.clone();

    let state_for_async = Rc::clone(state);
    glib::MainContext::default().spawn_local(async move {
        if let Err(error) = storage.save_profiles(&profiles).await {
            show_status(&state_for_async, &error.user_message());
            return;
        }
        show_status(
            &state_for_async,
            &format!("Profil „{}“ wurde gespeichert.", profile.name),
        );
    });
}
