# Zustandsmodell

## Zweck

Dieses Dokument beschreibt den fachlichen Zustand von IV. GTK-Widgets stellen diesen Zustand dar, sind aber nicht selbst die einzige Quelle der Wahrheit.

## Grundstruktur

```text
ApplicationState
└── WindowState[]
    └── TabState[]
        └── PaneTree
            ├── SplitNode
            └── TerminalPane
```

## Identitäten

Langlebige Einheiten besitzen stabile, typisierte IDs:

- `WindowId`
- `TabId`
- `PaneId`
- `ProfileId`
- `RequestId`

IDs werden nicht aus Positionen, sichtbaren Titeln oder GTK-Widget-Adressen abgeleitet.

## WindowState

Enthält:

- geordnete Tabs
- aktiven Tab
- Vollbildstatus
- Sichtbarkeit von KI- und Statusleiste

Invarianten:

1. Ein offenes Fenster enthält mindestens einen Tab.
2. Der aktive Tab existiert in der Tab-Liste.
3. Das Schließen des letzten Tabs folgt einer ausdrücklich definierten Benutzerregel.

## TabState

Enthält:

- stabile ID
- sichtbaren Titel
- Wurzel des Pane-Baums
- aktives Pane
- ursprüngliches Startverzeichnis
- optionales Startprofil

Invarianten:

1. Ein Tab enthält mindestens ein Terminal-Pane.
2. Das aktive Pane gehört zum Pane-Baum.
3. Titeländerungen verändern weder IDs noch Pfade.

## PaneTree

```text
PaneNode = Terminal(TerminalPaneState)
         | Split(SplitState)
```

Ein Split besitzt Orientierung, zwei Kinder und ein Teilungsverhältnis.

Regeln:

- kein Kind darf vollständig unsichtbar werden
- leere Split-Knoten sind unzulässig
- nach dem Schließen eines Kindes wird das verbleibende Kind hochgezogen
- verschachtelte Splits sind erlaubt

## TerminalPaneState

Enthält nur anwendungsrelevanten Zustand:

- `PaneId`
- Startverzeichnis
- bestmöglich erkanntes aktuelles Arbeitsverzeichnis
- Shell oder Startbefehl
- Prozessstatus
- letzter Exit-Code
- Zeitpunkt der letzten Ausgabe
- Auswahl- und Suchstatus
- optional zugeordnete KI-Anfrage

Das VTE-Widget selbst gehört nicht in dieses Zustandsmodell.

## Prozesszustand

```text
NotStarted
Starting
Running { pid }
Exited { code, timestamp }
FailedToStart { reason }
Closing
```

Regeln:

- Übergänge sind explizit
- `Running` setzt einen bestätigten Kindprozess voraus
- ein Exit-Code wird erst nach tatsächlichem Prozessende gesetzt
- Prozessende und Entfernen des Panes sind getrennte Vorgänge

## Fokusmodell

Der fachlich aktive Fokus wird getrennt vom GTK-Fokus geführt.

Nach Strukturänderungen gilt:

- ein neu erstelltes Pane wird aktiv
- nach dem Schließen wird ein existierendes Nachbar-Pane aktiv
- nach dem Schließen eines Tabs wird ein benachbarter Tab aktiv

## Arbeitsverzeichnisse

Es werden unterschieden:

- `initial_working_directory`
- `current_working_directory`

Das aktuelle Verzeichnis darf unbekannt sein. In diesem Fall wird kein veralteter Pfad als sicher aktuell dargestellt.

Ein neues Pane im aktuellen Verzeichnis verwendet den bestätigten aktuellen Pfad. Fehlt dieser, wird das ursprüngliche Startverzeichnis verwendet.

## KI-Zustand

```text
Idle
PreparingContext
Requesting { request_id }
Streaming { request_id }
Completed { request_id }
Cancelled { request_id }
Failed { request_id, reason }
```

KI-Zustand und Terminalprozesszustand bleiben getrennt.

Regeln:

- KI-Abbruch beeinflusst den Shell-Prozess nicht
- beim Schließen eines Panes wird dessen Anfrage abgebrochen
- Kontext ist ein unveränderlicher Snapshot der einzelnen Anfrage
- Antworten werden niemals automatisch ausgeführt

## Aktionen und Ereignisse

Beispiele:

- `CreateTab`
- `CloseTab`
- `SplitPane`
- `ClosePane`
- `ActivatePane`
- `ResizeSplit`
- `ProcessStarted`
- `ProcessExited`
- `WorkingDirectoryChanged`
- `StartAiRequest`
- `CancelAiRequest`

GTK-Callbacks lösen Aktionen aus, verändern aber nicht beliebig einzelne Zustandsfelder.

## Persistenz

Gespeichert werden dürfen:

- Einstellungen
- Startprofile
- Layoutstruktur
- Startverzeichnisse
- benutzerdefinierte Titel

Nicht gespeichert werden:

- Prozess-IDs als wiederherstellbare Sitzungen
- Terminalbuffer im MVP
- API-Schlüssel
- Terminalausgaben oder KI-Prompts ohne spätere ausdrückliche Entscheidung

## Fehlerfälle

Bei inkonsistentem Zustand:

1. keine Panik im UI-Thread
2. betroffene Aktion abbrechen
3. bestehende Terminals weiterlaufen lassen
4. ohne sensible Daten protokollieren
5. auf einen gültigen Zustand zurückfallen, wenn möglich

## Testbare Invarianten

- Splitten erhöht die Zahl der Terminalblätter um eins
- Pane-Schließen reduziert sie um eins
- nach jeder Strukturänderung existiert das aktive Pane
- keine Pane-ID erscheint doppelt
- Teilungsverhältnisse bleiben gültig
- KI-Zustandsänderungen verändern den Prozesszustand nicht
- Prozessende entfernt das Pane nicht automatisch