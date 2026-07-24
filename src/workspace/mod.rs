//! Workspace-Persistenz: Startprofile, Layout-Snapshots und Storage.
//!
//! Dieses Modul kapselt alles, was über den Terminalkern hinausgeht:
//! benannte Startprofile und die Wiederherstellung des letzten Layouts.
//! Es kennt absichtlich keine GTK- oder VTE-Typen; alle I/O-Vorgänge
//! laufen über `gio::File` und sind asynchron.
//!
//! Architekturentscheidung: siehe `docs/decisions/001-workspace-modul-und-persistenz.md`.

// Während der Phase-2-Iteration werden noch nicht alle Items aus dem
// `workspace/`-Modul aktiv genutzt; sie sind die Grundlage für die
// anstehenden Tabs (T12). Daher sind `dead_code` und `unused_imports`
// hier erlaubt.
#![allow(dead_code, reason = "Phase-2 foundation; UI-Integration in T12")]
#![allow(unused_imports, reason = "Phase-2 foundation; UI-Integration in T12")]

mod error;
mod layout;
mod profile;
mod start_config;
mod storage;

pub(crate) use error::{WorkspaceError, WorkspaceWarning};
pub(crate) use layout::{LayoutSnapshot, LayoutTab};
pub(crate) use profile::{ProfileId, StartProfile};
pub(crate) use start_config::{StartConfig, validate_path};
pub(crate) use storage::{LayoutDebouncer, WorkspaceStorage};

/// Aktuelle Schema-Version der TOML-Dateien.
///
/// Wird in den Dateien als Top-Level-Feld `schema_version` gespeichert.
/// Bei einer Inkompatibilität erzeugt der Lader eine
/// [`WorkspaceWarning::UnsupportedVersion`] und verwirft den Inhalt.
pub(crate) const SCHEMA_VERSION: u32 = 1;
