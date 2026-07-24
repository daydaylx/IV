# Terminal-Backend

## Dokumentstatus

Dieses Dokument beschreibt den geplanten fachlichen Vertrag des MVP-Terminal-Backends. Die dargestellten Operationen und Ereignisnamen sind konzeptuell und keine bereits verbindliche Rust-API.

Die tatsächliche Schnittstelle wird in Phase 0 gegen `gtk4-rs`, VTE und den realen Prozesslebenszyklus geprüft. Der Implementierungsstand steht in [`PROJECT_STATE.md`](PROJECT_STATE.md).

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

## Konzeptuelle Schnittstelle

> Die folgende Darstellung beschreibt benötigte Fähigkeiten. Namen, Signaturen und Rückgabetypen dürfen bei der Implementierung angepasst werden, solange Verantwortlichkeiten und Sicherheitsgrenzen erhalten bleiben.

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

Konzeptuelle Ereignisse:

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

Eine Operation wird erst Teil der produktiven Schnittstelle, wenn ein konkreter MVP-Anwendungsfall und eine passende Verifikation existieren.

## TerminalHandle

Ein Handle identifiziert genau eine laufende oder bereits beendete Terminalinstanz.

Regeln:

- Handles sind nicht mit Pane-Positionen identisch.
- Nach Prozessende darf der Exit-Status weiterhin gelesen werden.
- Ein ungültiger Handle liefert einen typisierten Fehler.
- GTK-Widgets werden nicht als öffentliche fachliche Handle-Typen verwendet.

## Startkonfiguration

Eine Startkonfiguration enthält höchstens:

- ausführbare Shell oder Programm
- Argumente
- Startverzeichnis
- bewusst zusammengestellte Umgebungsvariablen
- Terminalgröße

Startprofil-Metadaten gehören außerhalb des Backends.

Umgebungsvariablen werden nicht vollständig protokolliert. Zusätzliche Secrets werden nur übergeben, wenn ein konkreter gestarteter Prozess sie benötigt und der Nutzer sie bereits in seiner Umgebung bereitstellt.

## Prozesslebenszyklus

1. Backend validiert die Startkonfiguration.
2. PTY und Kindprozess werden gestartet.
3. `ProcessStarted` wird erst nach bestätigtem Start gemeldet.
4. Ausgabeereignisse dürfen die UI nicht blockieren.
5. Prozessende erzeugt genau ein Exit-Ereignis.
6. Entfernen des sichtbaren Panes und Freigabe des Prozesses werden kontrolliert koordiniert.
7. Ressourcen werden auch bei Fehlern, Abbruch und Fensterschließung freigegeben.

Beim Schließen gilt eine ausdrücklich definierte Policy, beispielsweise:

- laufenden Prozess höflich beenden,
- nach einer Frist eskalieren,
- Nutzer bei erkennbar laufender Aufgabe warnen.

Die konkrete Policy benötigt vor Implementierung eine dokumentierte Entscheidung. Stilles hartes Beenden ist nicht der Standard.

## Nebenläufigkeit

- GTK- und VTE-Aufrufe erfolgen nur im vorgesehenen Hauptkontext.
- Netzwerk- und KI-Arbeit gehört nie in das Backend.
- Ereignisse werden über klar definierte Kanäle oder Callbacks an die Anwendung gemeldet.
- Hochfrequente Ausgabe darf nicht für jedes Byte eine teure UI-Aktualisierung auslösen.
- Abmeldung und Schließen müssen gegen verspätete Ereignisse abgesichert sein.
- Blockierende Prozessabfragen dürfen den GTK-Hauptthread nicht anhalten.

## Textzugriff

Terminaltext wird nur auf ausdrückliche Aktion der Anwendung ausgelesen.

Ein fachlicher `TextSnapshot` enthält mindestens:

- Text
- Herkunft: Auswahl, sichtbarer Bereich oder ausdrücklich angeforderter Ausschnitt
- Zeitpunkt
- Hinweis, ob der Text gekürzt wurde

Pane-Zuordnung und weitere Anwendungsmetadaten liegen außerhalb des Backendkerns.

Das Backend entscheidet nicht, ob Text an einen KI-Provider gesendet werden darf. Freigabe, Secret-Filterung und Vorschau gehören in die KI- und Sicherheitsschicht.

## Arbeitsverzeichnis

Die Erkennung ist bestmöglich und kann fehlschlagen.

- `unbekannt` ist ein gültiger Zustand.
- Keine heuristische Vermutung wird als sicherer Pfad ausgegeben.
- Pfade werden vor Nutzung validiert.
- Das Backend meldet Änderungen, ohne Startprofile selbst anzupassen.

## VTE-Implementierung

`VteBackend` ist die einzige Schicht, die VTE-spezifische Typen und Signale kennen darf.

Erlaubte Abhängigkeit:

```text
UI/Application -> TerminalBackend -> VteBackend -> VTE
```

Nicht erlaubt:

```text
UI -> VTE
Profiles -> VTE
AI -> VTE
State model -> VTE widget
```

Die Backend-Grenze muss klein bleiben. Sie wird nicht vorsorglich zu einer generischen Multi-Backend-Plattform ausgebaut.

## Fehler

Backendfehler sind typisiert, beispielsweise:

- ungültige Startkonfiguration
- Startverzeichnis nicht verfügbar
- Prozess konnte nicht gestartet werden
- Operation auf geschlossenem Handle
- Zwischenablage nicht verfügbar
- Suche fehlgeschlagen
- interne Backendinkonsistenz

Fehlertexte enthalten keine vollständigen Umgebungsvariablen, Terminalinhalte oder Secrets.

## Austauschbarkeit

Die Abstraktion soll einen späteren Backendwechsel ermöglichen, aber keine hypothetische Multi-Backend-Plattform vorwegnehmen.

Für das MVP gilt:

- genau eine produktive Implementierung: VTE
- keine Laufzeitwahl zwischen Backends
- keine Funktionen nur für unbekannte zukünftige Backends
- keine zusätzliche Parser- oder Renderer-Abhängigkeit

## Verifikation

Automatisiert testbar sind mindestens:

- Startkonfiguration und Validierung
- Zustandsübergänge eines Handles
- genau ein Exit-Ereignis pro Prozess
- Verhalten nach geschlossenem Handle
- Event-Entkopplung vom Anwendungszustand
- kontrollierte Freigabe bei Fehler und Abbruch

Zusätzlich werden echte VTE-/PTY-Szenarien gemäß [`TEST_STRATEGY.md`](TEST_STRATEGY.md) manuell und als Integrationstest geprüft.
