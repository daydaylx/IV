# Mitwirken an IV

IV ist zunächst ein privates Projekt. Diese Regeln gelten trotzdem für Menschen und Coding-Agenten, damit Änderungen klein, prüfbar und konsistent bleiben.

## Vor jeder Änderung

1. [`AGENTS.md`](AGENTS.md) lesen.
2. Den tatsächlichen Stand in [`docs/PROJECT_STATE.md`](docs/PROJECT_STATE.md) prüfen.
3. Über [`docs/INDEX.md`](docs/INDEX.md) die für die Aufgabe relevanten Fachdokumente bestimmen.
4. Bestehenden Code und vorhandene Tests im betroffenen Bereich untersuchen.

Die gesamte Dokumentation muss nicht für jede kleine Aufgabe geladen werden.

## Arbeitsweise

1. Problem und Ziel konkret beschreiben.
2. Nicht-Ziele und Annahmen festlegen.
3. Kleinste sinnvolle Änderung planen.
4. Bestehende Muster und Tests prüfen.
5. Änderung ohne unabhängige Nebenrefaktorierungen umsetzen.
6. Formatierung, Lints und passende Tests ausführen.
7. Diff auf Scope, Sicherheit und unnötige Komplexität prüfen.
8. Zuständiges Fachdokument aktualisieren, wenn Verhalten oder Architektur geändert wurden.
9. `docs/PROJECT_STATE.md` nur bei einer tatsächlichen Änderung des Projektstands anpassen.
10. Einschlägige Punkte aus `docs/DEFINITION_OF_DONE.md` prüfen.

## Scope

Die verbindlichen Produktgrenzen stehen ausschließlich in [`AGENTS.md`](AGENTS.md). Keine lokale Kopie dieser Liste in Beiträgen, Pull Requests oder Agentenprofilen pflegen.

Bei unklarem Umfang gilt:

- keine neue Funktion erfinden,
- keine hypothetische Zukunftsarchitektur bauen,
- Annahme offen nennen,
- kleinste zur aktuellen Phase passende Lösung wählen.

## Rollen und Reviews

Die Auswahl und Reihenfolge der Agentenrollen steht in [`agents/README.md`](agents/README.md).

Mindestens:

- nicht triviale Implementierungen erhalten einen technischen Review,
- sicherheitsrelevante Änderungen erhalten zusätzlich einen Security-Review,
- Architekturänderungen benötigen vor der Umsetzung eine akzeptierte ADR.

## Branches und Commits

- Änderungen thematisch trennen.
- Kleine, verständliche Commits bevorzugen.
- Commit-Nachrichten beschreiben die Wirkung.
- Keine fremden oder unabhängigen Änderungen im selben Commit.
- Dokumentations- und Codeänderungen dürfen gemeinsam erfolgen, wenn sie dasselbe Verhalten betreffen.

Beispiele:

```text
feat: add terminal process state model
fix: release pane process after close
refactor: isolate VTE backend types
docs: clarify AI context security rules
```

Generische Nachrichten wie `update` oder `fix stuff` vermeiden.

## Rust-Qualität

Sobald ein Rust-Projekt vorhanden ist, vor Abschluss mindestens:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Zusätzlich:

- kein `unwrap()` oder `expect()` in normalen Laufzeitpfaden ohne klare Begründung,
- keine blockierende Arbeit im GTK-Hauptthread,
- Fehler typisieren und mit sicherem Kontext weitergeben,
- öffentliche APIs klein halten,
- globale mutable Zustände vermeiden,
- Abbruch- und Ressourcenpfade mitprüfen.

## Tests

Neue Logik benötigt Tests auf der niedrigsten sinnvollen Ebene. Reproduzierbare Fehler erhalten möglichst einen Regressionstest. Manuelle Terminaltests werden dokumentiert, wenn GTK-, VTE- oder echte Terminalinteraktion nicht sinnvoll automatisierbar sind.

Die verbindlichen Testbereiche stehen in [`docs/TEST_STRATEGY.md`](docs/TEST_STRATEGY.md).

## Architekturentscheidungen

Eine ADR ist erforderlich bei Änderungen an:

- Terminal-Backend,
- Zustandsmodell,
- Async-Runtime oder Thread-Grenzen,
- Persistenzformat,
- Secret-Speicherung,
- KI-Providerarchitektur,
- grundlegender Modulstruktur.

Ablauf und Vorlage stehen in [`docs/decisions/README.md`](docs/decisions/README.md).

## Fertigstellung

Eine Änderung ist erst abgeschlossen, wenn:

- die einschlägigen Punkte aus [`docs/DEFINITION_OF_DONE.md`](docs/DEFINITION_OF_DONE.md) erfüllt sind,
- ausgeführte und nicht ausführbare Prüfungen offen genannt wurden,
- bekannte Einschränkungen oder Risiken nicht verschwiegen werden,
- bewusst ausgeschlossener Umfang dokumentiert ist.
