# Agentenrollen

## Zweck

Die Dateien in diesem Ordner sind aufgabenspezifische Arbeitsprofile. Sie ersetzen nicht die verbindlichen Regeln aus [`AGENTS.md`](../AGENTS.md).

Vor jeder Rolle gelten:

1. `AGENTS.md` lesen.
2. `docs/PROJECT_STATE.md` lesen.
3. Über `docs/INDEX.md` nur die relevanten Fachdokumente laden.
4. Die Rolle mit dem kleinsten passenden Zuständigkeitsbereich wählen.

## Rollenauswahl

### [`architect.md`](architect.md)

Verwenden bei:

- neuen oder veränderten Modulgrenzen
- Änderungen am Zustandsmodell
- Terminal-Backend oder Backendwechsel
- Async-Runtime und Thread-Grenzen
- Persistenzformat oder Secret-Speicherung
- KI-Providerarchitektur
- grundlegender Verzeichnis- oder Paketstruktur

Ergebnis ist eine Entscheidung oder ein umsetzbarer Architekturvorschlag, nicht automatisch Quellcode.

### [`implementer.md`](implementer.md)

Verwenden bei:

- klar definierter Umsetzung
- begrenztem Bugfix
- Regressionstest
- kleiner Refaktorierung innerhalb bestehender Grenzen
- Dokumentationsänderung mit eindeutigem Ziel

Nicht verwenden, wenn zuerst eine grundlegende Architekturentscheidung fehlt.

### [`reviewer.md`](reviewer.md)

Nach nicht trivialen Änderungen verwenden. Der Review prüft insbesondere:

- Produktgrenzen und Scope
- Prozess- und Ressourcenlebenszyklus
- Zustandskonsistenz
- UI-Thread und Nebenläufigkeit
- Fehler- und Abbruchpfade
- Tests und unnötige Komplexität

### [`tester.md`](tester.md)

Verwenden bei:

- Fehlerreproduktion
- Testplanung
- Regressionen
- Terminal-, PTY- oder Prozessproblemen
- Tabs, Splits, Fokus und Layoutzuständen
- nicht zuverlässig automatisierbarem GTK-/VTE-Verhalten

### [`security-reviewer.md`](security-reviewer.md)

Zusätzlich verpflichtend bei Änderungen an:

- KI-Kontext oder Providerkommunikation
- Shell-Eingaben und Befehlsvorschlägen
- Prozessstart oder Prozessende
- System-Keyring und Secrets
- Logging oder Fehlerausgaben
- URLs, Links oder Zwischenablage
- Konfigurationsimporten und externen Endpunkten

Ein Sicherheitsreview ersetzt keinen normalen technischen Review.

## Typische Abläufe

### Kleine klar definierte Änderung

`implementer` → `reviewer`

### Reproduzierbarer Fehler

`tester` → `implementer` → `reviewer`

### Architekturprägende Änderung

`architect` → ADR → `implementer` → `reviewer`

### Sicherheitsrelevante Änderung

passende Hauptrolle → `security-reviewer` → `reviewer`

## Übergabeformat

Jede Rolle schließt mit einem kurzen Übergabeblock:

```markdown
## Übergabe

- Entscheidung oder Ergebnis:
- Betroffene Dateien und Module:
- Verbindliche Regeln:
- Ausgeführte oder erforderliche Prüfungen:
- Offene Risiken:
- Bewusst ausgeschlossener Umfang:
- Nächste Rolle oder nächster Schritt:
```

Nicht zutreffende Punkte dürfen knapp als `entfällt` markiert werden. Die Übergabe soll den notwendigen Kontext transportieren, ohne den gesamten bisherigen Arbeitsverlauf zu wiederholen.
