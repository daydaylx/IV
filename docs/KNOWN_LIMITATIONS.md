# Bekannte Einschränkungen

## Zweck

Dieses Dokument hält bewusst akzeptierte Grenzen fest. Einschränkungen werden nicht durch versteckte Zusatzarchitektur umgangen.

## Plattform

- Linux zuerst
- Wayland ist primäre Zielumgebung
- X11 nur, soweit GTK/VTE es ohne wesentlichen Zusatzaufwand tragen
- keine Unterstützung für Windows oder macOS im MVP

## Terminal

- VTE ist das einzige Backend im MVP
- kein eigener Terminalparser
- kein eigener GPU-Renderer
- keine vollständige Wiederherstellung laufender Prozesse nach Neustart
- Terminalbuffer werden nicht dauerhaft gespeichert

## Sitzungen

- Layouts können wiederhergestellt werden, Prozesse jedoch nicht nahtlos fortgesetzt werden
- das aktuelle Arbeitsverzeichnis kann technisch unbekannt sein
- ein ungültiges Verzeichnis fällt auf einen sicheren Startpfad zurück

## KI

- genau ein OpenAI-kompatibler Provider
- keine automatische Befehlsausführung
- keine Tool-Aufrufe oder Agenten-Orchestrierung
- kein MCP
- keine permanente Analyse von Terminalausgaben
- keine garantierte Erkennung aller Secrets
- KI-Antworten können sachlich falsch oder riskant sein und müssen vom Nutzer geprüft werden

## Oberfläche

- keine IDE-Funktionen
- kein Datei-Explorer
- kein integrierter Editor
- kein Debugger oder LSP
- kein visuelles Git-Frontend
- keine komplexe Projektverwaltung

## Synchronisierung und Konten

- keine Benutzerkonten
- keine Cloud-Synchronisierung
- keine Teamfunktionen
- keine Telemetrie

## Dokumentationsregel

Neue bekannte Grenzen werden ergänzt, sobald sie bestätigt sind. Eine Einschränkung wird erst entfernt, wenn Implementierung und Tests die Änderung belegen.