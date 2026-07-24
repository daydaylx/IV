# Konfiguration

## Ziel

IV verwendet eine kleine, nachvollziehbare TOML-Konfiguration mit sicheren Standardwerten. Die Anwendung muss auch bei fehlender oder teilweise fehlerhafter Datei starten können.

## Speicherorte

Die implementierte Datei liegt unter `$XDG_CONFIG_HOME/iv/config.toml`. Ohne gesetztes `XDG_CONFIG_HOME` verwendet GLib den XDG-Standardpfad, üblicherweise `~/.config/iv/config.toml`.

Die Datei wird beim Start asynchron über GIO geladen. Fehlt sie, startet IV ohne Meldung mit Standardwerten. Lese-, TOML- und Validierungsfehler werden ohne Dateiinhalt in einer kurzen Statusmeldung angezeigt.

## Grundregeln

- keine API-Schlüssel in TOML
- unbekannte Felder erzeugen höchstens eine Warnung
- ungültige Werte werden einzeln ersetzt, statt die ganze Datei abzulehnen
- Konfigurationsänderungen sind versionierbar
- künftige Schemaänderungen benötigen Migration oder eine verständliche Ablehnung
- keine automatische zerstörende Überschreibung beschädigter Dateien

## Implementiertes Schema

```toml
[font]
family = "monospace"
size = 12.0

[appearance]
theme = "system" # "system", "light" oder "dark"
```

Die Schriftgröße muss endlich sein und zwischen 6 und 72 Punkt liegen. Ungültige Felder fallen einzeln auf `monospace`, 12 Punkt beziehungsweise das Systemfarbschema zurück. Unbekannte Felder werden derzeit ignoriert.

Terminal-, Verhaltens-, KI-, Tastenkürzel- und Profilbereiche sind noch nicht implementiert.

## Secrets

Der API-Key wird über eine stabile Keyring-Kennung referenziert. Die Konfiguration enthält nur nicht geheime Providerdaten wie Endpunkt und Modellkennung.

## Validierung

Aktuell validiert:

- nicht leere Schriftfamilie
- Schriftgröße im Bereich 6 bis 72 Punkt
- erlaubte Theme-Werte
- TOML-Struktur und Feldtypen

Weitere Werte wie Scrollback-Limit, Shell, Provider-URL, Tastenkürzel, Profile und Layouts werden erst validiert, wenn diese Konfigurationsbereiche implementiert werden.

## Laufzeitänderungen

Beim Anwendungsstart auf alle vorhandenen und danach neu erstellten Panes angewendet:

- Theme
- Schriftgröße
- Schriftfamilie

Eine Laufzeitbeobachtung der Datei und eine Einstellungsoberfläche sind nicht implementiert; Änderungen werden nach einem Neustart wirksam.

## Profile und Layouts

Das `workspace`-Modul enthält versionierte, validierte Datenmodelle und asynchronen Storage für `$XDG_CONFIG_HOME/iv/workspace/profiles.toml` und `layout.toml`. Unbekannte Schema-Versionen oder beschädigte Dateien werden beim Speichern nicht überschrieben.

Diese Grundlage ist beim Start über `app::startup::bootstrap_workspace` angebunden. Profile können über den UI-Dialog (`ui::profile.rs`) angelegt sowie per `Alt+1`…`Alt+9` oder HeaderBar-Menü ausgewählt werden. Layout-Snapshots werden automatisch debounced gespeichert und beim Fensterabschluss geflusht.

Startprofile werden getrennt von globalen Einstellungen modelliert und enthalten nur:

- ID und Name
- Startverzeichnis
- optionale Shell
- optionaler Startbefehl
- optionales Layout
- optional bevorzugtes KI-Modell

Profile enthalten keine Secrets.

## Fehlerbehandlung

Bei Parse- oder Validierungsfehlern:

1. betroffene Fehlerklasse bestimmen, ohne Dateiinhalt zu protokollieren
2. sichere Standardwerte pro Feld verwenden
3. Anwendung starten
4. Nutzer über die Statuszeile hinweisen
5. Originaldatei nicht überschreiben

## Tests

- fehlende Datei
- leere Datei
- gültige Datei
- unbekannte Felder
- ungültige Typen
- Grenzwerte
- beschädigtes Gesamtdokument

Alte Schemaversionen, Profile und Serialisierung werden erst mit den entsprechenden Funktionen testpflichtig.
