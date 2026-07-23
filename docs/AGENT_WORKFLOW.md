# Agenten-Workflow

## Zweck

Dieser Ablauf verhindert große, unprüfbare Änderungen und schützt das Projekt vor schleichender IDE-Komplexität.

## 1. Einordnung

Vor jeder Änderung beantworten:

- Welches konkrete Terminalproblem wird gelöst?
- Gehört es zur aktuellen Phase?
- Welche Nicht-Ziele könnten berührt werden?
- Welche Module und Nutzerabläufe sind betroffen?
- Wie wird der Erfolg überprüft?

Fällt die Antwort auf die erste Frage schwach aus, Änderung nicht implementieren.

## 2. Planung

Der Plan enthält höchstens:

- Ziel
- Nicht-Ziel
- Annahmen
- betroffene Dateien/Module
- Risiken
- Umsetzungsschritte
- Tests und manuelle Verifikation
- Abschlusskriterien

Keine allgemeinen Neuentwürfe, wenn eine lokale Änderung genügt.

## 3. Umsetzung

- Änderung in kleinen Schritten vornehmen.
- Bestehende APIs verwenden, bevor neue Abstraktionen entstehen.
- Keine Nebenarbeiten oder kosmetischen Großumbauten einmischen.
- Bei entdeckten Altproblemen separat dokumentieren.
- Sicherheitsgrenzen nicht zugunsten schneller Demos umgehen.

## 4. Review

Review-Reihenfolge:

1. Produktgrenzen und Scope
2. Prozess- und Datenverlust-Risiken
3. Nebenläufigkeit und UI-Blockaden
4. Secret- und KI-Sicherheit
5. Zustandskonsistenz bei Tabs/Splits
6. Fehlerbehandlung
7. Tests und Wartbarkeit
8. Optik und Stil

## 5. Verifikation

Automatische Prüfungen aus `docs/DEFINITION_OF_DONE.md` ausführen. Terminalverhalten zusätzlich manuell prüfen, wenn PTY, VTE, Fokus, Resize, Clipboard oder Prozesslebenszyklus betroffen sind.

## 6. Abschluss

Eine Aufgabe wird nicht als fertig bezeichnet, wenn:

- Tests nicht ausgeführt wurden und kein Grund genannt ist,
- zentrale Fehlerfälle nur als TODO bestehen,
- Verhalten nur in einer Demo, nicht im realen Terminal geprüft wurde,
- autonome KI-Ausführung möglich ist,
- das Terminal ohne KI oder Netzwerk nicht mehr vollständig nutzbar ist.

## Rollenwahl

- Architektur oder Modulgrenzen: `agents/architect.md`
- konkrete Umsetzung: `agents/implementer.md`
- Qualitäts- und Scopeprüfung: `agents/reviewer.md`
- Teststrategie und Fehlerreproduktion: `agents/tester.md`
- KI, Secrets oder riskante Befehle: `agents/security-reviewer.md`

Ein Agent darf mehrere Rollen nacheinander übernehmen, muss die Perspektiven aber getrennt behandeln.
