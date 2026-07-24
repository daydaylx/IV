# Architekturentscheidungen

## Zweck

Dieser Ordner enthält Architecture Decision Records für Entscheidungen, die langfristige technische Grenzen verändern oder festlegen. ADRs dokumentieren nicht jede kleine Implementierungsentscheidung.

## Wann eine ADR erforderlich ist

- Terminal-Backend oder dessen öffentliche Grenze
- fachliches Zustandsmodell
- Async-Runtime oder Thread-Modell
- Persistenzformat und Versionierung
- Secret-Speicherung
- KI-Providerarchitektur
- grundlegende Modul- oder Paketstruktur
- neue zentrale UI- oder Laufzeittechnologie

Kleine lokale Refaktorierungen, Fehlerkorrekturen und die Aufnahme unkritischer Hilfsabhängigkeiten benötigen normalerweise keine ADR.

## Status

Eine ADR verwendet einen dieser Statuswerte:

- `proposed` – Vorschlag, noch nicht verbindlich
- `accepted` – verbindliche Entscheidung
- `superseded` – durch eine neuere ADR ersetzt
- `rejected` – geprüft und verworfen

## Benennung

```text
001-kurzer-titel.md
002-naechste-entscheidung.md
```

Die Nummer bleibt dauerhaft stabil. Eine akzeptierte ADR wird nicht rückwirkend inhaltlich umgeschrieben; wesentliche Änderungen erhalten eine neue ADR, die die vorherige ersetzt.

## Mindestinhalt

- Status und Datum
- Kontext und konkretes Problem
- Entscheidung
- verworfene Alternativen
- Auswirkungen und Risiken
- Verifikation
- Rückbau- oder Migrationsmöglichkeit

Vorlage: [`000-template.md`](000-template.md).

## Autorität

Eine akzeptierte ADR ist für die von ihr behandelte Architekturentscheidung maßgeblich. Sie ersetzt jedoch nicht:

- `AGENTS.md` für Produktgrenzen und Agentenverhalten,
- Code und Tests für den tatsächlich implementierten Stand,
- `PROJECT_STATE.md` für die aktuelle Projektphase.

Wird eine ADR noch nicht umgesetzt, muss dies in `PROJECT_STATE.md` oder im betreffenden Fachdokument erkennbar bleiben.
