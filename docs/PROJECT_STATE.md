# Aktueller Projektstand

> Diese Datei beschreibt den tatsächlichen Stand des Repositories. Zielbilder und spätere Phasen stehen in den jeweiligen Fachdokumenten und in der Roadmap.

## Status

- **Projektstatus:** Phase-0-Prototyp implementiert und lokal geprüft
- **Aktuelle Phase:** Phase 0 – technischer Prototyp
- **Implementierungsstand:** lauffähiger Rust-/GTK4-/libadwaita-/VTE-Anwendungskern mit genau einem Terminal
- **Primäre Plattform:** Linux mit Wayland
- **Terminal-Backend im MVP:** VTE
- **Letzte Dokumentprüfung:** 24. Juli 2026

## Implementiert

- Cargo-Projekt mit Rust 2024 und deklarierter Mindestversion Rust 1.92
- native libadwaita-Anwendung mit Anwendungs-ID `io.github.daydaylx.IV`
- ein Hauptfenster mit genau einem dominanten VTE-Terminal
- lokale Shell aus einer gültigen, absoluten und ausführbaren `SHELL`-Variable
- `/bin/sh` als validierter Fallback mit sichtbarer Warnung
- validiertes aktuelles Startverzeichnis mit Fallback auf `HOME` und anschließend `/`
- asynchroner VTE-Spawn mit zehn Sekunden Timeout und abbrechbarem Start
- normale Terminalein- und -ausgabe, GTK/VTE-Resize, Auswahl, Copy und Paste
- Copy/Paste-Aktionen `Ctrl+Shift+C` und `Ctrl+Shift+V`
- Fokus auf dem Terminal nach dem Programmstart
- typisierte Start-, Spawn- und Clipboardfehler mit kurzen Nutzertexten
- genau eine Verarbeitung des Shell-Endes
- `exit 0` schließt das Fenster; Signal oder Nichtnullstatus bleibt sichtbar und deaktiviert weitere Eingabe
- nichtblockierender Fensterabschluss: `SIGHUP`, nach 1,5 Sekunden `SIGKILL`, nach insgesamt 2,5 Sekunden endgültige UI-Freigabe
- VTE-Typen ausschließlich im privaten Adapter `terminal/vte_backend.rs`

## Geprüft

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --all-features`
- `cargo build`
- nativer Start in einer Linux-X11-Desktop-Sitzung mit bash
- ungültige `SHELL`-Variable und sichtbarer `/bin/sh`-Fallback
- Tastatureingabe und Terminalausgabe
- Unicode, Emoji und breite CJK-Zeichen
- Fenster- und Terminal-Resize von 1022 × 722 auf 780 × 540 Pixel
- Auswahl sowie Copy und Paste über die vorgesehenen Tastenkürzel
- interaktive Nutzung von `less`, `htop` und Vim über `vi`
- regulärer Shell-Exit mit Status 0
- sichtbarer Nichtnullstatus 130 ohne unkontrollierten Anwendungsabbruch
- Fensterschließen während laufender und verschachtelter Shell-Prozesse
- anschließend keine verbleibenden IV-, Shell- oder Zombie-Prozesse

Die manuellen Desktopprüfungen liefen unter X11. Wayland bleibt die primäre Zielumgebung, wurde in dieser Umgebung aber nicht manuell ausgeführt.

## Noch offen

- manuelle Wiederholung der Phase-0-Desktopprüfungen unter Wayland
- ein automatisierter echter VTE-/PTY-Integrationstest
- reproduzierbare Prüfung des seltenen `SIGKILL`-Timeoutpfads
- Tabs und Splits
- Startprofile und Layoutpersistenz
- KI-Provider, Streaming oder Kontextfilter
- reproduzierbare Performance-Messungen
- Paketierung oder Releaseprozess

## Nächster technischer Schritt

In Phase 1 den vorhandenen Einzelterminal-Container in einen ersten Tab-Container überführen, ohne bereits Splits einzuführen.

## Regel zur Aktualisierung

Diese Datei wird aktualisiert, wenn:

- ein Meilenstein beginnt oder abgeschlossen wird,
- ein bisher nur geplantes Modul tatsächlich implementiert ist,
- sich die aktuelle Priorität ändert,
- eine bestätigte Einschränkung den nächsten Schritt beeinflusst.

Roadmap und Fachdokumente dürfen zukünftige Zustände beschreiben. Diese Datei bleibt dagegen bewusst kurz und gegenwartsbezogen.
