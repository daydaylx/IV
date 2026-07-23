# AGENTS.md

## Auftrag

Arbeite an **IV**, einem nativen, leichtgewichtigen Linux-Terminal-Emulator mit optionaler KI-Unterstützung. Das Terminal bleibt immer Hauptprodukt. Lies vor jeder Änderung `README.md`, `docs/ARCHITECTURE.md` und `docs/DEFINITION_OF_DONE.md`.

## Verbindliche Produktgrenzen

IV ist **keine IDE**. Nicht hinzufügen:

- Codeeditor, Datei-Explorer, LSP oder Debugger
- visuelles Git-Frontend oder Projektmanagement
- Plugin-Marktplatz, Cloudkonto oder Teamfunktionen
- eigener Coding-Agent, MCP oder autonome Tool-Ausführung
- automatische Ausführung KI-generierter Befehle
- permanente Analyse oder Überwachung aller Terminalausgaben
- mehrere KI-Provider vor Abschluss des MVP
- eigener Terminalparser oder Renderer im MVP

Neue Funktionen müssen unmittelbar Terminalarbeit verbessern. Bei Zweifeln nicht implementieren, sondern als offene Entscheidung dokumentieren.

## Technische Leitplanken

- Sprache: Rust
- UI: GTK4 + libadwaita
- Terminal: VTE, hinter einer internen `TerminalBackend`-Abstraktion
- Plattform: Linux/Wayland zuerst
- Einstellungen: TOML
- Secrets: ausschließlich System-Keyring
- Netzwerk-, Git- und KI-Aufgaben niemals im UI-Thread
- Keine Telemetrie

## Architekturregeln

1. UI, Terminalsteuerung, Domänenlogik und Infrastruktur trennen.
2. VTE-spezifische Typen nicht außerhalb des Backend-Moduls verbreiten.
3. Terminalbetrieb darf nicht von KI, Netzwerk oder Persistenz abhängen.
4. Tabs und Splits als explizites Zustandsmodell behandeln, nicht über verstreute GTK-Callbacks.
5. Fehler als typisierte Ergebnisse weitergeben; keine stillen Fehler.
6. Globale mutable Zustände vermeiden.
7. Keine vorsorglichen Abstraktionen ohne zweiten konkreten Anwendungsfall.
8. Kleine, überprüfbare Änderungen bevorzugen.

## Sicherheitsregeln

- KI-Vorschläge dürfen nur in die Eingabe übernommen werden; Enter bleibt Nutzeraktion.
- Vor KI-Anfragen muss der gesendete Terminalkontext sichtbar sein.
- `.env`, Schlüsseldateien, Tokens und bekannte Secrets standardmäßig ausschließen oder maskieren.
- API-Schlüssel niemals loggen, in TOML schreiben oder in SQLite speichern.
- Logs dürfen keine vollständigen sensiblen Terminalausgaben enthalten.
- Riskante Shell-Befehle deutlich kennzeichnen, aber nicht selbst ausführen.

## Arbeitsablauf

1. Aufgabe gegen MVP und Nicht-Ziele prüfen.
2. Relevante Dateien lesen und bestehendes Verhalten verstehen.
3. Risiken, Annahmen und betroffene Module kurz benennen.
4. Kleinste sinnvolle Lösung planen.
5. Nur angeforderte Änderungen umsetzen.
6. Formatierung, statische Prüfungen und passende Tests ausführen.
7. Manuelle Terminalprüfung nennen, wenn Automatisierung nicht genügt.
8. Ergebnis mit geänderten Dateien, Prüfungen, offenen Risiken und nächstem Schritt abschließen.

## Qualitätsregeln

- Keine Platzhalter, Scheinfunktionen oder auskommentierten Altimplementierungen.
- Keine unnötigen Abhängigkeiten.
- Öffentliche APIs dokumentieren.
- Nutzerfehler verständlich anzeigen; technische Details in sichere Logs.
- UI vollständig tastaturbedienbar halten.
- Fokus, Resize, Prozessende und Fehlerzustände immer mitprüfen.
- Performance nicht behaupten, sondern messen.

## Pflichtprüfungen

Sobald ein Rust-Projekt vorhanden ist, mindestens:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

Bei Terminaländerungen zusätzlich manuell prüfen:

- Shell starten und beenden
- Resize
- Copy/Paste
- Unicode und breite Zeichen
- `vim`, `less`, `htop`
- keine Zombie-Prozesse

## Abschlussformat für Agenten

- **Geändert:** Dateien und Verhalten
- **Geprüft:** ausgeführte Tests/Checks
- **Risiken:** verbleibende technische Unsicherheiten
- **Nicht geändert:** bewusst ausgeschlossene Bereiche
- **Nächster Schritt:** genau eine sinnvolle Fortsetzung
