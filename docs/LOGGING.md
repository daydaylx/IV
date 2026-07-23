# Logging

## Ziel

Logs unterstützen Diagnose und Entwicklung, ohne Terminalinhalte, Zugangsdaten oder KI-Kontext unnötig zu speichern.

## Standard

- strukturierte Logs mit Zeit, Level, Modul und Ereignis
- keine Telemetrie und kein automatischer Upload
- normale Nutzung standardmäßig mit zurückhaltendem Log-Level
- Debug-Logging wird bewusst aktiviert

## Log-Level

- `error`: nicht selbstständig behebbarer Fehler einer Funktion
- `warn`: sicherer Fallback oder auffälliger Zustand
- `info`: wichtige Lebenszyklusereignisse ohne Nutzerdaten
- `debug`: technische Diagnose für Entwicklung
- `trace`: nur lokal und kurzfristig; nie standardmäßig aktiv

## Zulässige Beispiele

- Anwendungsversion und Plattform
- Modulstart und Modulende
- Pane- oder Request-ID als interne, nicht persistente Kennung
- Prozess gestartet oder beendet, ohne vollständige Kommandozeile
- Exit-Code
- Requestdauer, Statuscode und Antwortgröße
- verwendeter Konfigurationspfad
- Zahl geladener Tabs oder Profile

## Verbotene Inhalte

- API-Schlüssel und Tokens
- private Schlüssel
- Passwörter
- vollständige Umgebungsvariablen
- Shell-Eingaben
- Terminalausgaben
- Zwischenablageinhalte
- vollständige KI-Prompts und Antworten
- Authorization-Header
- vollständige URLs mit sensitiven Queryparametern

## Redaction

Fehlerobjekte und HTTP-Daten werden vor dem Logging bereinigt. Bekannte Schlüssel wie `authorization`, `token`, `api_key`, `password` und `secret` werden vollständig ersetzt, nicht nur teilweise maskiert.

## Pfade

Private absolute Pfade werden nur geloggt, wenn sie zur lokalen Diagnose erforderlich sind. Wo möglich werden Pfade gekürzt oder als Kategorie ausgegeben. Pfade werden nie an einen externen Dienst übertragen.

## Prozessinformationen

Standardmäßig werden nur Prozessart, Startstatus und Exit-Code erfasst. Vollständige Argumentlisten werden nicht geloggt, da sie Secrets enthalten können.

## KI- und Netzwerkereignisse

Zulässig:

- Providerkennung
- Modellkennung
- Request-ID
- Dauer
- Abbruchstatus
- HTTP-Status
- Zahl gesendeter und empfangener Bytes oder Tokens, falls verfügbar

Nicht zulässig:

- Kontexttext
- Prompttext
- Antworttext
- API-Key
- vollständige Header

## Speicherung

- Logs liegen lokal
- begrenzte Dateigröße und Rotation
- alte Logs werden gelöscht
- kein unbegrenztes Wachstum
- Dateirechte folgen dem privaten Benutzerkontext

Die konkrete Bibliothek und Rotation werden bei Implementierung dokumentiert.

## Debug-Modus

Debug-Modus darf zusätzliche technische Metadaten erfassen, aber keine verbotenen Inhalte. Eine Option „Terminalinhalt loggen“ wird im MVP nicht angeboten.

## Crash- und Fehlerberichte

Es gibt keinen automatischen Upload. Exportierte Diagnosepakete zeigen vor dem Speichern, welche Dateien enthalten sind, falls eine solche Funktion später entsteht.

## Review

Jede neue Logzeile wird auf Folgendes geprüft:

1. Ist sie für Diagnose nötig?
2. Kann ein Wert Nutzerdaten oder Secrets enthalten?
3. Reicht eine ID, Kategorie oder Länge statt des Inhalts?
4. Kann die Meldung sehr häufig auftreten?
5. Ist das gewählte Level angemessen?