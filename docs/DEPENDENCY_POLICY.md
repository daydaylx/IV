# Abhängigkeitsrichtlinie

## Ziel

Abhängigkeiten sollen Sicherheit, Wartbarkeit und Startzeit nicht unnötig verschlechtern. Jede neue Bibliothek benötigt einen klaren Nutzen.

## Grundregeln

- Standardbibliothek und bereits verwendete Kernbibliotheken bevorzugen
- keine Abhängigkeit nur für wenige triviale Hilfsfunktionen
- direkte Abhängigkeiten klein halten
- Features gezielt aktivieren, nicht pauschal alle
- keine ungewarteten oder unbekannten Kernabhängigkeiten ohne Prüfung
- keine Runtime oder UI-Technologie zusätzlich zu GTK4/libadwaita ohne Architekturentscheidung

## Prüfung vor Aufnahme

- löst sie ein reales aktuelles Problem?
- ist die Lizenz mit dem Projekt vereinbar?
- wird sie aktiv gepflegt?
- wie groß ist der transitive Abhängigkeitsbaum?
- verarbeitet sie Secrets, Netzwerk oder Prozesse?
- beeinflusst sie Binary-Größe, Startzeit oder Speicher?
- kann dieselbe Aufgabe mit vorhandenen Bibliotheken sauber gelöst werden?

## Besondere Kategorien

### Terminal und PTY

VTE bleibt im MVP die produktive Terminalimplementierung. Zusätzliche Parser oder Renderer werden nicht parallel eingeführt.

### Async

Es wird höchstens eine allgemeine Async-Runtime verwendet. GTK-Hauptkontext und Async-Runtime werden über klar definierte Grenzen gekoppelt.

### HTTP

Der Client muss Streaming, Abbruch, TLS, Zeitlimits und begrenzte Antwortgrößen unterstützen. Unsichere TLS-Ausnahmen werden nicht aktiviert.

### Serialisierung

TOML und eine etablierte Rust-Serialisierung werden bevorzugt. Konfigurationsschemata bleiben versionierbar.

### Secrets

Keyring-Zugriff erfolgt über eine gepflegte Linux-kompatible Bibliothek. Ein Klartext-Fallback ist unzulässig.

## Aktualisierungen

- Sicherheitsupdates erhalten Vorrang
- Major-Upgrades werden separat geprüft
- Lockfile wird versioniert
- automatische Updates dürfen Tests nicht umgehen
- veraltete Abhängigkeiten werden regelmäßig geprüft

## Entfernen

Abhängigkeiten werden entfernt, wenn sie ungenutzt, ungewartet, ersetzbar oder unverhältnismäßig schwer sind.

## Dokumentation

Architekturprägende Abhängigkeiten werden in einer ADR festgehalten. Kleinere Abhängigkeiten benötigen mindestens eine nachvollziehbare Begründung im Commit oder Pull Request.