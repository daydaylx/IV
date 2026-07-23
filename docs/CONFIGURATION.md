# Konfiguration

## Ziel

IV verwendet eine kleine, nachvollziehbare TOML-Konfiguration mit sicheren Standardwerten. Die Anwendung muss auch bei fehlender oder teilweise fehlerhafter Datei starten können.

## Speicherorte

Die endgültigen XDG-Pfade werden bei Implementierung festgelegt. Es gelten die Linux-XDG-Konventionen für Konfiguration, Daten, Cache und Logs.

## Grundregeln

- keine API-Schlüssel in TOML
- unbekannte Felder erzeugen höchstens eine Warnung
- ungültige Werte werden einzeln ersetzt, statt die ganze Datei abzulehnen
- Konfigurationsänderungen sind versionierbar
- bestehende Konfigurationen werden bei Schemaänderungen migriert oder verständlich abgelehnt
- keine automatische zerstörende Überschreibung beschädigter Dateien

## Geplante Bereiche

```toml
[appearance]
theme = "system"
font_family = "monospace"
font_size = 12.0

[terminal]
shell = "/bin/zsh"
scrollback_lines = 10000
open_links = true

[behavior]
confirm_close_running_process = true
restore_last_layout = false

[ai]
enabled = false
endpoint = "https://example.invalid/v1"
model = ""

[keybindings]
# benutzerdefinierte Überschreibungen
```

Dies ist ein Richtungsbeispiel, noch kein unveränderliches Schema.

## Secrets

Der API-Key wird über eine stabile Keyring-Kennung referenziert. Die Konfiguration enthält nur nicht geheime Providerdaten wie Endpunkt und Modellkennung.

## Validierung

Zu validieren sind mindestens:

- Schriftgröße in sinnvollem Bereich
- Scrollback-Limit
- erlaubte Theme-Werte
- gültige URL-Schemes für Provider
- existierende oder bewusst fehlende Shell
- eindeutige Tastenkürzel
- gültige Startprofilpfade
- Layoutstruktur gemäß Zustandsmodell

## Laufzeitänderungen

Sofort anwendbar:

- Theme
- Schriftgröße
- Sichtbarkeit der Statusleiste
- einige Tastenkürzel

Nur für neue Panes oder nach Neustart:

- Standardshell
- Startumgebung
- bestimmte Backendoptionen

Die UI muss kenntlich machen, wann eine Änderung wirksam wird.

## Profile

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

1. genaue Stelle und Ursache bestimmen
2. sichere Standardwerte verwenden
3. Anwendung soweit möglich starten
4. Nutzer auf die fehlerhafte Datei hinweisen
5. Originaldatei nicht ungefragt überschreiben

## Tests

- fehlende Datei
- leere Datei
- gültige Datei
- unbekannte Felder
- ungültige Typen
- Grenzwerte
- alte Schemaversion
- beschädigte Profile
- keine Secrets in Serialisierung