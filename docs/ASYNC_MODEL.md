# Async- und Threading-Modell

## Ziel

IV muss Terminaleingabe, Rendering und GTK-Interaktionen jederzeit reaktionsfähig halten. Hintergrundarbeit wird klar vom GTK-Hauptkontext getrennt.

## Grundregel

GTK- und VTE-Objekte werden ausschließlich im vorgesehenen GTK-Hauptkontext verwendet. Netzwerk, Dateizugriffe mit möglicher Latenz, KI-Streaming und rechenintensive Verarbeitung laufen außerhalb des UI-Threads.

## Verantwortungsbereiche

### GTK-Hauptkontext

- Widgets erstellen und verändern
- Fokus und Navigation
- VTE-Signale verarbeiten
- kleine Zustandsänderungen anwenden
- bereits vorbereitete Ergebnisse darstellen

### Hintergrundausführung

- HTTP-Anfragen und Streaming
- Konfigurations- und Profildateien laden oder schreiben
- größere Textaufbereitung
- optionale Secret-Prüfung
- Diagnose- und Benchmarkarbeiten

## Kommunikation

Zwischen Hintergrundtasks und UI werden typisierte Nachrichten verwendet. Nachrichten enthalten keine GTK- oder VTE-Referenzen.

Beispiele:

```text
AiChunk { request_id, text }
AiCompleted { request_id }
AiFailed { request_id, error }
ConfigLoaded { version, config }
PersistCompleted { operation_id }
```

## Lebenszyklus

Jede Hintergrundoperation besitzt:

- stabile ID
- Besitzer oder Zielkontext
- Abbruchmöglichkeit, sofern sinnvoll
- klaren Abschlusszustand

Wird ein Pane, Tab oder Fenster geschlossen, werden zugehörige Tasks abgebrochen oder deren spätere Ergebnisse sicher verworfen.

## Abbruch

Abbruch ist ein normaler Zustand, kein unerwarteter Fehler.

- KI-Requests sind abbrechbar
- Dateischreibvorgänge werden nur abgebrochen, wenn dies atomar sicher möglich ist
- Prozessbeendigung folgt dem Terminal-Backend-Vertrag
- nach Abbruch werden Ressourcen freigegeben

## Backpressure

Streaming- und Ausgabeereignisse dürfen die UI-Warteschlange nicht unbegrenzt füllen.

- kleine Chunks dürfen gebündelt werden
- Statusanzeigen werden gedrosselt
- veraltete Zwischenstände können verworfen werden
- Abschluss- und Fehlerereignisse dürfen nicht verloren gehen

## Fehler

Task-Fehler werden in typisierte Anwendungsergebnisse übersetzt. Ein fehlgeschlagener Hintergrundtask darf keine Panik im GTK-Hauptkontext auslösen.

## Runtime

Das Projekt verwendet höchstens eine allgemeine Async-Runtime. Die konkrete Wahl wird bei Implementierung dokumentiert. Es wird keine zweite Runtime nur für einzelne Bibliotheken eingeführt, sofern dies vermeidbar ist.

## Verbote

- blockierendes Warten im GTK-Hauptthread
- `sleep` im UI-Pfad
- GTK-Objekte in allgemeinen Worker-Tasks
- globale unkontrollierte Tasklisten
- detached Tasks ohne Besitzer oder Fehlerbehandlung
- unbegrenzte Kanäle für hochfrequente Daten ohne Begründung

## Tests

- UI bleibt während langsamer KI-Antwort bedienbar
- Abbruch stoppt Streaming
- verspätete Chunks geschlossener Panes werden ignoriert
- schnelle Ereignisfolgen erzeugen keine unkontrollierte Warteschlange
- Taskfehler verändern nicht den Terminalprozesszustand
- wiederholtes Öffnen und Schließen hinterlässt keine Tasks