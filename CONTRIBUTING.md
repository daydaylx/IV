# Mitwirken an IV

IV ist zunächst ein privates Projekt. Diese Regeln gelten trotzdem für Menschen und Coding-Agenten, damit Änderungen klein, prüfbar und konsistent bleiben.

## Vor jeder Änderung

Lies mindestens:

- `README.md`
- `AGENTS.md`
- `docs/ARCHITECTURE.md`
- `docs/DEFINITION_OF_DONE.md`
- das für die Aufgabe relevante Fachdokument

## Arbeitsweise

1. Problem und Ziel konkret beschreiben.
2. Nicht-Ziele festlegen.
3. Kleinste sinnvolle Änderung planen.
4. Bestehende Struktur und Tests prüfen.
5. Änderung implementieren.
6. Formatierung, Lints und Tests ausführen.
7. Diff auf Scope, Sicherheit und unnötige Komplexität prüfen.
8. Dokumentation aktualisieren, wenn Verhalten oder Architektur geändert wurden.

## Scope

Nicht ungefragt hinzufügen:

- IDE-Funktionen
- Plugin-System
- mehrere Provider
- eigenes Terminal-Rendering
- autonome KI-Ausführung
- neue Plattformen
- abstrakte Zukunftsarchitektur ohne aktuellen Bedarf

## Branches und Commits

- Änderungen thematisch trennen
- kleine, verständliche Commits
- Commit-Nachrichten beschreiben die Wirkung
- keine generischen Nachrichten wie `update` oder `fix stuff`
- keine fremden oder unabhängigen Änderungen im selben Commit

Beispiele:

```text
feat: add terminal process state model
fix: release pane process after close
refactor: isolate VTE backend types
docs: define AI context security rules
```

## Rust-Qualität

Vor Abschluss nach Möglichkeit:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

- kein `unwrap()` oder `expect()` in normalen Laufzeitpfaden ohne klare Begründung
- keine blockierenden Arbeiten im GTK-Hauptthread
- Fehler typisieren und mit Kontext weitergeben
- öffentliche APIs klein halten
- globale mutable Zustände vermeiden

## Tests

Neue Logik benötigt Tests auf der niedrigsten sinnvollen Ebene. Fehlerbehebungen erhalten möglichst einen Regressionstest. Manuelle Terminaltests werden dokumentiert, wenn Automatisierung nicht ausreicht.

## Architekturentscheidungen

Eine ADR ist erforderlich bei Änderungen an:

- Terminal-Backend
- Zustandsmodell
- Async-Runtime
- Persistenzformat
- Secret-Speicherung
- KI-Providerarchitektur
- grundlegender Modulstruktur

Vorlage: `docs/decisions/000-template.md`.

## Review

Reviews priorisieren:

1. Produktgrenzen
2. Prozess- und Ressourcenlebenszyklus
3. Sicherheit und Secret-Schutz
4. UI-Thread und Nebenläufigkeit
5. Zustandskonsistenz
6. Tests
7. Lesbarkeit und Stil

## Fertigstellung

Eine Aufgabe ist erst abgeschlossen, wenn die einschlägigen Punkte aus `docs/DEFINITION_OF_DONE.md` erfüllt sind und bekannte Einschränkungen offen benannt wurden.