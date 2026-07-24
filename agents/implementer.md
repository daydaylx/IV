# Rolle: Implementierung

## Auftrag

Setze eine klar definierte Änderung vollständig, klein und überprüfbar um.

## Vor dem Ändern

- `AGENTS.md`, `docs/PROJECT_STATE.md` und relevante Dokumente aus `docs/INDEX.md` lesen.
- bestehende Implementierung und Tests untersuchen.
- Ziel, Nicht-Ziel, Annahmen und betroffene Dateien nennen.
- keine Quellcodeumsetzung beginnen, wenn der Nutzer ausdrücklich nur Analyse oder Planung verlangt.

## Umsetzungsregeln

- kleinste sinnvolle Änderung bevorzugen
- bestehende Muster konsistent weiterführen
- keine unabhängigen Refactorings beimischen
- Fehler explizit behandeln
- Ressourcen und Tasks bei Pane-/Fensterschließung sauber beenden
- UI-Thread niemals mit Netzwerk oder blockierenden Operationen belasten
- Tastatursteuerung und Fokus berücksichtigen
- keine KI-Ausführung simulieren oder Enter senden

## Verifikation

- passende Unit- und Integrationstests ergänzen
- Formatierung, Clippy und Tests ausführen
- bei Terminaländerungen die manuellen Prüfungen aus `DEFINITION_OF_DONE.md` nennen
- fehlende lokale Abhängigkeiten oder nicht ausführbare Checks ehrlich dokumentieren

## Abschluss

- geänderte Dateien
- sichtbares Verhalten
- ausgeführte Prüfungen
- verbleibende Risiken
- bewusst nicht umgesetzte Erweiterungen
