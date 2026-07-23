# Terminal-Backend

## Ziel

Das Terminal-Backend kapselt VTE vollständig. UI, Tabs, Splits, Profile und KI dürfen nicht direkt von VTE-Typen abhängen.

## Verantwortlichkeiten

Das Backend ist zuständig für:

- Shell oder Startbefehl starten
- PTY- und Kindprozess-Lebenszyklus
- Eingaben an das Terminal senden
- Größenänderungen weitergeben
- Auswahl und Suchoperationen
- Scrollback-bezogene Funktionen
- Prozessende und Exit-Code melden
- bekannte Arbeitsverzeichnisänderungen melden
- sichtbaren oder markierten Text kontrolliert bereitstellen
- kontrolliertes Beenden

Nicht zuständig für:

- Tabs und Splits
- GTK-Navigation
- Einstellungen laden oder speichern
- Startprofile
- KI-Anfragen
- Kontextfilterung für Provider
- globale Tastenkürzel

## Schnittstelle

Die konkrete Rust-API wird während Phase 0 festgelegt. Fachlich muss sie mindestens folgende Operationen abbilden:

```text
spawn(config) -> TerminalHandle
send_input(handle, bytes)
resize(handle, columns, rows)
copy_selection(handle)
paste_clipboard(handle)
search(handle, query, direction)
selected_text(handle) -> Optional<TextSnapshot>
visible_text(handle) -> TextSnapshot
current_working_directory(handle) -> Optional<Path>
request_close(handle, policy)
```

Ereignisse:

```text
ProcessStarted
OutputObserved
WorkingDirectoryChanged
SelectionChanged
TitleChanged
ProcessExited
SpawnFailed
BackendError
```

## TerminalHandle

Ein Handle identifiziert genau eine laufende oder bereits beendete Terminalinstanz.

Regeln:

- Handles sind nicht mit Pane-Positionen identisch
- nach Prozessende darf der Exit-Status weiterhin gelesen werden
- ein ungültiger Handle liefert einen typisierten Fehler
- GTK-Widgets werden nicht als öffentliche Handle-Typen verwendet

## Startkonfiguration

Eine Startkonfiguration enthält höchstens:

- ausführbare Shell oder Programm
- Argumente
- Startverzeichnis
- Umgebungsvariablen
- Terminalgröße
- optionale Startprofil-Metadaten außerhalb des Backends

Umgebungsvariablen werden bewusst zusammengestellt. Secrets werden nicht ohne Grund ergänzt oder protokolliert.

## Prozesslebenszyklus

1. Backend validiert die Startkonfiguration.
2. PTY und Kindprozess werden gestartet.
3. `ProcessStarted` wird erst nach bestätigtem Start gemeldet.
4. Ausgabeereignisse dürfen die UI nicht blockieren.
5. Prozessende erzeugt genau ein `ProcessExited`-Ereignis.
6. Ressourcen werden unabhängig vom sichtbaren Pane freigegeben.

Beim Schließen gilt eine klar definierte Policy, beispielsweise:

- laufenden Prozess höflich beenden
- nach Frist eskalieren
- Benutzer bei erkennbar laufender Aufgabe warnen

Die konkrete Policy gehört in eine spätere Entscheidung; stilles hartes Beenden ist nicht der Standard.

## Nebenläufigkeit

- GTK- und VTE-Aufrufe erfolgen nur im dafür vorgesehenen Hauptkontext.
- Netzwerk- und KI-Arbeit gehört nie in das Backend.
- Ereignisse werden über klar definierte Kanäle oder Callbacks an die Anwendung gemeldet.
- Hochfrequente Ausgabe darf nicht für jedes Byte eine teure UI-Aktualisierung auslösen.
- Abmeldung und Schließen müssen gegen verspätete Ereignisse abgesichert sein.

## Textzugriff

Terminaltext wird nur auf ausdrückliche Aktion der Anwendung ausgelesen.

`TextSnapshot` enthält:

- Text
- Herkunft: Auswahl, sichtbarer Bereich oder ausdrücklich angeforderter Ausschnitt
- Zeitpunkt
- optionale Pane-ID außerhalb des Backendkerns
- Hinweis, ob der Text gekürzt wurde

Das Backend entscheidet nicht, ob Text an einen KI-Provider gesendet werden darf.

## Arbeitsverzeichnis

Die Erkennung ist bestmöglich und kann fehlschlagen.

- unbekannt ist ein gültiger Zustand
- keine heuristische Vermutung wird als sicherer Pfad ausgegeben
- Pfade werden vor Nutzung validiert
- das Backend meldet Änderungen, ohne Startprofile selbst anzupassen

## VTE-Implementierung

`VteBackend` ist die einzige Schicht, die VTE-spezifische Typen und Signale kennen darf.

Erlaubte Abhängigkeit:

```text
UI/Application -> TerminalBackend trait -> VteBackend -> VTE
```

Nicht erlaubt:

```text
UI -> VTE
Profiles -> VTE
AI -> VTE
State model -> VTE widget
```

## Fehler

Backendfehler sind typisiert, etwa:

- ungültige Startkonfiguration
- Startverzeichnis nicht verfügbar
- Prozess konnte nicht gestartet werden
- Operation auf geschlossenem Handle
- Zwischenablage nicht verfügbar
- Suche fehlgeschlagen
- interne Backendinkonsistenz

Fehlertexte enthalten keine vollständigen Umgebungsvariablen oder Terminalinhalte.

## Austauschbarkeit

Die Abstraktion soll einen späteren Backendwechsel ermöglichen, aber keine hypothetische Multi-Backend-Plattform vorwegnehmen.

Für das MVP gilt:

- genau eine produktive Implementierung: VTE
- keine Laufzeitwahl zwischen Backends
- keine Funktionen, die nur für ein unbekanntes zukünftiges Backend existieren

## Tests

Automatisiert testbar sind mindestens:

- Startkonfiguration und Validierung
- Zustandsübergänge des Handles
- genau ein Exit-Ereignis
- Verhalten nach geschlossenem Handle
- Event-Entkopplung vom Anwendungszustand

Zusätzlich werden echte VTE-/PTY-Szenarien gemäß `docs/TEST_STRATEGY.md` manuell und als Integrationstest geprüft.