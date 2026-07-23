# Teststrategie

## Ziel

Tests sollen vor allem die Risiken eines Terminal-Emulators abdecken: PTY- und Prozesslebenszyklus, Eingabe, Ausgabe, Resize, Fokus, Splits, Unicode, Nebenläufigkeit und die strikte Trennung der KI-Funktionen vom Terminalbetrieb.

## Testebenen

### Unit-Tests

Für reine Logik ohne GTK- oder VTE-Abhängigkeit:

- Pane-Baum und Fokuswechsel
- Tab- und Pane-Invarianten
- Konfigurationsvalidierung
- Startprofile und Layoutserialisierung
- Fehlerabbildung
- Kontextfilter und Secret-Ausschlüsse
- Zustandsautomaten für Prozesse und KI-Anfragen

### Integrationstests

Für mehrere zusammenwirkende Komponenten:

- Terminal-Backend mit PTY und Testprozess
- Prozessstart, Ausgabe und Exit-Code
- kontrolliertes Schließen
- Layout laden und validieren
- KI-Streaming mit lokalem Mock-Server
- Request-Abbruch und Netzwerkfehler
- Keyring-Zugriff über austauschbare Testabstraktion

### UI- und manuelle Tests

GTK-/VTE-Verhalten wird zusätzlich in einer reproduzierbaren Testmatrix geprüft. Manuelle Tests sind kein Ersatz für Unit-Tests, aber für echte Terminalinteraktion unverzichtbar.

## Verbindliche Terminalprogramme

Vor einem MVP-Abschluss werden mindestens geprüft:

- bash
- zsh
- `vim` oder `nvim`
- `less`
- `htop`
- SSH
- tmux
- zellij
- Pi Agent
- Codex CLI oder Claude Code

## Kern-Testfälle

### Prozesslebenszyklus

- erfolgreiche Shell-Erstellung
- nicht vorhandenes Programm
- ungültiges Startverzeichnis
- Prozess mit Exit-Code 0
- Prozess mit Fehler-Exit-Code
- Fenster oder Pane während laufendem Prozess schließen
- keine Zombie-Prozesse nach dem Schließen
- genau ein Exit-Ereignis pro Prozess

### Eingabe und Ausgabe

- normale Texteingabe
- Steuerzeichen und Tastenkombinationen
- schnelle, lange Ausgabe
- große einzelne Zeilen
- ANSI-Farben und Cursorbewegungen
- interaktive Programme
- Einfügen mehrzeiliger Inhalte

### Darstellung

- Fenster-Resize
- Pane-Resize
- HiDPI
- Unicode
- Emojis
- breite CJK-Zeichen
- kombinierende Zeichen
- lange Scrollback-Historie
- Hell-, Dunkel- und System-Theme

### Tabs und Splits

- neue Tabs und Panes
- horizontale und vertikale Splits
- verschachtelte Splits
- Fokus in alle Richtungen
- Pane-Größe ändern
- aktives Pane schließen
- letztes Pane oder letzten Tab schließen
- neues Pane im aktuellen und ursprünglichen Verzeichnis

### Suche und Auswahl

- vorwärts und rückwärts suchen
- keine Treffer
- Unicode-Suchbegriffe
- Auswahl kopieren
- Zwischenablage nicht verfügbar
- Auswahl über mehrere sichtbare Zeilen

### Konfiguration und Profile

- fehlende Konfiguration
- gültige Konfiguration
- unbekannte Felder
- falsche Datentypen
- gelöschte Startverzeichnisse
- beschädigte Layoutdatei
- sichere Standardwerte nach Fehlern

### KI-Funktionen

- keine Konfiguration: Terminal bleibt nutzbar
- Provider nicht erreichbar
- Antwort-Streaming
- Abbruch während Streaming
- Pane wird während Request geschlossen
- markierter Kontext wird vor Versand angezeigt
- kein Kontext ohne bewusste Auswahl
- keine automatische Befehlsausführung
- Übernahme eines Befehls simuliert kein Enter
- bekannte Secret-Dateien werden ausgeschlossen
- Logs enthalten weder API-Key noch vollständigen Prompt

## Performance- und Stabilitätstests

Mindestens folgende Belastungen werden geprüft:

- mehrere Stunden Nutzung
- schnelle Dauerausgabe
- viele Scrollback-Zeilen
- mehrere Tabs und verschachtelte Splits
- wiederholtes Öffnen und Schließen von Panes
- wiederholte KI-Anfragen und Abbrüche
- Netzwerkfehler während Terminalausgabe

Messwerte und Budgets werden in `docs/PERFORMANCE_BUDGET.md` festgehalten.

## Regressionstests

Jeder behobene reproduzierbare Fehler erhält nach Möglichkeit einen Test auf der niedrigsten sinnvollen Ebene. Ist eine Automatisierung nicht realistisch, wird der Fall in der manuellen Testmatrix dokumentiert.

## Testdaten

- keine realen API-Schlüssel
- keine privaten SSH-Schlüssel
- keine echten vertraulichen Terminalprotokolle
- temporäre Verzeichnisse und synthetische Daten
- Mock-Provider für KI-Tests

## Ausführung vor Abschluss einer Änderung

Mindestens:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Falls das Projekt andere offizielle Befehle definiert, werden diese in `CONTRIBUTING.md` und CI dokumentiert.

## Freigabekriterien

Eine Änderung ist nicht abgeschlossen, wenn:

- relevante Tests fehlen
- bekannte Testfehler verschwiegen werden
- der Terminalbetrieb nur mit aktivem Netzwerk funktioniert
- Prozesslecks oder UI-Blockaden reproduzierbar sind
- sicherheitskritische KI-Regeln nicht überprüft wurden

Die vollständige Abschlussprüfung richtet sich zusätzlich nach `docs/DEFINITION_OF_DONE.md`.