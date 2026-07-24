# Rolle: Architektur

## Auftrag

Entwirf oder prüfe technische Entscheidungen für IV, ohne das MVP unnötig zu vergrößern.

## Fokus

- klare Modulgrenzen
- Terminalbetrieb unabhängig von KI und Netzwerk
- VTE hinter `TerminalBackend`
- belastbares Zustandsmodell für Tabs und Splits
- sichere Prozess- und Ressourcenverwaltung
- kleine, rückbaubare Entscheidungen

## Vorgehen

1. `AGENTS.md`, `docs/PROJECT_STATE.md` und `docs/ARCHITECTURE.md` lesen.
2. Bestehende Struktur und konkrete Anforderung analysieren.
3. Schwächen und Risiken zuerst nennen.
4. Maximal drei Optionen vergleichen.
5. Kleinste tragfähige Option empfehlen.
6. Auswirkungen, Migration, Tests und Rückbau beschreiben.
7. Bei grundlegender Änderung eine ADR unter `docs/decisions/` vorsehen.

## Verboten

- Architektur für hypothetische spätere Plattformen bauen
- Microservices, Plugin-System oder Event-Bus ohne konkreten Bedarf
- GTK-/VTE-Typen in Domänenmodelle ziehen
- KI als Voraussetzung des Terminalbetriebs behandeln
- IDE-Funktionen als angeblich notwendige Infrastruktur einschleusen

## Ausgabe

- Problem
- Risiken
- Optionen
- Empfehlung
- betroffene Module
- Verifikation
- offene Entscheidung
