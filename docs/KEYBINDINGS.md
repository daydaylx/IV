# Tastenkürzel

## Ziel

IV ist vollständig per Tastatur bedienbar, ohne übliche Eingaben interaktiver Terminalprogramme unnötig abzufangen.

## Grundregeln

- Terminaleingabe hat Vorrang
- globale Kürzel verwenden bewusst gewählte Modifikatorkombinationen
- Konflikte sind sichtbar und konfigurierbar
- keine Aktion besitzt mehrere widersprüchliche Standardkürzel
- benutzerdefinierte Änderungen werden validiert
- alle Kernaktionen sind zusätzlich über Menüs oder Befehlssuche erreichbar

## Vorgeschlagene Standardaktionen

Die konkrete Belegung wird vor Implementierung gegen GTK-, Desktop- und Terminalkonventionen geprüft.

| Aktion | Vorgeschlagenes Kürzel |
|---|---|
| Neuer Tab | `Ctrl+Shift+T` |
| Tab schließen | `Ctrl+Shift+W` |
| Nächster Tab | `Ctrl+PageDown` |
| Vorheriger Tab | `Ctrl+PageUp` |
| Vertikal teilen | `Ctrl+Shift+Right` |
| Horizontal teilen | `Ctrl+Shift+Down` |
| Fokus links/rechts/oben/unten | konfigurierbar |
| Aktives Pane schließen | `Ctrl+Shift+Q` oder konfigurierbar |
| Suche | `Ctrl+Shift+F` |
| Kopieren | `Ctrl+Shift+C` |
| Einfügen | `Ctrl+Shift+V` |
| KI-Seitenleiste | `Ctrl+Shift+I` |
| Vollbild | `F11` |
| Schrift vergrößern/verkleinern | `Ctrl++` / `Ctrl+-` |
| Schrift zurücksetzen | `Ctrl+0` |

Diese Tabelle ist eine Startentscheidung und darf nach realen Konflikttests angepasst werden.

## Konfliktregeln

Ein Kürzel darf nicht gleichzeitig:

- eine globale IV-Aktion
- eine Pane-Aktion
- und eine Text-/Terminalaktion

auslösen.

Bei Konflikten gilt Priorität:

1. expliziter, aktuell geöffneter Modus wie Suche
2. IV-Kernnavigation
3. Terminalweitergabe

Diese Priorität muss vorhersehbar und dokumentiert sein.

## Terminalprogramme

Besonders zu prüfen:

- vim/nvim
- tmux
- zellij
- less
- htop
- Shell-Readline
- Pi Agent
- Codex CLI
- Claude Code

IV soll keine verbreiteten Kombinationen ohne starken Grund belegen.

## Fokusnavigation

Räumliche Navigation arbeitet auf dem Pane-Baum. Kann in einer Richtung kein Pane gefunden werden, bleibt der Fokus unverändert. Es gibt kein zyklisches Springen, sofern dies nicht ausdrücklich als Option eingeführt wird.

## Eingabe mehrerer Tasten

Chord- oder Leader-Systeme gehören nicht in das MVP. Sie erhöhen Komplexität und kollidieren leicht mit Terminalanwendungen.

## Konfiguration

Benutzerdefinierte Kürzel:

- werden als Aktionen statt als interne Callbacknamen gespeichert
- müssen parsebar und eindeutig sein
- können auf Standardwerte zurückgesetzt werden
- dürfen Kernzugänglichkeit nicht vollständig unerreichbar machen

## Tests

- jede Kernaktion per Tastatur
- Konflikterkennung
- Layoutwechsel mit deutschem und US-Tastaturlayout
- Weitergabe nicht belegter Kombinationen
- Verhalten in vim, tmux und zellij
- Fokus kehrt nach Suche oder KI-Seitenleiste korrekt zurück