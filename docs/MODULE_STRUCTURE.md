# Modulstruktur

## Ziel

Die Modulstruktur hält UI, Zustand, Terminaltechnik, Persistenz und KI klar getrennt. Abhängigkeiten verlaufen nach innen zur Domänenlogik, nicht kreuz und quer zwischen Widgets und Infrastruktur.

## Vorgeschlagene Struktur

```text
src/
├── app/
├── domain/
├── terminal/
├── ui/
├── config/
├── profiles/
├── ai/
├── infrastructure/
└── main.rs
```

Die tatsächlichen Namen dürfen sich während Phase 0 leicht ändern. Die Verantwortungsgrenzen bleiben verbindlich.

## `app`

Koordiniert Anwendungsfälle und Zustandsänderungen:

- Tabs und Panes erstellen oder schließen
- Aktionen verarbeiten
- Dienste verbinden
- Lebenszyklen koordinieren

Keine direkten VTE- oder HTTP-Typen.

## `domain`

Enthält reine, weitgehend frameworkfreie Logik:

- IDs
- Zustandsmodelle
- Pane-Baum
- Prozess- und KI-Zustände
- Profile und Layoutdaten
- fachliche Fehler

Keine GTK-, VTE-, Tokio- oder HTTP-Abhängigkeit, sofern vermeidbar.

## `terminal`

Enthält:

- `TerminalBackend`-Vertrag
- Terminal-Handles und Ereignisse
- `VteBackend`
- Prozess- und PTY-nahe Umsetzung

Nur die konkrete VTE-Implementierung kennt VTE-Typen.

## `ui`

Enthält GTK4/libadwaita-Komponenten:

- Fenster
- Tabdarstellung
- Splitdarstellung
- Terminal-Pane-Container
- KI-Seitenleiste
- Einstellungen

Widgets lesen Zustand und lösen Aktionen aus. Sie enthalten keine eigenständige versteckte Geschäftslogik.

## `config`

- TOML-Schema
- Validierung
- Migration
- sichere Standardwerte
- XDG-Pfade

Keine Secrets.

## `profiles`

- Startprofile
- Layoutserialisierung
- letzte Verzeichnisse
- Validierung gespeicherter Pfade

## `ai`

- Providervertrag
- Request- und Antworttypen
- Kontextaufbereitung
- SecretFilter
- Requestlebenszyklus

Keine Terminal- oder GTK-Objekte.

## `infrastructure`

Technische Adapter, zum Beispiel:

- Keyring
- HTTP-Client
- Dateisystem
- Logging
- Zeitquelle

## Erlaubte Abhängigkeiten

```text
ui -> app -> domain
app -> terminal/config/profiles/ai interfaces
terminal/config/profiles/ai implementations -> domain
infrastructure -> externe Bibliotheken
```

## Nicht erlaubt

- `domain -> ui`
- `domain -> VTE`
- `ai -> GTK`
- `profiles -> VTE`
- direkte HTTP-Aufrufe aus Widgets
- Konfigurationsschreibvorgänge aus beliebigen GTK-Callbacks
- gemeinsame globale mutable Service-Sammlung

## Sichtbarkeit

- standardmäßig privat
- `pub(crate)` nur bei echter modulübergreifender Nutzung
- öffentliche API klein und absichtsvoll
- interne Implementierungstypen nicht unnötig exportieren

## Wachstum

Neue Module entstehen nur bei klarer Verantwortlichkeit. Ein Verzeichnis mit nur einer dünnen Datei ist nicht automatisch bessere Architektur. Ebenso werden große Dateien geteilt, sobald mehrere unabhängige Verantwortlichkeiten erkennbar sind.

## Prüfung

Bei jeder neuen Abhängigkeit zwischen Modulen fragen:

1. Verläuft sie in die vorgesehene Richtung?
2. Leakt ein Frameworktyp über eine Grenze?
3. Kann ein fachlicher Typ oder ein Interface die Kopplung reduzieren?
4. Ist die Abstraktion heute nötig oder nur hypothetisch?