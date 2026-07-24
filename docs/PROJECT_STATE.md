# Aktueller Projektstand

> Diese Datei beschreibt den tatsächlichen Stand des Repositories. Zielbilder und spätere Phasen stehen in den jeweiligen Fachdokumenten und in der Roadmap.

## Status

- **Projektstatus:** Planung und technische Vorbereitung
- **Aktuelle Phase:** Phase 0 – technischer Prototyp
- **Implementierungsstand:** noch kein belastbarer Rust-/GTK4-/VTE-Anwendungskern im Repository
- **Primäre Plattform:** Linux mit Wayland
- **Terminal-Backend im MVP:** VTE
- **Letzte Dokumentprüfung:** 24. Juli 2026

## Bereits festgelegt

- Produktziel und klare Nicht-Ziele
- Rust, GTK4, libadwaita und VTE als MVP-Technologien
- Kapselung von VTE hinter einer internen Backend-Grenze
- fachliches Zielmodell für Tabs, Splits und Prozesse
- optionale und vom Terminalkern getrennte KI-Integration
- Sicherheits-, Logging-, Test- und Performancegrundsätze
- Agentenrollen für Architektur, Implementierung, Review, Tests und Sicherheit

Diese Festlegungen sind Planungsgrundlage. Sie belegen noch keine vollständige Implementierung.

## Noch nicht als implementiert voraussetzen

- stabile GTK4/libadwaita-Anwendung
- vollständige VTE- und PTY-Integration
- Tabs und Splits
- Startprofile und Layoutpersistenz
- KI-Provider, Streaming oder Kontextfilter
- belastbare automatisierte Testabdeckung
- reproduzierbare Performance-Messungen
- Paketierung oder Releaseprozess

Agenten dürfen diese Bereiche nicht als vorhanden behandeln, bevor Code und Tests dies belegen.

## Nächster technischer Meilenstein

Eine minimale GTK4/libadwaita-Anwendung, die genau ein VTE-Terminal einbettet und eine lokale Standardshell zuverlässig startet.

### Umfang

- minimale Rust-Projektstruktur
- Anwendungsstart und ein Fenster
- ein VTE-Terminal
- bash oder zsh starten
- Eingabe und Ausgabe
- Copy und Paste
- Resize
- kontrollierter Prozess- und Ressourcenabschluss
- verständliche Fehler bei ungültiger Shell oder ungültigem Startverzeichnis

### Nicht-Ziele dieses Meilensteins

- Tabs oder Splits
- Einstellungen und Profile
- Sitzungswiederherstellung
- KI-Integration
- alternatives Terminal-Backend
- Plugin-System
- allgemeine Zukunftsarchitektur ohne aktuellen Bedarf

### Abschlusskriterien

- bash und zsh starten zuverlässig
- `vim` oder `nvim`, `less` und `htop` sind nutzbar
- Unicode und breite Zeichen funktionieren
- Resize beschädigt die Darstellung nicht
- das Schließen hinterlässt keine Zombie-Prozesse
- Terminal- und Prozessfehler beenden die Anwendung nicht unkontrolliert

## Regel zur Aktualisierung

Diese Datei wird aktualisiert, wenn:

- ein Meilenstein beginnt oder abgeschlossen wird,
- ein bisher nur geplantes Modul tatsächlich implementiert ist,
- sich die aktuelle Priorität ändert,
- eine bestätigte Einschränkung den nächsten Schritt beeinflusst.

Roadmap und Fachdokumente dürfen zukünftige Zustände beschreiben. Diese Datei bleibt dagegen bewusst kurz und gegenwartsbezogen.
