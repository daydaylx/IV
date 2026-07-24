# Architektur des IV-MVP

## Dokumentstatus

Dieses Dokument beschreibt das akzeptierte Architekturziel für das MVP. Es ist keine Behauptung, dass alle genannten Module und Typen bereits implementiert sind.

Der tatsächliche Stand steht in [`PROJECT_STATE.md`](PROJECT_STATE.md). Abweichende grundlegende Entscheidungen werden als ADR unter [`decisions/`](decisions/) dokumentiert.

## Ziel

Die Architektur schützt den Terminalbetrieb vor UI-, Netzwerk- und KI-Komplexität. Sie ist bewusst klein und darf erst erweitert werden, wenn ein konkreter MVP-Anwendungsfall dies verlangt.

## Vorgesehene Module

```text
src/
├── app/            # Start, Lebenszyklus, Zusammensetzung
├── ui/             # GTK4/libadwaita-Ansichten und Eingabeaktionen
├── terminal/       # Backend-Schnittstelle, VTE-Adapter, Terminalzustand
├── workspace/      # Startprofile, Arbeitsverzeichnisse, Layoutdaten
├── ai/             # Kontext, Streaming, Vorschläge; optional
├── settings/       # TOML, Defaults, Validierung
├── security/       # Keyring, Secret-Filter, sichere Logs
└── platform/       # Linux-/Desktop-Integration
```

Diese Struktur ist ein Zielbild. Für den technischen Prototyp darf sie kleiner sein. Module dürfen aber nicht willkürlich vermischt werden.

## Abhängigkeitsrichtung

```text
UI ───────────────┐
                  v
Anwendungslogik -> Domänenzustand
                  ^
Terminal/AI/Storage/Platform-Adapter
```

- Domänenzustand kennt keine GTK- oder VTE-Typen.
- UI löst Aktionen aus und rendert Zustand; sie besitzt nicht die Terminallogik.
- VTE wird ausschließlich im Terminaladapter verwendet.
- KI darf Terminalzustand nur über explizit freigegebenen Kontext lesen.

## Kernmodelle

### TerminalPane

- ID
- Startverzeichnis
- aktive Shell bzw. Startbefehl
- Prozessstatus
- sichtbarer Titel
- Backend-Handle nur innerhalb der Terminal-Infrastruktur

### Tab

- ID
- Titel
- Wurzel der Split-Struktur
- aktiver Pane

### SplitNode

```text
Pane(id)
Split {
  orientation,
  ratio,
  first,
  second
}
```

### StartProfile

- Name
- Startverzeichnis
- optionale Shell
- optionaler Startbefehl
- optionales Layout
- optionales bevorzugtes KI-Modell

### AiContext

- explizit ausgewählte Terminalzeilen
- Herkunft und Umfang
- maskierte sensible Werte
- sichtbare Vorschau vor Versand

Die konkreten Rust-Typen werden erst mit der jeweiligen Implementierung verbindlich. Namen in diesem Dokument sind fachliche Orientierung, keine unveränderliche API-Vorgabe.

## TerminalBackend

Die Schnittstelle muss nur tatsächlich benötigte Operationen anbieten:

- Terminal-Widget bzw. Darstellungsobjekt erzeugen
- Prozess starten
- Eingabe einfügen, aber nicht künstlich bestätigen
- Arbeitsverzeichnis soweit zuverlässig ermitteln
- Resize und Fokus behandeln
- Auswahl und sichtbaren Text abrufen
- Prozessende und Exit-Code melden
- Ressourcen kontrolliert freigeben

Keine abstrakte Universal-API entwerfen. VTE-spezifische Fähigkeiten erst abstrahieren, wenn sie im MVP verwendet werden.

## Nebenläufigkeit

- GTK-Hauptthread: ausschließlich UI-nahe Arbeit.
- KI-Streaming, Dateizugriffe und andere blockierende Aufgaben asynchron.
- Ergebnisse kontrolliert in den UI-Kontext zurückreichen.
- Abbruch und Fenster-/Pane-Schließung müssen laufende Aufgaben beenden oder sicher entkoppeln.
- Keine blockierenden Netzwerk- oder Prozessabfragen in GTK-Callbacks.

## Persistenz

Im MVP bevorzugt:

- TOML für Einstellungen und kleine Startprofile
- keine Datenbank, solange strukturierte Dateien ausreichen
- Keyring für API-Schlüssel
- Layoutzustand versionieren, damit ungültige ältere Daten sauber verworfen werden können

## Fehlergrenzen

- Fehler eines KI-Providers dürfen niemals das Terminal schließen.
- Fehler eines einzelnen Panes dürfen andere Tabs möglichst nicht beenden.
- Ungültige Konfiguration fällt auf sichere Defaults zurück.
- Fehler werden im UI verständlich und in Logs technisch, aber ohne Secrets dargestellt.

## Verbotene Kopplungen

- KI-Modul schreibt direkt in PTY oder simuliert Enter.
- UI liest oder verändert VTE-Interna außerhalb des Terminalmoduls.
- Startprofile enthalten IDE-Metadaten, Dateianalyse oder Build-Konfiguration.
- Prozessstatus behauptet nicht erkennbare Modellzustände.
- Terminalrendering wartet auf Netzwerk, Speicherung oder KI.

## Architekturänderungen

Eine Änderung an diesen Grenzen benötigt eine ADR unter `docs/decisions/` mit:

- Problem und Kontext
- gewählter Lösung
- verworfenen Alternativen
- Auswirkungen und Risiken
- Verifikation
- Rückbau- oder Migrationsoption

Ablauf: [`decisions/README.md`](decisions/README.md).
