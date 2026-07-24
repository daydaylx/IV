//! Asynchroner Storage für `profiles.toml` und `layout.toml`.
//!
//! Schreibt atomar via `*.tmp` + `replace_contents_future_async` und
//! räumt verwaiste temporäre Dateien beim Start auf.

use std::cell::RefCell;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::gio;
use gtk::prelude::*;

use crate::workspace::SCHEMA_VERSION;
use crate::workspace::error::{WorkspaceError, WorkspaceWarning};
use crate::workspace::layout::LayoutSnapshot;
use crate::workspace::profile::StartProfile;

/// Asynchrone Persistenzschicht für Profile und Layout.
#[derive(Clone)]
pub(crate) struct WorkspaceStorage {
    root: PathBuf,
}

impl WorkspaceStorage {
    /// Erstellt einen Storage-Wrapper für das angegebene Verzeichnis.
    /// Das Verzeichnis wird **nicht** automatisch angelegt.
    pub(crate) fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// Liefert den Pfad zu `profiles.toml`.
    pub(crate) fn profiles_path(&self) -> PathBuf {
        self.root.join("profiles.toml")
    }

    /// Liefert den Pfad zu `layout.toml`.
    pub(crate) fn layout_path(&self) -> PathBuf {
        self.root.join("layout.toml")
    }

    /// Lädt `profiles.toml` asynchron. Liefert leere Liste plus
    /// Warnung, falls die Datei fehlt oder ungültig ist.
    pub(crate) async fn load_profiles(&self) -> (Vec<StartProfile>, Vec<WorkspaceWarning>) {
        match Self::load_file(&self.profiles_path()).await {
            Ok(Some(contents)) => Self::parse_profiles(&contents),
            Ok(None) => (Vec::new(), Vec::new()),
            Err(warning) => (Vec::new(), vec![warning]),
        }
    }

    /// Speichert die Profile atomar asynchron.
    pub(crate) async fn save_profiles(
        &self,
        profiles: &[StartProfile],
    ) -> Result<(), WorkspaceError> {
        let body = serialize_profiles(profiles)?;
        Self::write_atomic(&self.profiles_path(), &body, StoredFileKind::Profiles).await
    }

    /// Lädt `layout.toml` asynchron. Liefert leeren Snapshot plus
    /// Warnung, falls die Datei fehlt oder ungültig ist.
    pub(crate) async fn load_layout(&self) -> (LayoutSnapshot, Vec<WorkspaceWarning>) {
        match Self::load_file(&self.layout_path()).await {
            Ok(Some(contents)) => Self::parse_layout(&contents),
            Ok(None) => (LayoutSnapshot::empty(), Vec::new()),
            Err(warning) => (LayoutSnapshot::empty(), vec![warning]),
        }
    }

    /// Speichert das Layout atomar asynchron.
    pub(crate) async fn save_layout(
        &self,
        snapshot: &LayoutSnapshot,
    ) -> Result<(), WorkspaceError> {
        let body = serialize_layout(snapshot)?;
        Self::write_atomic(&self.layout_path(), &body, StoredFileKind::Layout).await
    }

    /// Lädt eine Datei asynchron über `gio::File`.
    async fn load_file(path: &Path) -> Result<Option<Vec<u8>>, WorkspaceWarning> {
        let file = gio::File::for_path(path);
        match file.load_contents_future().await {
            Ok((contents, _)) => Ok(Some(contents.to_vec())),
            Err(error) if error.kind::<gio::IOErrorEnum>() == Some(gio::IOErrorEnum::NotFound) => {
                Ok(None)
            }
            Err(_) => Err(WorkspaceWarning::ReadFailed),
        }
    }

    async fn write_atomic(
        path: &Path,
        body: &str,
        kind: StoredFileKind,
    ) -> Result<(), WorkspaceError> {
        Self::ensure_writable_file(path, kind).await?;
        let file = gio::File::for_path(path);
        file.replace_contents_future(
            body.as_bytes().to_vec(),
            None,
            true,
            gio::FileCreateFlags::REPLACE_DESTINATION,
        )
        .await
        .map_err(|(_, error)| WorkspaceError::Io(error.message().to_owned()))?;
        Ok(())
    }

    async fn ensure_writable_file(path: &Path, kind: StoredFileKind) -> Result<(), WorkspaceError> {
        let Some(contents) = Self::load_file(path)
            .await
            .map_err(|_| WorkspaceError::Io("read failed".to_owned()))?
        else {
            return Ok(());
        };
        let text = std::str::from_utf8(&contents).map_err(|_| WorkspaceError::InvalidUtf8)?;
        let document = text
            .parse::<toml::Value>()
            .map_err(|_| WorkspaceError::InvalidToml)?;
        let version = document
            .get("schema_version")
            .and_then(toml::Value::as_integer)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(WorkspaceError::InvalidToml)?;
        if version != SCHEMA_VERSION {
            return Err(WorkspaceError::UnsupportedVersion(version));
        }

        match kind {
            StoredFileKind::Profiles => {
                let file = document
                    .try_into::<ProfilesFile>()
                    .map_err(|_| WorkspaceError::InvalidToml)?;
                if !profiles_are_valid(&file.profile) {
                    return Err(WorkspaceError::InvalidToml);
                }
            }
            StoredFileKind::Layout => {
                let snapshot = document
                    .try_into::<LayoutSnapshot>()
                    .map_err(|_| WorkspaceError::InvalidToml)?;
                snapshot.validate()?;
            }
        }
        Ok(())
    }

    fn parse_profiles(contents: &[u8]) -> (Vec<StartProfile>, Vec<WorkspaceWarning>) {
        let text = match std::str::from_utf8(contents) {
            Ok(text) => text,
            Err(_) => return (Vec::new(), vec![WorkspaceWarning::InvalidToml]),
        };
        let value: toml::Value = match text.parse() {
            Ok(value) => value,
            Err(_) => return (Vec::new(), vec![WorkspaceWarning::InvalidToml]),
        };
        let version = value
            .get("schema_version")
            .and_then(|v| v.as_integer())
            .unwrap_or(0) as u32;
        if version != SCHEMA_VERSION {
            return (
                Vec::new(),
                vec![WorkspaceWarning::UnsupportedVersion(version)],
            );
        }
        match value.try_into::<ProfilesFile>() {
            Ok(file) if profiles_are_valid(&file.profile) => (file.profile, Vec::new()),
            Err(_) => (Vec::new(), vec![WorkspaceWarning::InvalidToml]),
            Ok(_) => (Vec::new(), vec![WorkspaceWarning::InvalidToml]),
        }
    }

    fn parse_layout(contents: &[u8]) -> (LayoutSnapshot, Vec<WorkspaceWarning>) {
        let text = match std::str::from_utf8(contents) {
            Ok(text) => text,
            Err(_) => return (LayoutSnapshot::empty(), vec![WorkspaceWarning::InvalidToml]),
        };
        let value: toml::Value = match text.parse() {
            Ok(value) => value,
            Err(_) => return (LayoutSnapshot::empty(), vec![WorkspaceWarning::InvalidToml]),
        };
        let version = value
            .get("schema_version")
            .and_then(|v| v.as_integer())
            .unwrap_or(0) as u32;
        if version != SCHEMA_VERSION {
            return (
                LayoutSnapshot::empty(),
                vec![WorkspaceWarning::UnsupportedVersion(version)],
            );
        }
        match value.try_into::<LayoutSnapshot>() {
            Ok(snapshot) if snapshot.validate().is_ok() => (snapshot, Vec::new()),
            Err(_) => (LayoutSnapshot::empty(), vec![WorkspaceWarning::InvalidToml]),
            Ok(_) => (LayoutSnapshot::empty(), vec![WorkspaceWarning::InvalidToml]),
        }
    }
}

#[derive(Clone, Copy)]
enum StoredFileKind {
    Profiles,
    Layout,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ProfilesFile {
    schema_version: u32,
    #[serde(default)]
    profile: Vec<StartProfile>,
}

fn serialize_profiles(profiles: &[StartProfile]) -> Result<String, WorkspaceError> {
    let file = ProfilesFile {
        schema_version: SCHEMA_VERSION,
        profile: profiles.to_vec(),
    };
    Ok(toml::to_string(&file)?)
}

fn serialize_layout(snapshot: &LayoutSnapshot) -> Result<String, WorkspaceError> {
    Ok(toml::to_string(snapshot)?)
}

fn profiles_are_valid(profiles: &[StartProfile]) -> bool {
    let mut ids = HashSet::new();
    profiles
        .iter()
        .all(|profile| ids.insert(profile.id) && profile.validate().is_ok())
}

/// Debounce-Helfer für Layout-Updates.
///
/// Führt maximal einen Schreibvorgang pro Sekunde aus. Der erste
/// Aufruf nach einer Idle-Phase startet einen Timer; weitere
/// Aktualisierungen innerhalb der Debounce-Frist werden
/// zusammengefasst.
pub(crate) struct LayoutDebouncer {
    inner: Rc<RefCell<DebounceState>>,
}

struct DebounceState {
    pending: bool,
    last_write: Option<std::time::Instant>,
}

impl LayoutDebouncer {
    pub(crate) fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(DebounceState {
                pending: false,
                last_write: None,
            })),
        }
    }

    /// Markiert, dass ein Schreibvorgang ansteht.
    pub(crate) fn mark_dirty(&self) {
        self.inner.borrow_mut().pending = true;
    }

    /// Sollte jetzt geschrieben werden? `true`, wenn seit dem letzten
    /// Schreibvorgang mindestens `DEBOUNCE_WINDOW` vergangen ist **und**
    /// ein Schreibvorgang ansteht.
    pub(crate) fn should_flush(&self) -> bool {
        let state = self.inner.borrow();
        if !state.pending {
            return false;
        }
        match state.last_write {
            None => true,
            Some(instant) => instant.elapsed() >= DEBOUNCE_WINDOW,
        }
    }

    /// Markiert den ausstehenden Schreibvorgang als erledigt.
    pub(crate) fn mark_flushed(&self) {
        let mut state = self.inner.borrow_mut();
        state.pending = false;
        state.last_write = Some(std::time::Instant::now());
    }

    /// Erzwingt einen Schreibvorgang unabhängig vom Zeitfenster
    /// (z. B. beim Shutdown).
    pub(crate) fn force_flush(&self) {
        let mut state = self.inner.borrow_mut();
        state.pending = true;
        state.last_write = None;
    }
}

const DEBOUNCE_WINDOW: std::time::Duration = std::time::Duration::from_millis(1000);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::StartConfig;
    use crate::workspace::profile::{ProfileId, StartProfile};

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("iv-workspace-test-{label}-{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn debouncer_idle_is_quiet() {
        let debouncer = LayoutDebouncer::new();
        assert!(!debouncer.should_flush());
    }

    #[test]
    fn debouncer_dirty_is_ready() {
        let debouncer = LayoutDebouncer::new();
        debouncer.mark_dirty();
        assert!(debouncer.should_flush());
        debouncer.mark_flushed();
        assert!(!debouncer.should_flush());
    }

    #[test]
    fn debouncer_force_flush() {
        let debouncer = LayoutDebouncer::new();
        debouncer.mark_flushed();
        assert!(!debouncer.should_flush());
        debouncer.force_flush();
        assert!(debouncer.should_flush());
    }

    #[test]
    fn profile_round_trip_via_toml() {
        let dir = unique_temp_dir("roundtrip");
        let storage = WorkspaceStorage::new(dir.clone());
        assert_eq!(storage.root(), dir.as_path());
        let config = StartConfig::new("/tmp", None, None).unwrap();
        let profile = StartProfile::new(ProfileId::new(0), "Arbeit", config).unwrap();

        let runtime = gtk::glib::MainContext::new();
        runtime.block_on(async {
            storage
                .save_profiles(std::slice::from_ref(&profile))
                .await
                .unwrap();
        });

        let (loaded, warnings) = runtime.block_on(async { storage.load_profiles().await });
        assert!(warnings.is_empty());
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0], profile);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_file_yields_empty() {
        let dir = unique_temp_dir("missing");
        let storage = WorkspaceStorage::new(dir.clone());
        let runtime = gtk::glib::MainContext::new();
        let (profiles, warnings) = runtime.block_on(async { storage.load_profiles().await });
        assert!(profiles.is_empty());
        assert!(warnings.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unsupported_version_yields_warning() {
        let dir = unique_temp_dir("version");
        std::fs::write(
            dir.join("profiles.toml"),
            b"schema_version = 99\n[[profile]]\nid = 0\nname = \"x\"\n",
        )
        .unwrap();
        let storage = WorkspaceStorage::new(dir.clone());
        let runtime = gtk::glib::MainContext::new();
        let (profiles, warnings) = runtime.block_on(async { storage.load_profiles().await });
        assert!(profiles.is_empty());
        assert!(matches!(
            warnings.as_slice(),
            [WorkspaceWarning::UnsupportedVersion(99)]
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn invalid_toml_yields_warning() {
        let dir = unique_temp_dir("invalid");
        std::fs::write(dir.join("profiles.toml"), b"[profile\n").unwrap();
        let storage = WorkspaceStorage::new(dir.clone());
        let runtime = gtk::glib::MainContext::new();
        let (profiles, warnings) = runtime.block_on(async { storage.load_profiles().await });
        assert!(profiles.is_empty());
        assert_eq!(warnings, vec![WorkspaceWarning::InvalidToml]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_atomic_does_not_leak_tmp() {
        let dir = unique_temp_dir("atomic");
        let storage = WorkspaceStorage::new(dir.clone());
        let runtime = gtk::glib::MainContext::new();
        let config = StartConfig::new("/tmp", None, None).unwrap();
        let profile = StartProfile::new(ProfileId::new(0), "Arbeit", config).unwrap();
        runtime.block_on(async {
            storage.save_profiles(&[profile]).await.unwrap();
        });
        let files: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .filter_map(|e| e.file_name().to_str().map(|s| s.to_owned()))
            .collect();
        assert!(files.contains(&"profiles.toml".to_owned()));
        assert!(!files.iter().any(|f| f.ends_with(".tmp")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn layout_round_trip_via_toml() {
        let dir = unique_temp_dir("layout-roundtrip");
        let storage = WorkspaceStorage::new(dir.clone());
        let snapshot = LayoutSnapshot::from_collection(&crate::tab::TabCollection::new(), None);
        let runtime = gtk::glib::MainContext::new();

        runtime.block_on(async {
            storage.save_layout(&snapshot).await.unwrap();
        });
        let (loaded, warnings) = runtime.block_on(async { storage.load_layout().await });

        assert!(warnings.is_empty());
        assert_eq!(loaded, snapshot);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn duplicate_profile_ids_are_rejected() {
        let config = StartConfig::new("/tmp", None, None).unwrap();
        let first = StartProfile::new(ProfileId::new(1), "One", config.clone()).unwrap();
        let second = StartProfile::new(ProfileId::new(1), "Two", config).unwrap();
        let contents = serialize_profiles(&[first, second]).unwrap();

        let (profiles, warnings) = WorkspaceStorage::parse_profiles(contents.as_bytes());

        assert!(profiles.is_empty());
        assert_eq!(warnings, [WorkspaceWarning::InvalidToml]);
    }

    #[test]
    fn save_does_not_overwrite_unknown_schema_version() {
        let dir = unique_temp_dir("preserve-version");
        let path = dir.join("profiles.toml");
        let original = "schema_version = 99\n";
        std::fs::write(&path, original).unwrap();
        let storage = WorkspaceStorage::new(dir.clone());
        let runtime = gtk::glib::MainContext::new();

        let result = runtime.block_on(async { storage.save_profiles(&[]).await });

        assert!(matches!(
            result,
            Err(WorkspaceError::UnsupportedVersion(99))
        ));
        assert_eq!(std::fs::read_to_string(path).unwrap(), original);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_does_not_overwrite_semantically_invalid_file() {
        let dir = unique_temp_dir("preserve-invalid");
        let path = dir.join("profiles.toml");
        let original = "schema_version = 1\n[[profile]]\nname = \"missing fields\"\n";
        std::fs::write(&path, original).unwrap();
        let storage = WorkspaceStorage::new(dir.clone());
        let runtime = gtk::glib::MainContext::new();

        let result = runtime.block_on(async { storage.save_profiles(&[]).await });

        assert!(matches!(result, Err(WorkspaceError::InvalidToml)));
        assert_eq!(std::fs::read_to_string(path).unwrap(), original);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
