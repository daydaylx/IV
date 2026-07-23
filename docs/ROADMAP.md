# Roadmap

## Zweck

Diese Roadmap legt die Reihenfolge der Entwicklung von IV fest. Sie schützt das Projekt vor Scope Creep und verhindert, dass KI-, Profil- oder Komfortfunktionen vor einem stabilen Terminalkern entstehen.

## Grundregel

Eine Phase beginnt erst, wenn die Abschlusskriterien der vorherigen Phase erfüllt sind. Offene Punkte werden dokumentiert, aber nicht durch neue Architektur oder zusätzliche Funktionen umgangen.

## Phase 0 – Technischer Prototyp

### Ziel

Eine minimale GTK4-Anwendung bettet VTE zuverlässig ein und startet eine lokale Shell.

### Umfang

- Rust-Projektstruktur
- GTK4- und libadwaita-Anwendung
- ein Fenster
- ein VTE-Terminal
- Standardshell starten
- Copy und Paste
- Resize
- kontrolliertes Schließen
- minimale Fehlerausgabe

### Nicht enthalten

- Tabs
- Splits
- Einstellungen
- Profile
- KI
- Sitzungswiederherstellung

### Abschlusskriterien

- bash und zsh starten zuverlässig
- `vim`, `less` und `htop` sind nutzbar
- Unicode und breite Zeichen werden korrekt dargestellt
- Fenstergrößenänderungen beschädigen die Darstellung nicht
- beim Schließen bleiben keine Zombie-Prozesse zurück

## Phase 1 – Terminal-MVP

### Ziel

IV ist als alltäglicher Terminal-Emulator nutzbar.

### Umfang

- Tabs
- horizontale und vertikale Splits
- aktives Pane und Fokuswechsel
- Pane-Größe ändern
- Suche im Scrollback
- anklickbare Links
- Schrift- und Theme-Einstellungen
- zentrale Tastenkürzel
- Vollbild

### Abschlusskriterien

- mehrere Stunden stabil nutzbar
- Pi Agent funktioniert zuverlässig
- mindestens Codex CLI oder Claude Code funktioniert zuverlässig
- lange Ausgaben frieren die Oberfläche nicht ein
- Tabs und Splits sind vollständig per Tastatur bedienbar

## Phase 2 – Startprofile und Layouts

### Ziel

Häufige Arbeitsumgebungen lassen sich schnell und reproduzierbar öffnen.

### Umfang

- zuletzt verwendete Verzeichnisse
- Startprofile
- optionale Shell und Startbefehle
- gespeicherte Tab- und Split-Layouts
- Wiederherstellung des letzten Layouts
- robuste Konfigurationsvalidierung

### Nicht enthalten

- Datei-Explorer
- Projektanalyse
- Build-System-Erkennung
- Git-Projektverwaltung

### Abschlusskriterien

- ein häufiges Arbeitsverzeichnis ist mit höchstens zwei Aktionen erreichbar
- gelöschte oder ungültige Pfade werden verständlich behandelt
- beschädigte Konfiguration verhindert keinen Programmstart
- das Wiederherstellen eines Layouts beeinflusst keine bereits laufende Sitzung

## Phase 3 – KI-Assistent

### Ziel

Optionale KI-Hilfe verbessert konkrete Terminalaufgaben, ohne den normalen Terminalbetrieb zu verändern.

### Umfang

- genau ein OpenAI-kompatibler Provider
- API-Key im System-Keyring
- ausgewählten Terminaltext erklären
- letzte bewusst ausgewählte Ausgabe analysieren
- Fehlermeldungen erklären
- Shell-Befehle erzeugen und verbessern
- Streaming und Abbruch
- Kontextvorschau vor dem Senden
- Vorschlag kopieren oder in die Eingabe übernehmen

### Verbindliche Grenzen

- keine automatische Ausführung
- kein simuliertes Enter
- keine dauerhafte Überwachung
- keine Tool-Aufrufe des Modells
- kein MCP
- keine Agenten-Orchestrierung
- keine mehreren Provider

### Abschlusskriterien

- Terminal bleibt während KI-Anfragen flüssig
- Abbruch beendet Requests zuverlässig
- ohne Netzwerk und Provider bleibt das Terminal vollständig nutzbar
- Secret-Dateien werden nicht automatisch als Kontext übernommen
- API-Schlüssel erscheinen weder in Konfiguration noch Logs

## Phase 4 – Stabilisierung

### Ziel

IV ist robust genug für regelmäßige private Nutzung.

### Umfang

- Integrationstests
- manuelle Testmatrix
- Speicher- und CPU-Messungen
- Fehlerbehandlung
- Logging ohne sensible Daten
- Barrierefreiheitsprüfung
- Paketierung für die Zielumgebung

### Abschlusskriterien

- keine bekannten kritischen Fehler
- keine reproduzierbaren Prozesslecks
- KI- und Netzwerkfehler beeinträchtigen das Terminal nicht
- alle verbindlichen Tests aus `docs/TEST_STRATEGY.md` wurden ausgeführt
- die Definition of Done ist erfüllt

## Nach dem MVP

Mögliche spätere Themen werden erst nach dem MVP bewertet:

- alternatives Terminal-Backend
- zusätzliche Provider
- erweiterte Sitzungswiederherstellung
- weitere Linux-Desktop-Integrationen

Diese Punkte sind keine Zusagen. Sie benötigen jeweils eine Architekturentscheidung und einen nachgewiesenen Nutzen für Terminalarbeit.