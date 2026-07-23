# Fehlerbehandlung

## Ziel

Fehler sollen verständlich, typisiert und lokal begrenzt behandelt werden. Ein Fehler in KI, Konfiguration oder Persistenz darf laufende Terminals nicht beenden.

## Grundregeln

- keine still geschluckten Fehler
- keine Panics für erwartbare Laufzeitfehler
- Fehler erhalten technischen Kontext, aber keine Secrets
- Benutzertexte erklären Auswirkung und nächste sinnvolle Aktion
- interne Fehlerketten bleiben für Diagnose erhalten
- UI, Domäne und Infrastruktur übersetzen Fehler an ihren Grenzen

## Fehlerklassen

### Terminalkritisch

Beispiele:

- Shell konnte nicht gestartet werden
- PTY-Erstellung fehlgeschlagen
- Terminal-Handle ist inkonsistent

Verhalten:

- betroffenes Pane zeigt einen klaren Fehlerzustand
- andere Panes bleiben nutzbar
- erneuter Start oder Schließen wird angeboten

### Wiederherstellbar

Beispiele:

- Startverzeichnis fehlt
- Zwischenablage nicht verfügbar
- Layout enthält ungültige Werte

Verhalten:

- sicherer Fallback
- nicht-blockierende Meldung
- Aktion kann erneut versucht werden

### Optionale Dienste

Beispiele:

- KI-Provider nicht erreichbar
- Keyring gesperrt
- Netzwerkzeitüberschreitung

Verhalten:

- nur die betroffene Zusatzfunktion schlägt fehl
- Terminal bleibt vollständig bedienbar
- Request kann abgebrochen oder wiederholt werden

### Programmierfehler

Verletzte Invarianten werden in Debug-Builds deutlich sichtbar gemacht. In produktiven Builds wird soweit möglich ein gültiger Zustand wiederhergestellt, protokolliert und die betroffene Aktion beendet.

## Benutzeroberfläche

- kurze Meldung zuerst
- Details optional aufklappbar
- modale Dialoge nur bei notwendiger Entscheidung
- wiederholte identische Fehler nicht als Meldungsflut anzeigen
- Fehlermeldungen verschwinden nicht, bevor der Nutzer sie wahrnehmen kann

## Typisierte Fehler

Module definieren eigene Fehlerarten und geben keine beliebigen Stringfehler über mehrere Schichten weiter. Fehlerquellen werden mit `source` beziehungsweise einer nachvollziehbaren Kette erhalten.

## Konfiguration

Bei ungültiger Konfiguration:

1. fehlerhafte Stelle identifizieren
2. gültige Teile möglichst weiterverwenden
3. für ungültige Werte sichere Standards einsetzen
4. Anwendung starten lassen, sofern der Terminalkern möglich bleibt
5. keine Datei automatisch zerstörend überschreiben

## Nebenläufigkeit

- abgebrochene Tasks sind kein generischer Fehler
- verspätete Ergebnisse geschlossener Panes werden verworfen
- Kanalabbrüche werden als Lebenszyklusereignis bewertet
- Fehler aus Hintergrundtasks werden in den Hauptkontext übertragen, ohne diesen zu blockieren

## Logging

Technische Details werden gemäß `docs/LOGGING.md` protokolliert. Terminalinhalt, API-Schlüssel und vollständige Prompts dürfen nicht Teil allgemeiner Fehlerlogs sein.

## Tests

Für jeden relevanten Fehlerpfad wird geprüft:

- sichtbare Auswirkung
- sicherer Folgezustand
- keine Beschädigung anderer Sitzungen
- keine Ressourcen- oder Prozesslecks
- keine sensiblen Daten in Meldung oder Log