# Roadmap

## Dokumentstatus

Dieses Dokument beschreibt die geplante Entwicklungsreihenfolge. Es belegt nicht, dass ein Punkt bereits implementiert ist.

Der tatsächlich aktuelle Stand steht ausschließlich in [`PROJECT_STATE.md`](PROJECT_STATE.md).

## Zweck

Die Roadmap schützt IV vor Scope Creep und verhindert, dass KI-, Profil- oder Komfortfunktionen vor einem stabilen Terminalkern entstehen.

## Grundregeln

- Eine Phase beginnt erst, wenn die Abschlusskriterien der vorherigen Phase erfüllt sind.
- Offene Punkte werden dokumentiert, aber nicht durch zusätzliche Architektur umgangen.
- Detaillierte Fachregeln stehen in den zuständigen Dokumenten aus [`INDEX.md`](INDEX.md).
- Änderungen der Reihenfolge benötigen eine nachvollziehbare Begründung und eine Aktualisierung von `PROJECT_STATE.md`.

## Phase 0 – Technischer Prototyp

### Ziel

Eine minimale GTK4/libadwaita-Anwendung bettet VTE zuverlässig ein und startet eine lokale Shell.

### Umfang

- Rust-Projektstruktur
- GTK4- und libadwaita-Anwendung
- ein Fenster
- ein VTE-Terminal
- Standardshell starten
- Eingabe und Ausgabe
- Copy und Paste
- Resize
- kontrolliertes Schließen
- minimale verständliche Fehlerausgabe

### Nicht enthalten

- Tabs und Splits
- Einstellungen
- Profile
- KI
- Sitzungswiederherstellung
- alternatives Terminal-Backend

### Abschlusskriterien

- bash und zsh starten zuverlässig
- `vim` oder `nvim`, `less` und `htop` sind nutzbar
- Unicode und breite Zeichen werden korrekt dargestellt
- Fenstergrößenänderungen beschädigen die Darstellung nicht
- beim Schließen bleiben keine Zombie-Prozesse zurück
- Fehler beim Start einer Shell beenden die Anwendung nicht unkontrolliert

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
- Prozess- und Pane-Lebenszyklen hinterlassen keine Leaks oder Zombie-Prozesse

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
- Wiederaufnahme laufender Prozesse

### Abschlusskriterien

- ein häufiges Arbeitsverzeichnis ist mit höchstens zwei Aktionen erreichbar
- gelöschte oder ungültige Pfade werden verständlich behandelt
- beschädigte Konfiguration verhindert keinen Programmstart
- Layoutdaten sind versioniert und validierbar
- das Wiederherstellen eines Layouts beeinflusst keine bereits laufende Sitzung

## Phase 3 – KI-Assistent

### Ziel

Optionale KI-Hilfe verbessert konkrete Terminalaufgaben, ohne den normalen Terminalbetrieb zu verändern.

### Umfang

- genau ein OpenAI-kompatibler Provider
- API-Key im System-Keyring
- bewusst ausgewählten Terminaltext erklären
- Fehlermeldungen analysieren
- Shell-Befehle erzeugen, verbessern und erklären
- Streaming und Abbruch
- Kontextvorschau vor dem Senden
- Vorschlag kopieren oder in die Eingabe übernehmen

Die verbindlichen Grenzen und technischen Details stehen in [`AI_INTEGRATION.md`](AI_INTEGRATION.md), [`SECURITY.md`](SECURITY.md) und [`LOGGING.md`](LOGGING.md).

### Abschlusskriterien

- Terminal bleibt während KI-Anfragen flüssig
- Abbruch beendet Requests zuverlässig
- ohne Netzwerk und Provider bleibt das Terminal vollständig nutzbar
- Kontext wird nur bewusst ausgewählt und vor Versand angezeigt
- bekannte Secret-Dateien werden nicht automatisch übernommen
- API-Schlüssel erscheinen weder in Konfiguration noch Logs
- kein Vorschlag wird automatisch ausgeführt oder mit Enter bestätigt

## Phase 4 – Stabilisierung

### Ziel

IV ist robust genug für regelmäßige private Nutzung.

### Umfang

- Integrationstests
- reproduzierbare manuelle Testmatrix
- Speicher-, CPU- und Startzeitmessungen
- Fehlerbehandlung und sichere Logs
- Barrierefreiheitsprüfung
- Paketierung für die Zielumgebung
- Aktualisierung der tatsächlichen Einschränkungen

### Abschlusskriterien

- keine bekannten kritischen Fehler
- keine reproduzierbaren Prozesslecks
- KI- und Netzwerkfehler beeinträchtigen das Terminal nicht
- alle verbindlichen Tests aus `TEST_STRATEGY.md` wurden ausgeführt
- Performanceaussagen sind mit Messungen belegt
- `DEFINITION_OF_DONE.md` und `RELEASE_CHECKLIST.md` sind erfüllt

## Nach dem MVP

Mögliche spätere Themen werden erst nach dem MVP bewertet:

- alternatives Terminal-Backend
- zusätzliche Provider
- erweiterte Sitzungswiederherstellung
- weitere Linux-Desktop-Integrationen

Diese Punkte sind keine Zusagen. Sie benötigen jeweils eine Produkt- oder Architekturentscheidung und einen nachgewiesenen Nutzen für Terminalarbeit.
