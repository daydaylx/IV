# KI-Integration

## Ziel

Die KI unterstützt konkrete Terminalaufgaben, bleibt aber optional und technisch vollständig vom Terminalkern getrennt.

## MVP-Umfang

- genau ein OpenAI-kompatibler Provider
- ein konfigurierbarer Endpunkt
- ein Modell
- Streaming
- Abbruch
- markierten oder bewusst ausgewählten Terminaltext erklären
- Fehlermeldungen analysieren
- Shell-Befehle erzeugen, verbessern und erklären
- Vorschläge kopieren oder in die Eingabe übernehmen

## Nicht enthalten

- automatische Ausführung
- simuliertes Enter
- Tool-Aufrufe
- MCP
- autonomer Agent
- permanente Terminalüberwachung
- automatische Analyse aller Ausgaben
- mehrere Provider
- Gesprächsgedächtnis über Sitzungen hinweg

## Architektur

```text
UI
└── AiController
    ├── ContextBuilder
    ├── SecretFilter
    ├── AiProvider
    └── RequestLifecycle
```

Der Provider kennt weder GTK- noch VTE-Typen. Er erhält nur einen geprüften Request und liefert typisierte Streaming-Ereignisse.

## Kontextfluss

1. Nutzer löst eine KI-Aktion aus.
2. Die Anwendung erzeugt einen Text-Snapshot.
3. Herkunft, Umfang und Kürzung werden angezeigt.
4. SecretFilter markiert oder entfernt bekannte sensible Muster.
5. Nutzer bestätigt den finalen Kontext.
6. Request wird mit stabiler `RequestId` gestartet.
7. Antwort wird gestreamt und kann abgebrochen werden.
8. Ergebnis wird als Erklärung oder Vorschlag dargestellt.

Ohne Bestätigung wird nichts übertragen.

## Requestmodell

Ein Request enthält mindestens:

- Request-ID
- Aktionstyp
- Systemanweisung
- bestätigten Kontext
- optionalen Nutzertext
- Modellkennung
- Abbruchsignal

Er enthält nicht:

- API-Key als serialisierbares Feld
- GTK-/VTE-Referenzen
- kompletten Terminalbuffer ohne Auswahl
- implizit geladene Dateien

## Antwortmodell

Ereignisse:

```text
Started
Chunk { text }
Completed { usage? }
Cancelled
Failed { kind, message }
```

Antworttext wird als nicht vertrauenswürdiger Inhalt behandelt.

## Befehlsvorschläge

- als Vorschlag kennzeichnen
- keine automatische Ausführung
- Kopieren und Einfügen getrennt anbieten
- Einfügen verändert nur die aktuelle Eingabe
- kein Enter oder anderer Bestätigungsschritt wird simuliert
- mehrzeilige Vorschläge werden sichtbar als solche behandelt

Eine optionale Risikowarnung darf bekannte destruktive Muster markieren, ist aber keine Sicherheitsgarantie.

## Providerfehler

- fehlender API-Key
- Keyring nicht verfügbar
- ungültiger Endpunkt
- TLS-Fehler
- Zeitüberschreitung
- Rate Limit
- ungültige Antwort
- zu große Antwort
- Abbruch

Keiner dieser Fehler darf das Terminal-Pane beeinflussen.

## Datenschutz

- keine Telemetrie
- keine automatische Speicherung von Prompts oder Antworten
- keine Terminalinhalte in allgemeinen Logs
- Endpunkt und Modell sind vor Versand erkennbar
- Kontextmenge wird minimiert

## Testbarkeit

Der Provider wird hinter einer Schnittstelle gekapselt. Tests verwenden einen lokalen Mock-Provider und prüfen:

- Streamingreihenfolge
- Abbruch
- Zeitüberschreitung
- Rate Limit und Fehlerantworten
- geschlossene Panes
- keine automatische Ausführung
- Redaction bekannter Secrets
- keine sensiblen Daten in Logs

## Erweiterungen nach dem MVP

Weitere Provider oder lokale Modelle benötigen eine neue Produkt- und Architekturentscheidung. Die MVP-Schnittstelle soll sauber sein, aber keine komplexe Providerplattform vorwegnehmen.