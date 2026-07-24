# IV – Leichtgewichtiger KI-Terminal-Emulator

IV ist ein nativer, leichtgewichtiger Terminal-Emulator für Linux mit moderner, reduzierter Oberfläche und optionaler KI-Unterstützung.

Das Terminal bleibt jederzeit der Mittelpunkt. KI, Startprofile und Layouts sind ergänzende Funktionen und dürfen den normalen Terminalbetrieb weder verlangsamen noch komplizierter machen.

> **Aktueller Stand:** Planung und technische Vorbereitung für Phase 0. Der belastbare Rust-/GTK4-/VTE-Anwendungskern ist noch nicht als implementiert vorauszusetzen. Details: [`docs/PROJECT_STATE.md`](docs/PROJECT_STATE.md).

## Projektziel

IV soll ein schnelles, tastaturorientiertes Alltagsterminal für klassische Shell-Arbeit und terminalbasierte Coding-Agenten werden.

Geplante Einsatzbereiche:

- lokale Shell-Arbeit
- SSH
- tmux und zellij
- Pi Agent
- Codex CLI
- Claude Code
- andere interaktive Terminalprogramme

Das Projekt ist zunächst für die private Nutzung vorgesehen.

## Klare Abgrenzung

IV ist:

- ein Terminal-Emulator
- eine native Linux-Anwendung
- schnell und tastaturorientiert
- für klassische Shell-Arbeit geeignet
- optional um kontrollierte KI-Hilfen ergänzt

IV ist ausdrücklich **keine IDE**.

Nicht Bestandteil des MVP sind:

- integrierter Codeeditor oder Datei-Explorer
- Language Server oder Debugger
- visuelles Git-Frontend
- Aufgaben- oder Projektmanagement
- Plugin-Marktplatz
- eigener Coding-Agent oder Agenten-Orchestrierung
- MCP oder autonome Tool-Ausführung
- automatische Ausführung KI-generierter Befehle
- Cloudkonto, Synchronisierung oder Teamfunktionen
- eigener Terminalparser oder GPU-Renderer

Leitregel:

> Alles, was nicht direkt Terminalarbeit verbessert, gehört nicht in das MVP.

Die verbindlichen Grenzen für Coding-Agenten stehen in [`AGENTS.md`](AGENTS.md).

## Technische Basis

- **Sprache:** Rust
- **UI:** GTK4 + libadwaita
- **Terminal:** VTE
- **Plattform:** Linux und Wayland zuerst
- **Einstellungen:** TOML
- **Secrets:** System-Keyring
- **Hintergrundarbeit:** asynchron und vom GTK-Hauptthread getrennt

VTE wird hinter einer internen `TerminalBackend`-Grenze gekapselt. Ein späterer Backendwechsel bleibt möglich, ist aber kein Bestandteil des MVP.

## Geplanter MVP

### Terminal-Grundfunktionen

- lokale Standardshell starten
- zuverlässige PTY- und Prozessverwaltung
- Eingabe, Ausgabe und Scrollback
- Copy, Paste und Textauswahl
- Suche im Scrollback
- anklickbare Links
- Unicode und breite Zeichen
- zuverlässiges Resize
- Vollbild
- Schrift- und Theme-Einstellungen
- kontrollierter Prozessabschluss

### Tabs und Splits

- mehrere Tabs
- horizontale und vertikale Splits
- aktives Pane wechseln
- Pane-Größe ändern und Pane schließen
- neues Pane im aktuellen oder ursprünglichen Startverzeichnis
- Tabs umbenennen
- vollständige Tastaturbedienung

### Startprofile und Layouts

Ein Startprofil bleibt bewusst klein und kann enthalten:

- Name
- Startverzeichnis
- optionale Shell
- optionaler Startbefehl
- optionales Tab- und Split-Layout
- optional bevorzugtes KI-Modell

Nicht enthalten sind Projektanalyse, Build-System-Erkennung, Datei-Explorer oder integrierte Git-Verwaltung.

### Optionale KI-Unterstützung

Das MVP plant genau einen OpenAI-kompatiblen Provider.

Vorgesehene Funktionen:

- bewusst ausgewählten Terminaltext erklären
- Fehlermeldungen analysieren
- Shell-Befehle erzeugen, verbessern oder erklären
- Antworten streamen und abbrechen
- Vorschläge kopieren oder in die aktuelle Eingabe übernehmen

Verbindliche Sicherheitsgrenzen:

- keine automatische Befehlsausführung
- kein simuliertes Enter
- keine Tool-Aufrufe des Modells
- keine permanente Terminalüberwachung
- sichtbare Kontextvorschau vor dem Senden
- API-Schlüssel ausschließlich im System-Keyring
- Terminalbetrieb bleibt ohne Netzwerk und KI vollständig nutzbar

Details: [`docs/AI_INTEGRATION.md`](docs/AI_INTEGRATION.md) und [`docs/SECURITY.md`](docs/SECURITY.md).

## Entwicklungsreihenfolge

1. **Phase 0:** minimaler GTK4-/VTE-Prototyp mit einer Shell
2. **Phase 1:** alltagstaugliches Terminal mit Tabs und Splits
3. **Phase 2:** Startprofile und Layouts
4. **Phase 3:** optionale KI-Unterstützung
5. **Phase 4:** Stabilisierung, Messungen und Paketierung

Die vollständigen Ziele und Abschlusskriterien stehen in [`docs/ROADMAP.md`](docs/ROADMAP.md). Die aktuelle Phase steht ausschließlich in [`docs/PROJECT_STATE.md`](docs/PROJECT_STATE.md).

## Nächster Meilenstein

Der nächste technische Schritt ist eine minimale GTK4/libadwaita-Anwendung mit genau einem VTE-Terminal und sauberem Shell-Prozesslebenszyklus.

Noch nicht Teil dieses Schritts:

- Tabs und Splits
- Profile und Persistenz
- KI
- alternatives Terminal-Backend
- zusätzliche Plattformen

## Dokumentation

Der Einstieg in die technische Dokumentation ist [`docs/INDEX.md`](docs/INDEX.md).

Wichtige Dokumente:

- [`docs/PROJECT_STATE.md`](docs/PROJECT_STATE.md) – aktueller Umsetzungsstand
- [`docs/ROADMAP.md`](docs/ROADMAP.md) – geplante Reihenfolge
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) – Modulgrenzen
- [`docs/STATE_MODEL.md`](docs/STATE_MODEL.md) – Tabs, Splits und Prozesszustände
- [`docs/TERMINAL_BACKEND.md`](docs/TERMINAL_BACKEND.md) – VTE-/PTY-Grenze
- [`docs/SECURITY.md`](docs/SECURITY.md) – Vertrauens- und Sicherheitsmodell
- [`docs/TEST_STRATEGY.md`](docs/TEST_STRATEGY.md) – Testbereiche
- [`docs/DEFINITION_OF_DONE.md`](docs/DEFINITION_OF_DONE.md) – Abschluss einer Änderung
- [`agents/README.md`](agents/README.md) – Auswahl der Agentenrollen

## Erfolgskriterium

Das Projekt ist erfolgreich, wenn IV freiwillig als schnelles und angenehm bedienbares Alltagsterminal genutzt wird und die optionale KI-Unterstützung konkrete Terminalaufgaben verbessert, ohne daraus eine IDE oder einen autonomen Agenten zu machen.
