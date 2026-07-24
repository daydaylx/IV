//! Asynchroner Lebenszyklus des Workspace-Layers.

//! Asynchroner Lebenszyklus des Workspace-Layers.
//!
//! Diese Datei kapselt:
//! - `bootstrap_workspace`: asynchrones Laden beim Programmstart
//! - `save_layout_debounced`: debounced Speichern nach Strukturänderungen
//! - `save_layout_now`: synchroner Flush beim Schließen des Fensters
//!
//! Persistenz und Wiederherstellung nutzen `WorkspaceStorage` aus dem
//! `workspace/`-Modul. Fehler werden in [`WorkspaceWarning`]s gewandelt
//! und über `show_status` an die UI gemeldet; die App fällt ansonsten
//! auf das Phase-1-Verhalten zurück.

#![allow(
    dead_code,
    reason = "Workspace-Layer ist angelegt, wird aber erst ab T12 von der UI aufgerufen"
)]
#![allow(clippy::collapsible_if, reason = "Lesbarkeit der Setup-Logik")]
#![allow(clippy::needless_borrow, reason = "wird in T12 konsolidiert")]

use std::path::PathBuf;
use std::rc::Rc;

use gtk::glib;

use crate::ui::window::{UiState, show_status, update_profile_state, update_tab_collection};
use crate::workspace::{
    LayoutDebouncer, LayoutSnapshot, StartProfile, WorkspaceStorage, WorkspaceWarning,
};

/// Liefert den Pfad zum Workspace-Ordner unterhalb des
/// `XDG_CONFIG_HOME`-Verzeichnisses.
pub(crate) fn workspace_dir() -> PathBuf {
    gtk::glib::user_config_dir().join("iv").join("workspace")
}

/// Stellt sicher, dass der Workspace-Ordner existiert.
///
/// Gibt den Storage-Wrapper zurück. Bei einem I/O-Fehler wird
/// `WorkspaceWarning::ReadFailed` zurückgegeben und der Storage zeigt
/// auf den (möglicherweise nicht angelegten) Pfad; das Schreiben wird
/// dann ebenfalls fehlschlagen und gemeldet.
pub(crate) fn ensure_workspace() -> (WorkspaceStorage, Vec<WorkspaceWarning>) {
    let mut warnings = Vec::new();
    let dir = workspace_dir();
    if !dir.exists() {
        if let Err(error) = std::fs::create_dir_all(&dir) {
            warnings.push(WorkspaceWarning::ReadFailed);
            show_workspace_error(&format!(
                "Der Workspace-Ordner konnte nicht angelegt werden: {error}."
            ));
        }
    }
    let storage = WorkspaceStorage::new(dir);
    (storage, warnings)
}

fn show_workspace_error(message: &str) {
    gtk::glib::g_warning!("iv.workspace", "{}", message);
}

/// Startet den asynchronen Bootstrap des Workspaces.
///
/// Reihenfolge:
/// 1. Workspace-Ordner anlegen
/// 2. Profile asynchron laden
/// 3. Layout asynchron laden
/// 4. Geladene Profile in `UiState` ablegen
/// 5. Tabs aus dem Snapshot wiederherstellen
/// 6. Warnungen in der Statuszeile anzeigen
pub(crate) fn bootstrap_workspace(state: std::rc::Rc<UiState>) {
    glib::MainContext::default().spawn_local(async move {
        let (storage, setup_warnings) = ensure_workspace();
        *state.workspace_storage.borrow_mut() = Some(storage.clone());
        for warning in &setup_warnings {
            show_status(&state, &warning.user_message());
        }

        let (profiles, profile_warnings) = storage.load_profiles().await;
        for warning in &profile_warnings {
            show_status(&state, &warning.user_message());
        }
        *state.profiles.borrow_mut() = profiles;

        let (snapshot, layout_warnings) = storage.load_layout().await;
        for warning in &layout_warnings {
            show_status(&state, &warning.user_message());
        }

        let profiles = state.profiles.borrow().clone();
        match LayoutSnapshot::into_collection(snapshot, &profile_id_set(&profiles)) {
            Ok((collection, active_profile_id, restore_warnings)) => {
                for warning in &restore_warnings {
                    show_status(&state, &warning.user_message());
                }
                update_profile_state(&state, active_profile_id);
                update_tab_collection(&state, collection);
            }
            Err(error) => {
                show_status(&state, &error.user_message());
            }
        }
    });
}

fn profile_id_set(
    profiles: &[StartProfile],
) -> std::collections::BTreeMap<crate::workspace::ProfileId, ()> {
    profiles.iter().map(|profile| (profile.id, ())).collect()
}

/// Markiert das aktuelle Layout als verändert.
///
/// Wird von UI-Aktionen aufgerufen (Tab hinzufügen, schließen, Pane
/// splitten, schließen, Tab-Titel ändern). Der eigentliche
/// Schreibvorgang wird in [`flush_pending_layout`] angestoßen.
pub(crate) fn mark_layout_dirty(state: &std::rc::Rc<UiState>) {
    state.layout_debouncer.borrow_mut().mark_dirty();
    flush_pending_layout(std::rc::Rc::clone(state));
}

/// Prüft, ob ein Schreibvorgang ansteht, und führt ihn aus.
pub(crate) fn flush_pending_layout(state: std::rc::Rc<UiState>) {
    let should_flush = state.layout_debouncer.borrow().should_flush();
    if !should_flush {
        return;
    }
    save_layout_now(&state);
}

/// Synchroner Flush des aktuellen Layouts.
///
/// Wird beim Schließen des Fensters aufgerufen. Setzt das
/// `pending`-Flag zurück, auch wenn das Speichern fehlschlägt.
pub(crate) fn save_layout_now(state: &Rc<UiState>) {
    let Some(storage) = state.workspace_storage.borrow().clone() else {
        return;
    };
    let snapshot = {
        let collection = state.tab_collection.borrow();
        let active_profile_id = *state.active_profile_id.borrow();
        LayoutSnapshot::from_collection(&collection, active_profile_id)
    };
    state.layout_debouncer.borrow_mut().mark_flushed();
    let state_for_async = Rc::clone(state);
    glib::MainContext::default().spawn_local(async move {
        if let Err(error) = storage.save_layout(&snapshot).await {
            gtk::glib::g_warning!(
                "iv.workspace",
                "Layout konnte nicht gespeichert werden: {}",
                error
            );
            show_status(&state_for_async, &error.user_message());
        }
    });
}

/// Initialisiert einen frischen [`LayoutDebouncer`].
pub(crate) fn new_debouncer() -> LayoutDebouncer {
    LayoutDebouncer::new()
}
