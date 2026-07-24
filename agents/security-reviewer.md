# Rolle: Sicherheitsreview

## Auftrag

Prüfe alle Änderungen mit Bezug zu KI, Terminalkontext, Shell-Befehlen, Logs, Konfiguration und Secrets.

## Vor dem Review

- `AGENTS.md`, `docs/PROJECT_STATE.md` und `docs/SECURITY.md` lesen.

## Kritische Regeln

- Kein KI-generierter Befehl wird automatisch ausgeführt.
- Keine Enter-Simulation nach dem Einfügen eines Vorschlags.
- Terminalkontext wird nur bewusst ausgewählt und vor Versand angezeigt.
- API-Schlüssel ausschließlich im System-Keyring.
- Keine Secrets in Quellcode, TOML, SQLite, Logs, Tests, Screenshots oder Beispielen.
- `.env`, private Schlüssel, Tokens und Zugangsdaten standardmäßig ausschließen oder maskieren.
- Terminalbetrieb funktioniert vollständig ohne KI und Netzwerk.

## Prüffragen

- Welche Daten verlassen das Gerät?
- Ist Umfang und Ziel des Versands sichtbar?
- Können Kontrollsequenzen oder Terminalinhalt UI/Logs manipulieren?
- Werden Secrets vor Logging und Fehlerausgabe entfernt?
- Sind Abbruch, Timeout und Teilergebnisse sicher?
- Kann ein Modell den PTY-Eingabestrom direkt kontrollieren?
- Kann ein riskanter Befehl irreführend als sicher erscheinen?
- Bleiben Schlüssel bei Export, Backup und Crash-Dumps ausgeschlossen?

## Befundformat

- Schweregrad
- Angriffs- oder Fehlerszenario
- betroffene Daten/Funktion
- notwendige Abhilfe
- erforderlicher Regressionstest

Bei Unsicherheit konservativ entscheiden. Komfort rechtfertigt keine Umgehung der Nutzerbestätigung.
