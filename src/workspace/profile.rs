//! Startprofile.
//!
//! Ein [`StartProfile`] bündelt einen Namen mit einer [`StartConfig`]
//! (Verzeichnis, optionale Shell, optionaler Startbefehl). Profile werden
//! in `profiles.toml` gespeichert und über [`ProfileId`] identifiziert.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::workspace::StartConfig;
use crate::workspace::WorkspaceError;

/// Stabile Identität eines Profils innerhalb der aktuellen
/// Konfigurationsdatei. IDs werden nicht aus Profilnamen abgeleitet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub(crate) struct ProfileId(u64);

impl ProfileId {
    pub(crate) const fn new(id: u64) -> Self {
        Self(id)
    }
}

impl fmt::Display for ProfileId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "profile-{}", self.0)
    }
}

/// Ein benanntes Startprofil.
///
/// Profile sind im MVP einfach gehalten: ein Anzeigename und eine
/// [`StartConfig`]. Validierung erfolgt **schema-only** – die reale
/// Existenz des Arbeitsverzeichnisses wird ausschließlich beim Anwenden
/// geprüft.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StartProfile {
    pub(crate) id: ProfileId,
    pub(crate) name: String,
    pub(crate) start_config: StartConfig,
}

impl StartProfile {
    /// Erstellt ein neues Profil und validiert Name und `start_config`
    /// (Schema-only, keine Existenzprüfung).
    pub(crate) fn new(
        id: ProfileId,
        name: impl Into<String>,
        start_config: StartConfig,
    ) -> Result<Self, WorkspaceError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(WorkspaceError::MissingField("profile.name"));
        }
        start_config.validate()?;
        Ok(Self {
            id,
            name,
            start_config,
        })
    }

    pub(crate) fn rename(&mut self, name: impl Into<String>) -> Result<(), WorkspaceError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(WorkspaceError::MissingField("profile.name"));
        }
        self.name = name;
        Ok(())
    }

    pub(crate) fn validate(&self) -> Result<(), WorkspaceError> {
        if self.name.trim().is_empty() {
            return Err(WorkspaceError::MissingField("profile.name"));
        }
        self.start_config.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::StartConfig;

    fn valid_start_config() -> StartConfig {
        StartConfig::new("/tmp", None, None).expect("schema-valid")
    }

    #[test]
    fn new_accepts_non_empty_name() {
        let profile = StartProfile::new(ProfileId::new(1), "Arbeit", valid_start_config());
        assert!(profile.is_ok());
    }

    #[test]
    fn new_rejects_empty_name() {
        let profile = StartProfile::new(ProfileId::new(1), "   ", valid_start_config());
        assert!(matches!(
            profile,
            Err(WorkspaceError::MissingField("profile.name"))
        ));
    }

    #[test]
    fn rename_rejects_empty() {
        let mut profile =
            StartProfile::new(ProfileId::new(1), "Arbeit", valid_start_config()).unwrap();
        assert!(profile.rename("").is_err());
        assert!(profile.rename("Neu").is_ok());
        assert_eq!(profile.name, "Neu");
    }

    #[test]
    fn id_display_is_stable() {
        assert_eq!(ProfileId::new(42).to_string(), "profile-42");
    }
}
