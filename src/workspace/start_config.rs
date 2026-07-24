//! Startkonfiguration für ein Pane.
//!
//! [`StartConfig`] bündelt Verzeichnis, optionale Shell und optionalen
//! Startbefehl. Sie lebt im `workspace/`-Modul, damit `pane/` und
//! `tab/` keine Pfad- oder Befehlslogik importieren müssen.

use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::workspace::WorkspaceError;

/// Beschreibung, wie ein Pane gestartet wird.
///
/// - `working_directory` ist ein bereits schema-validierter absoluter
///   Pfad (siehe [`validate_path`]).
/// - `shell` ist ein optionales ausführbares Programm. `None` bedeutet:
///   Shell aus Umgebung verwenden.
/// - `command` ist ein optionaler Startbefehl als Argumentliste. Das
///   erste Element ist das Programm, der Rest die Argumente.
///
/// **Existenz** des Verzeichnisses wird **nicht** durch `validate`
/// geprüft. Existenzprüfung erfolgt erst beim Anwenden, damit Profile
/// für noch nicht existierende Projektverzeichnisse anlegbar bleiben.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StartConfig {
    pub(crate) working_directory: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) shell: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) command: Option<Vec<String>>,
}

impl StartConfig {
    /// Erstellt eine neue Startkonfiguration und prüft das Schema.
    pub(crate) fn new(
        working_directory: impl AsRef<Path>,
        shell: Option<PathBuf>,
        command: Option<Vec<String>>,
    ) -> Result<Self, WorkspaceError> {
        let directory = validate_path(working_directory.as_ref())?;
        if let Some(program) = &shell {
            validate_path(program)?;
        }
        if let Some(args) = &command {
            if args.is_empty() {
                return Err(WorkspaceError::MissingField("command"));
            }
            validate_path(Path::new(&args[0]))?;
            if args.iter().any(|argument| argument.contains('\0')) {
                return Err(WorkspaceError::InvalidPath(
                    "Befehlsargument enthält ein NUL-Zeichen".to_owned(),
                ));
            }
        }
        Ok(Self {
            working_directory: directory,
            shell,
            command,
        })
    }

    /// Schema-Validierung der enthaltenen Pfade ohne Existenzprüfung.
    pub(crate) fn validate(&self) -> Result<(), WorkspaceError> {
        validate_path(&self.working_directory)?;
        if let Some(program) = &self.shell {
            validate_path(program)?;
        }
        if let Some(args) = &self.command {
            if args.is_empty() {
                return Err(WorkspaceError::MissingField("command"));
            }
            validate_path(Path::new(&args[0]))?;
            if args.iter().any(|argument| argument.contains('\0')) {
                return Err(WorkspaceError::InvalidPath(
                    "Befehlsargument enthält ein NUL-Zeichen".to_owned(),
                ));
            }
        }
        Ok(())
    }
}

/// Schema-Validierung eines Pfads.
///
/// Regeln:
/// - muss absolut sein
/// - darf keine `..`-Komponente enthalten
/// - `~` ist nur als eigenständiges Präfix erlaubt (z. B. `~/projects/foo`)
/// - das Ergebnis enthält keine `..`; ein zulässiges `~`-Präfix bleibt
///   für die spätere, nutzerbezogene Auflösung erhalten
///
/// Es wird **keine** Existenz geprüft.
pub(crate) fn validate_path(path: &Path) -> Result<PathBuf, WorkspaceError> {
    let raw = path
        .to_str()
        .ok_or_else(|| WorkspaceError::InvalidPath("kein UTF-8".to_owned()))?;
    if raw.is_empty() {
        return Err(WorkspaceError::InvalidPath("leerer Pfad".to_owned()));
    }
    if raw.contains('\0') {
        return Err(WorkspaceError::InvalidPath(
            "NUL-Zeichen ist nicht erlaubt".to_owned(),
        ));
    }
    let has_tilde_prefix = raw == "~" || raw.starts_with("~/");
    if !Path::new(raw).has_root() && !has_tilde_prefix {
        return Err(WorkspaceError::InvalidPath("nicht absolut".to_owned()));
    }
    for component in Path::new(raw).components() {
        if matches!(component, Component::ParentDir) {
            return Err(WorkspaceError::InvalidPath(
                "„..“ ist nicht erlaubt".to_owned(),
            ));
        }
    }
    Ok(PathBuf::from(raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_path_is_accepted() {
        let result = validate_path(Path::new("/home/user/project"));
        assert_eq!(result.unwrap(), PathBuf::from("/home/user/project"));
    }

    #[test]
    fn tilde_prefix_is_accepted() {
        let result = validate_path(Path::new("~/projects/foo"));
        assert_eq!(result.unwrap(), PathBuf::from("~/projects/foo"));
    }

    #[test]
    fn empty_path_is_rejected() {
        let result = validate_path(Path::new(""));
        assert!(matches!(result, Err(WorkspaceError::InvalidPath(_))));
    }

    #[test]
    fn relative_path_is_rejected() {
        let result = validate_path(Path::new("foo/bar"));
        assert!(matches!(result, Err(WorkspaceError::InvalidPath(_))));
    }

    #[test]
    fn named_user_tilde_is_rejected() {
        let result = validate_path(Path::new("~other/project"));
        assert!(matches!(result, Err(WorkspaceError::InvalidPath(_))));
    }

    #[test]
    fn traversal_is_rejected() {
        let result = validate_path(Path::new("/home/user/../etc"));
        assert!(matches!(result, Err(WorkspaceError::InvalidPath(_))));
    }

    #[test]
    fn start_config_rejects_traversal_in_directory() {
        let result = StartConfig::new("/home/../etc", None, None);
        assert!(matches!(result, Err(WorkspaceError::InvalidPath(_))));
    }

    #[test]
    fn start_config_rejects_empty_command() {
        let result = StartConfig::new("/tmp", None, Some(vec![]));
        assert!(matches!(
            result,
            Err(WorkspaceError::MissingField("command"))
        ));
    }

    #[test]
    fn start_config_accepts_minimal_input() {
        let result = StartConfig::new("/tmp", None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn start_config_accepts_command_with_arguments() {
        let result = StartConfig::new("/tmp", None, Some(vec!["/usr/bin/nvim".into()]));
        assert!(result.is_ok());
    }

    #[test]
    fn start_config_rejects_traversal_in_command() {
        let result = StartConfig::new("/tmp", None, Some(vec!["../bin".into()]));
        assert!(matches!(result, Err(WorkspaceError::InvalidPath(_))));
    }

    #[test]
    fn nul_bytes_are_rejected() {
        assert!(validate_path(Path::new("/tmp/\0bad")).is_err());
        let result = StartConfig::new(
            "/tmp",
            None,
            Some(vec!["/bin/echo".into(), "bad\0argument".into()]),
        );
        assert!(matches!(result, Err(WorkspaceError::InvalidPath(_))));
    }
}
