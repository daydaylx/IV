//! Typisierte Fehler und Warnungen für das Workspace-Modul.
//!
//! Fehler und Warnungen folgen dem Muster aus `terminal::TerminalError`
//! und `settings::SettingsWarning`: trennscharf, mit kurzen
//! Nutzertexten über [`user_message`](WorkspaceError::user_message) bzw.
//! [`user_message`](WorkspaceWarning::user_message).

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WorkspaceError {
    /// Beim Lesen einer Konfigurationsdatei ist ein I/O-Fehler aufgetreten.
    Io(String),
    /// Die Datei enthält kein gültiges UTF-8.
    InvalidUtf8,
    /// Die Datei ist kein gültiges TOML.
    InvalidToml,
    /// Die Datei trägt eine unbekannte Schema-Version.
    UnsupportedVersion(u32),
    /// Ein Pflichtfeld fehlt oder ist leer.
    MissingField(&'static str),
    /// Ein Pfad erfüllt die Schema-Validierung nicht.
    InvalidPath(String),
    /// Eine Profil-ID wird referenziert, ist aber nicht definiert.
    UnknownProfile(String),
    /// Allgemeiner Serialisierungsfehler beim Schreiben.
    Serialize(String),
    /// Der gespeicherte Pane-Baum verletzt eine Zustandsinvariante.
    InvalidLayout,
}

impl WorkspaceError {
    pub(crate) fn user_message(&self) -> String {
        match self {
            Self::Io(_) => "Die Workspace-Konfiguration konnte nicht gelesen werden.".to_owned(),
            Self::InvalidUtf8 => {
                "Die Workspace-Konfiguration enthält ungültige Zeichen.".to_owned()
            }
            Self::InvalidToml => "Die Workspace-Konfiguration ist kein gültiges TOML.".to_owned(),
            Self::UnsupportedVersion(_) => {
                "Die Workspace-Konfiguration hat ein unbekanntes Format und wird verworfen."
                    .to_owned()
            }
            Self::MissingField(field) => {
                format!("Pflichtfeld fehlt in der Workspace-Konfiguration: {field}.")
            }
            Self::InvalidPath(reason) => {
                format!("Ungültiger Pfad in der Workspace-Konfiguration: {reason}.")
            }
            Self::UnknownProfile(id) => {
                format!("Profil „{id}“ ist nicht definiert.")
            }
            Self::Serialize(_) => {
                "Die Workspace-Konfiguration konnte nicht geschrieben werden.".to_owned()
            }
            Self::InvalidLayout => {
                "Das gespeicherte Terminal-Layout ist ungültig und wird verworfen.".to_owned()
            }
        }
    }
}

impl fmt::Display for WorkspaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.user_message())
    }
}

impl std::error::Error for WorkspaceError {}

impl From<toml::de::Error> for WorkspaceError {
    fn from(_error: toml::de::Error) -> Self {
        Self::InvalidToml
    }
}

impl From<toml::ser::Error> for WorkspaceError {
    fn from(error: toml::ser::Error) -> Self {
        Self::Serialize(error.to_string())
    }
}

/// Nicht-blockierende Probleme beim Laden der Workspace-Konfiguration.
///
/// Warnungen werden im UI sichtbar gemacht, führen aber nicht zum
/// Abbruch. Die Anwendung fällt auf sichere Defaults zurück.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WorkspaceWarning {
    /// Datei konnte nicht gelesen werden.
    ReadFailed,
    /// Datei ist kein gültiges TOML.
    InvalidToml,
    /// Datei trägt eine unbekannte Schema-Version.
    UnsupportedVersion(u32),
    /// Beim Anwenden wurde ein nicht existierendes Verzeichnis verwendet.
    DirectoryMissing(String),
    /// Das aktive Profil wurde nicht gefunden; das aktive Profil wird
    /// auf „kein Profil" zurückgesetzt.
    ActiveProfileMissing,
}

impl WorkspaceWarning {
    pub(crate) fn user_message(&self) -> String {
        match self {
            Self::ReadFailed => {
                "Der Workspace-Ordner konnte nicht gelesen werden; Standardwerte werden verwendet."
                    .to_owned()
            }
            Self::InvalidToml => {
                "Eine Workspace-Datei ist kein gültiges TOML; Standardwerte werden verwendet."
                    .to_owned()
            }
            Self::UnsupportedVersion(_) => {
                "Eine Workspace-Datei hat ein unbekanntes Format; Standardwerte werden verwendet."
                    .to_owned()
            }
            Self::DirectoryMissing(path) => {
                format!("Das Verzeichnis „{path}“ existiert nicht; HOME wird verwendet.")
            }
            Self::ActiveProfileMissing => {
                "Das aktive Profil existiert nicht mehr; es wird auf „kein Profil“ zurückgesetzt."
                    .to_owned()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_messages_are_non_empty() {
        let cases = [
            WorkspaceError::Io("x".into()),
            WorkspaceError::InvalidUtf8,
            WorkspaceError::InvalidToml,
            WorkspaceError::UnsupportedVersion(99),
            WorkspaceError::MissingField("name"),
            WorkspaceError::InvalidPath("traversal".into()),
            WorkspaceError::UnknownProfile("p1".into()),
            WorkspaceError::Serialize("oops".into()),
            WorkspaceError::InvalidLayout,
        ];
        for error in &cases {
            assert!(!error.user_message().is_empty());
        }
    }

    #[test]
    fn warning_messages_are_non_empty() {
        let cases = [
            WorkspaceWarning::ReadFailed,
            WorkspaceWarning::InvalidToml,
            WorkspaceWarning::UnsupportedVersion(2),
            WorkspaceWarning::DirectoryMissing("/nope".into()),
            WorkspaceWarning::ActiveProfileMissing,
        ];
        for warning in &cases {
            assert!(!warning.user_message().is_empty());
        }
    }
}
