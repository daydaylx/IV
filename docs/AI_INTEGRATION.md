# KI-Integration

## Dokumentstatus

Dieses Dokument beschreibt den geplanten Umfang und die Sicherheitsgrenzen für Phase 3. Es belegt keine bereits vorhandene KI-Implementierung.

Der tatsächliche Projektstand steht in [`PROJECT_STATE.md`](PROJECT_STATE.md). Vor Phase 3 darf dieses Dokument keine Architekturarbeit auslösen, die für den stabilen Terminalkern nicht erforderlich ist.

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

## Geplante Architektur

```text
UI
└── AiController
    ├── ContextBuilder
    ├── SecretFilter
    ├── AiProvider
    └── RequestLifecycle
```

Diese Namen beschreiben Verantwortlichkeiten und sind keine verbindlichen Rust-Typnamen.

Der Provider kennt weder GTK- noch VTE-Typen. Er erhält nur einen geprüften Request und liefert typisierte Streaming-Ereignisse.

## Kontextfluss

1. Nutzer löst eine KI-Aktion aus.
2. Die Anwendung erzeugt einen unveränderlichen Text-Snapshot.
3. Herkunft, Umfang und Kürzung werden angezeigt.
4. Ein Secret-Filter markiert oder entfernt bekannte sensible Muster.
5. Nutzer prüft und bestätigt den finalen Kontext.
6. Request wird mit stabiler Request-ID gestartet.
7. Antwort wird gestreamt und kann abgebrochen werden.
8. Ergebnis wird als nicht vertrauenswürdige Erklärung oder Vorschlag dargestellt.

Ohne Bestätigung wird nichts übertragen.

## Konzeptuelles Requestmodell

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
- kompletten Terminalbuffer ohne bewusste Auswahl
- implizit geladene Dateien

Konkrete Typen und Feldnamen werden erst mit der Implementierung verbindlich.

## Konzeptuelles Antwortmodell

Ereignisse bilden mindestens folgende Zustände ab:

```text
Started
Chunk { text }
Completed { usage? }
Cancelled
Failed { kind, message }
```

Antworttext wird als nicht vertrauenswürdiger Inhalt behandelt.

## Befehlsvorschläge

- als KI-generierten Vorschlag kennzeichnen
- keine automatische Ausführung
- Kopieren und Einfügen getrennt anbieten
- Einfügen verändert nur die aktuelle Eingabe
- kein Enter oder anderer Bestätigungsschritt wird simuliert
- mehrzeilige Vorschläge sichtbar als solche behandeln

Eine optionale Risikowarnung darf bekannte destruktive Muster markieren, ist aber keine Sicherheitsgarantie.

## Providerfehler

Mindestens behandeln:

- fehlender API-Key
- Keyring nicht verfügbar
- ungültiger Endpunkt
- TLS-Fehler
- Zeitüberschreitung
- Rate Limit
- ungültige Antwort
- zu große Antwort
- Abbruch

Keiner dieser Fehler darf Terminalprozess oder Terminal-Pane beeinflussen.

## Datenschutz

- keine Telemetrie
- keine automatische Speicherung von Prompts oder Antworten
- keine Terminalinhalte in allgemeinen Logs
- Endpunkt und Modell vor Versand erkennbar
- Kontextmenge minimieren
- keine unsichere Klartextspeicherung bei fehlendem Keyring

Weitere Regeln: [`SECURITY.md`](SECURITY.md) und [`LOGGING.md`](LOGGING.md).

## Testbarkeit

Der Provider wird hinter einer kleinen Schnittstelle gekapselt. Tests verwenden einen lokalen Mock-Provider und prüfen mindestens:

- Streamingreihenfolge
- Abbruch und Zeitüberschreitung
- Rate Limit und Fehlerantworten
- Pane- oder Fensterschließung während einer Anfrage
- keine automatische Ausführung
- keine Enter-Simulation
- Redaction bekannter Secrets
- keine sensiblen Daten in Logs
- vollständige Terminalnutzung ohne Netzwerk und Provider

## Erweiterungen nach dem MVP

Weitere Provider oder lokale Modelle benötigen eine neue Produkt- und Architekturentscheidung. Die MVP-Schnittstelle soll sauber sein, aber keine Providerplattform vorwegnehmen.
