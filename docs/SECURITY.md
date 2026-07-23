# Sicherheit

## Sicherheitsziel

IV verarbeitet Shell-Eingaben, Terminalausgaben, lokale Pfade und optional KI-Kontext. Das Hauptziel ist, unbeabsichtigte Befehlsausführung, Secret-Abfluss und eine Kopplung des Terminalbetriebs an Netzwerkdienste zu verhindern.

## Vertrauensgrenzen

### Lokal vertrauenswürdig

- Anwendungscode
- explizit gewählte lokale Konfiguration
- System-Keyring
- vom Nutzer gestartete Shell und Programme

### Nicht automatisch vertrauenswürdig

- Terminalausgaben
- kopierte Texte
- KI-Antworten
- Providerantworten
- externe URLs
- Inhalte fremder Repositories
- Umgebungsvariablen und Dateien mit möglichen Secrets

Terminalausgabe kann manipulative Anweisungen enthalten. Sie wird als Daten behandelt, nicht als vertrauenswürdige Steueranweisung.

## Verbindliche Regeln für KI-Befehle

1. Kein KI-Vorschlag wird automatisch ausgeführt.
2. Die Anwendung simuliert nach dem Einfügen niemals Enter.
3. Ein Vorschlag wird deutlich als KI-generiert gekennzeichnet.
4. Der Nutzer kann einen Vorschlag kopieren oder in die Eingabe übernehmen.
5. Riskante Befehle dürfen zusätzlich gewarnt werden, aber die Warnung ersetzt nicht die manuelle Bestätigung.
6. Modellantworten erhalten keine direkten Tools zur lokalen Befehlsausführung.

## Kontextfreigabe

Terminalinhalt wird nur nach einer bewussten Nutzeraktion an einen Provider gesendet.

Vor dem Senden wird sichtbar:

- welcher Text übertragen wird
- aus welchem Pane er stammt
- ob der Text gekürzt wurde
- welcher Provider und welches Modell verwendet werden

Standardmäßig ausgeschlossen:

- `.env` und Varianten
- private Schlüssel
- SSH-Schlüssel
- Zugangstoken
- Credential-Dateien
- bekannte Cloud- und Paketmanager-Credentials
- Inhalte, die als Secret erkannt wurden

Secret-Erkennung ist eine zusätzliche Schutzschicht und keine Garantie. Die bewusste Kontextvorschau bleibt erforderlich.

## API-Schlüssel

- ausschließlich im System-Keyring speichern
- nicht in TOML, SQLite, Logs oder Crashreports
- nicht im Klartext in UI-Debugansichten darstellen
- nur für die Dauer einer Anfrage im notwendigen Speicherbereich halten
- bei fehlendem Keyring keine unsichere Klartext-Fallbackspeicherung anbieten

## Logging

Logs enthalten standardmäßig nicht:

- Terminalausgaben
- Shell-Eingaben
- vollständige KI-Prompts
- KI-Antworten
- API-Schlüssel
- vollständige Umgebungsvariablen
- private Schlüssel oder Credential-Inhalte

Details stehen in `docs/LOGGING.md`.

## Netzwerk

- KI-Funktionen sind optional
- ohne Netzwerk bleibt das Terminal vollständig nutzbar
- TLS-Zertifikatsfehler werden nicht umgangen
- Endpunkte werden validiert
- Redirects und Fehlerantworten werden begrenzt behandelt
- Requests besitzen Zeitlimits und sind abbrechbar
- Antwortgrößen werden sinnvoll begrenzt

## Prozess- und PTY-Sicherheit

- Startbefehle werden nicht unnötig über zusätzliche Shell-Interpretation zusammengesetzt
- Programm und Argumente werden strukturiert übergeben, soweit möglich
- Arbeitsverzeichnisse werden validiert
- Kindprozesse werden kontrolliert beendet und aufgeräumt
- Prozess-IDs werden nicht als Beweis für Identität wiederverwendet
- fehlerhafte Prozesse dürfen den GTK-Hauptthread nicht blockieren

## Zwischenablage und Links

- mehrzeiliges Einfügen darf erkennbar behandelt werden
- anklickbare Links werden nicht automatisch geöffnet
- ungewöhnliche oder potenziell unsichere Schemes werden abgelehnt
- kopierter Text wird nicht verändert, ohne dies sichtbar zu machen

## Konfigurationssicherheit

- unbekannte oder fehlerhafte Werte führen zu sicheren Standards
- Konfigurationsdateien enthalten keine Secrets
- Dateirechte werden bei sensiblen lokalen Metadaten angemessen gesetzt
- importierte Profile werden validiert
- Pfadtraversal und unzulässige Dateipfade werden abgewiesen

## Bedrohungsszenarien

Mindestens zu berücksichtigen:

- Terminalausgabe versucht, den Agenten oder Nutzer zu manipulieren
- versehentlich markierter Secret-Text wird als KI-Kontext vorbereitet
- Provider liefert einen destruktiven Befehl
- manipulierte Konfiguration enthält unerwartete Pfade oder Endpunkte
- Netzwerkantwort ist extrem groß oder bleibt hängen
- Pane wird während einer Anfrage geschlossen
- Logs oder Fehlermeldungen geben vertrauliche Daten aus
- ein Kindprozess endet nicht sauber

## Sicherheitsreview

Eine Änderung benötigt besondere Prüfung, wenn sie:

- KI-Kontext auswählt oder überträgt
- Befehle in das Terminal einfügt
- Prozesse startet oder beendet
- Secrets oder Keyring-Zugriff betrifft
- URLs öffnet
- Konfiguration importiert
- Logging oder Fehlerausgaben erweitert

Dabei ist `agents/security-reviewer.md` zu verwenden.

## Meldung und Dokumentation

Gefundene Sicherheitsprobleme werden nicht durch stilles Abschwächen der Tests verdeckt. Auswirkungen, betroffene Daten, Reproduktionsweg und Behebung werden sachlich dokumentiert.

## Nicht-Ziele

Das MVP verspricht keine vollständige Sandbox für vom Nutzer gestartete Programme. Shell und Terminalprogramme besitzen grundsätzlich die Rechte des angemeldeten Nutzers. IV muss jedoch verhindern, dass die eigene KI-Integration diese Rechte autonom nutzt.