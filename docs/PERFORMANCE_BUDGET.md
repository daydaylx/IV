# Performance-Budget

## Ziel

IV soll sich wie ein leichtgewichtiger nativer Terminal-Emulator anfühlen. Performance wird gemessen, nicht nur subjektiv behauptet.

## Prioritäten

1. Eingabe reagiert unmittelbar.
2. Terminalausgabe blockiert die UI nicht.
3. Resize bleibt flüssig.
4. Hintergrund- und KI-Aufgaben beeinflussen den Terminalbetrieb nicht.
5. Speicher wächst bei normaler Nutzung kontrolliert.

## Vorläufige Budgets

Diese Werte sind Startziele und werden auf der Zielhardware überprüft:

- Kaltstart bis sichtbares Fenster: möglichst unter 500 ms
- Start bis eingabebereite Shell: möglichst unter 1 s
- UI-Reaktion auf Tastatureingabe: keine sichtbar wahrnehmbare Verzögerung
- Resize: flüssig ohne längere Blockaden
- Leerlauf-CPU: nahe null
- Speicher mit einem leeren Pane: deutlich unter typischen Electron-Terminals
- KI-Anfrage: keine messbare Verschlechterung der Terminaleingabe

Es wird keine harte Zahl als bestanden behauptet, bevor reproduzierbare Messungen existieren.

## Messszenarien

- ein leeres Pane
- zehn Tabs mit je einem Pane
- mehrere verschachtelte Splits
- lange Scrollback-Historie
- schnelle Dauerausgabe
- interaktive Programme
- laufendes KI-Streaming
- wiederholtes Öffnen und Schließen von Panes

## Regeln

- keine blockierenden Netzwerk- oder Dateiaufgaben im UI-Thread
- hochfrequente Statusaktualisierungen bündeln
- Terminalausgabe nicht unnötig kopieren
- keine dauerhafte Analyse des Terminalbuffers
- große KI-Kontexte nur auf ausdrückliche Aktion erzeugen
- Ressourcen geschlossener Panes und Requests zuverlässig freigeben

## Regressionen

Eine relevante Verschlechterung bei Startzeit, Eingabelatenz, CPU oder Speicher muss erklärt und entweder behoben oder als bewusste Entscheidung dokumentiert werden.

## Werkzeuge

Geeignete Werkzeuge werden bei Projektstart festgelegt, beispielsweise:

- `hyperfine` für Startmessungen
- `/usr/bin/time` und Systemmonitoring für Speicher
- Profiler für CPU und Allocations
- reproduzierbare Ausgabegeneratoren für PTY-Last

Messbefehle und Ergebnisse gehören später in eine versionierte Benchmark-Anleitung, nicht in Behauptungen im README.