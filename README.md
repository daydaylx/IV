# IV – Leichtgewichtiger KI-Terminal-Emulator

## Projektziel

IV ist ein nativer, leichtgewichtiger Terminal-Emulator für Linux mit moderner, reduzierter Oberfläche und optionaler KI-Unterstützung.

Das Terminal bleibt jederzeit der Mittelpunkt der Anwendung. KI, Sitzungen und Arbeitsverzeichnisse sind ergänzende Funktionen und dürfen den normalen Terminalbetrieb weder verlangsamen noch komplizierter machen.

Das Projekt ist zunächst ausschließlich für die private Nutzung vorgesehen.

---

## Klare Abgrenzung

IV ist:

- ein Terminal-Emulator
- eine native Linux-Anwendung
- schnell und tastaturorientiert
- für klassische Shell-Arbeit geeignet
- für Pi Agent, Codex CLI, Claude Code, SSH, tmux und zellij geeignet
- um kontrollierte KI-Hilfen für Terminalaufgaben ergänzt

IV ist ausdrücklich keine IDE.

Nicht Bestandteil des Projekts sind:

- integrierter Codeeditor
- Datei-Explorer im IDE-Stil
- Language Server
- Debugger
- visuelles Git-Frontend
- Aufgaben- oder Projektmanagement
- Plugin-Marktplatz
- eigener Coding-Agent
- autonome Befehlsausführung
- Cloudkonto oder Synchronisierung
- Teamfunktionen

Leitregel:

> Alles, was nicht direkt Terminalarbeit verbessert, gehört nicht in das MVP.

---

## Technische Grundentscheidung

### Plattform

- Linux zuerst
- Wayland als primäre Anzeigeumgebung
- X11 nur ohne größeren Zusatzaufwand

### Technologie

- Rust
- GTK4
- libadwaita
- VTE als Terminal-Widget
- Tokio oder vergleichbare asynchrone Ausführung für Netzwerk- und Hintergrundaufgaben
- TOML für Einstellungen
- System-Keyring für API-Schlüssel

### Terminal-Backend

Das MVP verwendet VTE. Der Zugriff wird intern gekapselt, damit die restliche Anwendung nicht direkt von VTE abhängt.

```text
TerminalBackend
└── VteBackend
```

Ein späterer Wechsel auf libghostty oder ein anderes Backend ist möglich, aber nicht Bestandteil des MVP.

---

# MVP-Umfang

## 1. Terminal-Grundfunktionen

Das MVP muss enthalten:

- lokale Standard-Shell starten
- Eingabe und Ausgabe
- zuverlässige PTY-Prozessverwaltung
- Scrollback
- Copy und Paste
- Textauswahl
- Suche im Scrollback
- anklickbare Links
- Unicode und breite Zeichen
- Größenänderung ohne Darstellungsfehler
- Vollbildmodus
- konfigurierbare Schriftart und Schriftgröße
- Hell-, Dunkel- und System-Theme
- kontrollierter Prozessabschluss

## 2. Tabs und Splits

Das MVP unterstützt:

- mehrere Tabs
- horizontale Splits
- vertikale Splits
- aktiven Pane wechseln
- Pane-Größe ändern
- Pane schließen
- neuen Tab öffnen
- neues Pane im aktuellen Arbeitsverzeichnis öffnen
- neues Pane im ursprünglichen Startverzeichnis öffnen
- Tabs umbenennen

Tabs und Splits müssen vollständig per Tastatur bedienbar sein.

## 3. Arbeitsverzeichnisse

IV verwendet bewusst keine umfangreiche Projektverwaltung.

Stattdessen gibt es einfache Arbeitsverzeichnisse beziehungsweise Startprofile.

Ein Startprofil enthält nur:

- Name
- Startverzeichnis
- optionale Shell
- optionaler Startbefehl
- optionales Tab- und Split-Layout
- optional bevorzugtes KI-Modell

Funktionen:

- häufig verwendete Verzeichnisse speichern
- zuletzt verwendete Verzeichnisse anzeigen
- Startprofil öffnen
- Layout speichern
- letztes Layout wiederherstellen

Nicht enthalten:

- Datei-Explorer
- Projektanalyse
- Build-System-Erkennung
- IDE-Projektmodelle
- integrierte Git-Verwaltung

## 4. KI-Unterstützung

Die KI ist optional und wird nur bei Bedarf eingeblendet.

Das MVP unterstützt genau einen OpenAI-kompatiblen Provider.

Funktionen:

- markierten Terminaltext erklären
- letzte Terminalausgabe analysieren
- Fehlermeldung erklären
- Shell-Befehl erzeugen
- vorhandenen Shell-Befehl verbessern
- komplexen Befehl erklären
- Antwort streamen
- laufende Anfrage abbrechen
- vorgeschlagenen Befehl kopieren
- vorgeschlagenen Befehl in die Terminaleingabe übernehmen

Nicht enthalten:

- automatische Befehlsausführung
- Tool-Aufrufe durch das Modell
- MCP
- autonomer Agent
- automatische Analyse jeder Terminalausgabe
- dauerhafte Überwachung des Terminals
- mehrere Provider im MVP

## 5. Prozessstatus

Die Anwendung darf nur technisch erkennbare Zustände anzeigen.

Mögliche Informationen:

- laufender Hauptprozess
- Prozess beendet
- Exit-Code
- Zeitpunkt der letzten Ausgabe
- möglicherweise wartende Eingabe

Nicht angezeigt werden angebliche interne Modellzustände wie:

- denkt
- plant
- versteht
- hängt sicher

Ein möglicher Stillstand darf nur neutral formuliert werden, zum Beispiel:

> Seit fünf Minuten keine neue Ausgabe.

## 6. Oberfläche

Die Oberfläche bleibt reduziert.

```text
┌──────────────────────────────────────────────────────────┐
│ Tab 1   Tab 2   +                         KI ▸   Menü    │
├──────────────────────────────────────────────────────────┤
│                                                          │
│                         Terminal                         │
│                                                          │
├──────────────────────────────────────────────────────────┤
│ ~/Projekt   │ Prozess: pi   │ letzte Ausgabe: gerade    │
└──────────────────────────────────────────────────────────┘
```

Die KI-Seitenleiste erscheint nur bei Bedarf:

```text
┌───────────────────────────────────┬──────────────────────┐
│                                   │ KI-Assistent         │
│             Terminal              │                      │
│                                   │ Analyse              │
│                                   │ Befehlsvorschlag     │
│                                   │ Kontext              │
└───────────────────────────────────┴──────────────────────┘
```

UI-Regeln:

- Terminal erhält standardmäßig den gesamten verfügbaren Platz
- KI-Seitenleiste ist einklappbar
- Statusleiste ist reduzierbar
- keine permanenten Chatblasen im Terminal
- keine Blockdarstellung für jeden Shell-Befehl
- keine IDE-typischen Seitenleisten
- keine überladenen Animationen
- Tastaturbedienung hat Vorrang

---

# Sicherheitsregeln

Verbindliche Regeln:

1. Kein KI-generierter Befehl wird automatisch ausgeführt.
2. Vorschläge werden nur in die Eingabe übernommen.
3. Der Nutzer bestätigt die Ausführung selbst mit Enter.
4. Terminalkontext wird nur nach bewusster Auswahl gesendet.
5. Vor dem Absenden ist sichtbar, welcher Kontext übertragen wird.
6. API-Schlüssel werden ausschließlich im System-Keyring gespeichert.
7. `.env`, private Schlüssel und bekannte Secret-Dateien sind standardmäßig ausgeschlossen.
8. KI-Anfragen laufen unabhängig vom Terminal-Rendering.
9. Netzwerkfehler dürfen den Terminalbetrieb nicht beeinträchtigen.
10. Es gibt keine Telemetrie und keine Cloudpflicht.

---

# Nicht Bestandteil des MVP

- libghostty-Integration
- eigener Terminalparser
- eigener GPU-Renderer
- Windows- oder macOS-Unterstützung
- Plugin-System
- Marketplace
- integrierter Editor
- Datei-Explorer
- Language Server
- Debugger
- komplexe Git-Funktionen
- Agenten-Orchestrierung
- Subagenten
- MCP
- Thinking-Anzeige
- vollständige Wiederherstellung laufender Prozesse
- Cloud-Synchronisierung
- Benutzerkonten

---

# Empfohlene Entwicklungsphasen

## Phase 0 – Technischer Prototyp

Ziel: VTE zuverlässig in eine minimale GTK4-Anwendung einbetten.

Umfang:

- GTK4-Fenster
- ein Terminal
- Standardshell starten
- Eingabe und Ausgabe
- Resize
- Copy und Paste
- sauberer Prozessabschluss

Abschlusskriterien:

- zsh oder bash läuft stabil
- Unicode funktioniert
- vim, htop und less funktionieren
- keine Zombie-Prozesse nach dem Schließen

## Phase 1 – Terminal-MVP

Umfang:

- Tabs
- Splits
- Suche
- Links
- Tastenkürzel
- Schrift- und Theme-Einstellungen

Abschlusskriterien:

- mehrere Stunden stabil nutzbar
- Pi Agent läuft zuverlässig
- Codex CLI oder Claude Code läuft zuverlässig
- kein Einfrieren bei längerer Ausgabe

## Phase 2 – Startprofile und Layouts

Umfang:

- Arbeitsverzeichnisse speichern
- Startprofile
- letzte Verzeichnisse
- Tab- und Split-Layout speichern
- Layout wiederherstellen

Abschlusskriterien:

- häufige Arbeitsverzeichnisse sind mit maximal zwei Aktionen erreichbar
- ungültige oder gelöschte Pfade werden sauber behandelt
- gespeicherte Layouts beschädigen keine laufenden Sitzungen

## Phase 3 – KI-Assistent

Umfang:

- ein OpenAI-kompatibler Provider
- Streaming
- Anfrage abbrechen
- ausgewählten Terminaltext analysieren
- Fehler erklären
- Befehle vorschlagen
- Vorschläge in die Eingabe übernehmen
- API-Key im System-Keyring

Abschlusskriterien:

- keine automatische Ausführung
- Kontext ist vor dem Senden sichtbar
- Terminal bleibt während der Anfrage flüssig
- Abbruch beendet die Anfrage zuverlässig
- API-Schlüssel stehen in keiner Klartextdatei

## Phase 4 – Stabilisierung

Umfang:

- Fehlerbehandlung
- Performance-Messung
- Speicherverbrauch prüfen
- Konfigurationsvalidierung
- Logging ohne sensible Daten
- Integrationstests

Abschlusskriterien:

- kein bekannter kritischer Fehler
- fehlerhafte Einstellungen verhindern keinen Programmstart
- Logs enthalten keine API-Schlüssel
- Terminalbetrieb bleibt unabhängig von KI-Fehlern stabil

---

# MVP-Abschlusskriterien

Das MVP ist abgeschlossen, wenn:

1. IV unter Linux zuverlässig startet.
2. Shell, SSH und interaktive Terminalprogramme stabil funktionieren.
3. Tabs und Splits zuverlässig bedienbar sind.
4. Pi Agent und mindestens ein weiterer Coding-Agent funktionieren.
5. Arbeitsverzeichnisse und einfache Layouts gespeichert werden können.
6. Terminaltext bewusst als KI-Kontext gewählt werden kann.
7. KI-Antworten gestreamt und abgebrochen werden können.
8. Befehlsvorschläge niemals automatisch ausgeführt werden.
9. Die Anwendung während KI-Anfragen flüssig bleibt.
10. API-Schlüssel sicher im System-Keyring gespeichert sind.
11. Keine Telemetrie, Anmeldung oder Cloudpflicht besteht.
12. Die Anwendung in mehreren echten Arbeitssitzungen freiwillig statt Kitty verwendet wurde.

---

# Erfolgskriterium

Das Projekt ist erfolgreich, wenn IV ein schnelles, angenehm bedienbares Terminal bleibt und die KI-Unterstützung konkrete Terminalaufgaben verbessert, ohne daraus eine IDE oder einen autonomen Agenten zu machen.
