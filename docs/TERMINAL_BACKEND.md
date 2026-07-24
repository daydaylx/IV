# Terminal-Backend

## Dokumentstatus

Dieses Dokument beschreibt den fachlichen Vertrag des MVP-Terminal-Backends. Die dargestellten Operationen und Ereignisnamen bleiben konzeptuell und sind keine verbindliche Rust-API.

Die kleine tatsächliche Phase-0-Schnittstelle wurde gegen `gtk4-rs`, VTE und den realen Prozesslebenszyklus geprüft. Der Implementierungsstand steht in [`PROJECT_STATE.md`](PROJECT_STATE.md).

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

### Konkrete Policy in Phase 0

Phase 0 verwendet eine feste, nichtblockierende Schließsequenz:

1. Ein noch laufender Shell-Prozess erhält `SIGHUP`.
2. GTK und VTE verarbeiten Prozessereignisse weiter; der Hauptthread wartet nicht blockierend.
3. Ist derselbe Prozesszustand nach 1,5 Sekunden noch aktiv, folgt `SIGKILL`.
4. Nach insgesamt 2,5 Sekunden wird das Fenster auch dann freigegeben, wenn kein weiteres VTE-Ereignis eingetroffen ist.
5. Verspätete oder doppelte Spawn- und Exit-Ereignisse werden anhand des Zustands ignoriert.

Ein noch startender Spawn wird zunächst über `gio::Cancellable` abgebrochen und unterliegt derselben begrenzten Abschlusssequenz. Ein normaler Exitstatus 0 schließt die Phase-0-Anwendung. Ein Nichtnullstatus oder Signal hält das Fenster mit deaktivierter Eingabe und sichtbarer Meldung offen.

Die Signale richten sich in Phase 0 an die von VTE zurückgegebene Shell-PID, nicht an eine selbst verwaltete Prozessgruppe. Eine allgemeine Job- oder Prozessmanagementschicht ist ausdrücklich nicht implementiert.

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

Die konkrete Phase-0-Abhängigkeit lautet:

Erlaubte Abhängigkeit:

```text
Application -> UI -> Terminal -> VteBackend -> VTE
```

Nicht erlaubt:

```text
UI -> VTE
Profiles -> VTE
AI -> VTE
State model -> VTE widget
```

Die Backend-Grenze muss klein bleiben. Sie wird nicht vorsorglich zu einer generischen Multi-Backend-Plattform ausgebaut.

`Terminal` ist dabei eine konkrete interne Fassade und kein vorsorglicher Trait. Die UI erhält von ihr nur ein allgemeines GTK-Widget, typisierte Ereignisse und die aktuell benötigten Operationen Start, Fokus, Copy, Paste und Schließen.

Die vorhandene Prozessumgebung wird ohne Protokollierung an die Shell weitergereicht. Nicht als UTF-8 darstellbare Einträge können über die String-basierte VTE-Bindingschnittstelle nicht übertragen werden. `TERM` und `COLORTERM` werden gezielt auf `xterm-256color` und `truecolor` gesetzt, damit interaktive Programme nicht eine ungeeignete Elternkonfiguration wie `TERM=dumb` übernehmen.

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
