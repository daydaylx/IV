# Dokumentationsindex

## Zweck

Dieser Index legt fest, welche Dokumente für welche Aufgabe relevant sind. Agenten sollen nicht pauschal die gesamte Dokumentation laden, sondern nur den notwendigen Kontext.

## Verbindliche Grundreihenfolge

Vor jeder Aufgabe:

1. [`AGENTS.md`](../AGENTS.md) lesen.
2. [`PROJECT_STATE.md`](PROJECT_STATE.md) lesen.
3. Über diesen Index die aufgabenspezifischen Dokumente bestimmen.
4. Vor Abschluss die einschlägigen Punkte aus [`DEFINITION_OF_DONE.md`](DEFINITION_OF_DONE.md) prüfen.

## Dokumentzuständigkeiten

| Dokument | Hauptverantwortung |
|---|---|
| [`README.md`](../README.md) | Produktüberblick, aktueller Einstieg und öffentliche Projektbeschreibung |
| [`AGENTS.md`](../AGENTS.md) | verbindliche Regeln für Coding-Agenten |
| [`PROJECT_STATE.md`](PROJECT_STATE.md) | tatsächlich aktueller Projektstand und nächster Meilenstein |
| [`ROADMAP.md`](ROADMAP.md) | geplante Entwicklungsreihenfolge |
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | Modulgrenzen und Abhängigkeitsrichtung |
| [`STATE_MODEL.md`](STATE_MODEL.md) | fachlicher Zustand, Invarianten und Zustandsübergänge |
| [`TERMINAL_BACKEND.md`](TERMINAL_BACKEND.md) | Verantwortlichkeiten und Grenzen des Terminal-Backends |
| [`AI_INTEGRATION.md`](AI_INTEGRATION.md) | fachlicher und technischer Umfang der KI-Integration |
| [`SECURITY.md`](SECURITY.md) | Vertrauensgrenzen, Secret-Schutz und sicherheitskritische Regeln |
| [`LOGGING.md`](LOGGING.md) | zulässige und unzulässige Protokollierung |
| [`UX_PRINCIPLES.md`](UX_PRINCIPLES.md) | Bedienung, Layout, Fokus und Barrierearmut |
| [`KEYBINDINGS.md`](KEYBINDINGS.md) | Tastenkürzel und Konfliktregeln |
| [`TEST_STRATEGY.md`](TEST_STRATEGY.md) | Testebenen, Kernfälle und Regressionen |
| [`PERFORMANCE_BUDGET.md`](PERFORMANCE_BUDGET.md) | Messziele und Performance-Szenarien |
| [`DEPENDENCY_POLICY.md`](DEPENDENCY_POLICY.md) | Aufnahme und Pflege von Abhängigkeiten |
| [`MODULE_STRUCTURE.md`](MODULE_STRUCTURE.md) | aktuelle Modulaufteilung und erlaubte Abhängigkeiten |
| [`ASYNC_MODEL.md`](ASYNC_MODEL.md) | GLib-Hauptkontext, Hintergrundarbeit und Abbruch |
| [`ERROR_HANDLING.md`](ERROR_HANDLING.md) | Fehlerklassen, Übersetzung und Nutzerwirkung |
| [`CONFIGURATION.md`](CONFIGURATION.md) | implementiertes TOML-Schema, XDG-Pfad und Validierung |
| [`CONTEXT_LEDGER.md`](CONTEXT_LEDGER.md) | bestätigte dauerhafte Projektentscheidungen |
| [`KNOWN_LIMITATIONS.md`](KNOWN_LIMITATIONS.md) | bestätigte und bewusst akzeptierte Einschränkungen |
| [`DEFINITION_OF_DONE.md`](DEFINITION_OF_DONE.md) | Abschlusskriterien einer Änderung |
| [`RELEASE_CHECKLIST.md`](RELEASE_CHECKLIST.md) | Freigabe einer Version oder testbaren Zwischenversion |
| [`decisions/`](decisions/) | akzeptierte Architekturentscheidungen und verworfene Alternativen |

## Kontext nach Aufgabentyp

| Aufgabe | Zusätzlich lesen |
|---|---|
| GTK4, libadwaita oder allgemeine UI | `ARCHITECTURE.md`, `UX_PRINCIPLES.md` |
| VTE, PTY, Shellstart oder Prozessende | `TERMINAL_BACKEND.md`, `STATE_MODEL.md`, `TEST_STRATEGY.md` |
| Tabs, Splits oder Fokus | `STATE_MODEL.md`, `KEYBINDINGS.md`, `TEST_STRATEGY.md` |
| Einstellungen, Profile oder Persistenz | `ARCHITECTURE.md`, `CONFIGURATION.md`, `ASYNC_MODEL.md`, `SECURITY.md`, passende ADRs |
| KI-Kontext, Streaming oder Befehlsvorschläge | `AI_INTEGRATION.md`, `SECURITY.md`, `LOGGING.md`, `TEST_STRATEGY.md` |
| Netzwerk, Keyring, Secrets, Links oder Zwischenablage | `SECURITY.md`, `LOGGING.md`, `TEST_STRATEGY.md` |
| Performance oder Speicherverbrauch | `PERFORMANCE_BUDGET.md`, `TEST_STRATEGY.md` |
| neue Abhängigkeit | `DEPENDENCY_POLICY.md`, gegebenenfalls `SECURITY.md` |
| Architekturänderung | `ARCHITECTURE.md`, passende Fachdokumente und `decisions/` |
| Release oder stabile Zwischenversion | `RELEASE_CHECKLIST.md`, `KNOWN_LIMITATIONS.md`, `ROADMAP.md` |

## Autorität bei Abweichungen

Es gibt keine pauschale Datei, die jede andere Quelle überschreibt. Maßgeblich ist die zuständige Quelle:

1. `AGENTS.md` für Agentenverhalten und unverhandelbare Projektgrenzen.
2. Akzeptierte ADRs für bewusst getroffene Architekturentscheidungen.
3. Code und bestandene Tests für tatsächlich implementiertes Verhalten.
4. `PROJECT_STATE.md` für den aktuellen Umsetzungsstand.
5. Fachdokumente für den geplanten oder implementierten Bereich.
6. `ROADMAP.md` für zukünftige Reihenfolge, nicht für bereits vorhandenes Verhalten.

Widersprüche dürfen nicht still aufgelöst werden. Der Agent nennt die betroffenen Quellen und schlägt die kleinste konsistente Korrektur vor.

## Dokumentationsregeln

- Geplantes Verhalten nicht als bereits implementiert darstellen.
- Große Wiederholungen vermeiden; auf die zuständige Quelle verlinken.
- Architekturprägende Entscheidungen als ADR dokumentieren.
- Bei Verhaltensänderungen Code, Tests und zuständiges Fachdokument gemeinsam aktualisieren.
- `PROJECT_STATE.md` nach jedem abgeschlossenen Meilenstein aktualisieren.
