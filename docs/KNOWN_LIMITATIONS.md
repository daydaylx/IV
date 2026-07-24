# Bekannte Einschränkungen

## Dokumentstatus

Dieses Dokument enthält bestätigte oder bewusst akzeptierte Einschränkungen. Allgemeine Produkt-Nicht-Ziele stehen in `AGENTS.md`; zukünftige Phasen stehen in `ROADMAP.md`.

Eine Einschränkung darf erst als behoben entfernt werden, wenn Implementierung und passende Tests dies belegen.

## Aktueller Projektstand

Der Rust-/GTK4-/VTE-Anwendungskern und die vorgesehenen Phase-1-Kernfunktionen sind implementiert. Phase 1 ist wegen offener Wayland-, Langzeit- und realer TUI-Prüfungen noch nicht abgeschlossen.

Aktuelle Details: [`PROJECT_STATE.md`](PROJECT_STATE.md).

## Plattform

- Linux ist die einzige geplante MVP-Plattform.
- Wayland ist die primäre Zielumgebung.
- X11 wird nur unterstützt, soweit GTK4 und VTE dies ohne wesentliche Zusatzarchitektur tragen.
- Windows und macOS sind nicht Teil des MVP.

## Terminal-Backend

- VTE ist die einzige produktive Terminalimplementierung des MVP.
- Ein Backendwechsel zur Laufzeit ist nicht vorgesehen.
- Es gibt im MVP keinen eigenen Terminalparser und keinen eigenen GPU-Renderer.
- VTE ist hinter einer internen `Terminal`-Fassade gekapselt; eine austauschbare Laufzeit-Backendauswahl gibt es nicht.

## Prozesse und Sitzungen

- Gespeicherte Layouts stellen Struktur und Startkonfiguration wieder her, nicht laufende Prozesse.
- Terminalbuffer werden im MVP nicht dauerhaft gespeichert.
- Das aktuelle Arbeitsverzeichnis kann technisch unbekannt sein.
- Ist das aktuelle Verzeichnis unbekannt oder ungültig, muss auf einen bestätigten Startpfad zurückgefallen werden.
- Vollständige Wiederaufnahme einer laufenden Shell nach einem Anwendungsneustart ist nicht vorgesehen.
- Die Daten- und Storage-Grundlage für Startprofile und Layoutpersistenz ist vorhanden, aber noch nicht an Startsequenz, Terminalstart oder UI angebunden.

## KI-Unterstützung

- Das MVP unterstützt genau einen OpenAI-kompatiblen Provider.
- KI funktioniert nur nach bewusster Nutzeraktion und ist keine Voraussetzung für Terminalbetrieb.
- Secret-Erkennung kann bekannte Muster reduzieren, aber keine vollständige Erkennung garantieren.
- Modellantworten können sachlich falsch, unvollständig oder riskant sein.
- Risikowarnungen für Befehle sind eine zusätzliche Hilfe und keine Sicherheitsgarantie.
- Gesprächsverlauf über Sitzungen hinweg wird im MVP nicht gespeichert.

Die verbindlichen Grenzen stehen in [`AI_INTEGRATION.md`](AI_INTEGRATION.md) und [`SECURITY.md`](SECURITY.md).

## Zustands- und Verzeichniserkennung

- Prozessstatus kann nur technisch erkennbare Zustände anzeigen.
- IV kann nicht zuverlässig behaupten, ob ein fremdes Terminalprogramm „denkt“, „plant“ oder „hängt“.
- Der Zeitpunkt der letzten Ausgabe ist nur ein technischer Hinweis und kein Beweis für Stillstand.
- Arbeitsverzeichniserkennung ist bestmöglich; unbekannt bleibt ein gültiger Zustand.

## Oberfläche

- Die Terminalfläche bleibt dominant; permanente IDE-Seitenleisten oder Chatblöcke sind nicht vorgesehen.
- Kernfunktionen müssen ohne Maus erreichbar sein.
- Sehr komplexe Sitzungs-, Projekt- oder Fensterverwaltung ist kein Ziel des MVP.
- Vollbild ist noch nicht implementiert.
- Die Scrollback-Suche ist case-sensitive und durchsucht genau das Pane, in dem sie geöffnet wurde.
- Pane-Größen lassen sich derzeit nur über den `gtk::Paned`-Griff mit der Maus verändern.
- Die Kürzel `Alt+Pfeiltaste` und `Ctrl+Shift+Q` können mit Terminalprogrammen kollidieren.

## Pflege

Neue Einträge benötigen:

- ein bestätigtes technisches oder produktbezogenes Limit,
- Auswirkungen auf Nutzer oder Implementierung,
- gegebenenfalls einen Verweis auf Test, Issue oder ADR.

Geplante, aber noch nicht implementierte Funktionen sind nicht automatisch „bekannte Einschränkungen“. Sie gehören in `ROADMAP.md` oder `PROJECT_STATE.md`.
