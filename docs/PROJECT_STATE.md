# Aktueller Projektstand

> Diese Datei beschreibt den tatsächlichen Stand des Repositories. Zielbilder und spätere Phasen stehen in den jeweiligen Fachdokumenten und in der Roadmap.

## Status

- **Projektstatus:** Phase-1-Tab-Container implementiert und kompiliert
- **Aktuelle Phase:** Phase 1 – Terminal-MVP (Tab-Container fertig; Splits, Suche, Links, Einstellungen noch offen)
- **Implementierungsstand:** lauffähiger Rust-/GTK4-/libadwaita-/VTE-Anwendungskern mit mehreren Terminal-Tabs
- **Primäre Plattform:** Linux mit Wayland
- **Terminal-Backend im MVP:** VTE
- **Letzte Dokumentprüfung:** 25. Juli 2026

## Implementiert

- Cargo-Projekt mit Rust 2024 und deklarierter Mindestversion Rust 1.92
- native libadwaita-Anwendung mit Anwendungs-ID `io.github.daydaylx.IV`
- Tab-Container (`gtk::Notebook`) mit mehreren unabhängigen Terminal-Tabs
- Tab-Domänenschicht (`TabCollection`, `TabId`, `TabInfo`) ohne GTK-/VTE-Abhängigkeiten
- neuer Tab: `Ctrl+Shift+T`
- Tab schließen: `Ctrl+Shift+W`; letzter Tab schließt das Fenster
- Tab-Wechsel: `Ctrl+PageDown` / `Ctrl+PageUp` (zyklisch)
- Tab-Titel wird aus VTE `window_title_changed` abgeleitet
- Fokus automatisch auf Terminal nach Tab-Erstellen und Tab-Wechsel
- ein Hauptfenster mit `adw::HeaderBar`, `gtk::Notebook` und Statuszeile
- lokale Shell aus einer gültigen, absoluten und ausführbaren `SHELL`-Variable
- `/bin/sh` als validierter Fallback mit sichtbarer Warnung
- validiertes aktuelles Startverzeichnis mit Fallback auf `HOME` und anschließend `/`
- asynchroner VTE-Spawn mit zehn Sekunden Timeout und abbrechbarem Start
- normale Terminalein- und -ausgabe, GTK/VTE-Resize, Auswahl, Copy und Paste
- Copy/Paste-Aktionen `Ctrl+Shift+C` und `Ctrl+Shift+V` (wirken auf aktiven Tab)
- Fokus auf dem Terminal nach dem Programmstart
- typisierte Start-, Spawn- und Clipboardfehler mit kurzen Nutzertexten
- `exit 0` im letzten Tab schließt das Fenster; in anderen Tabs wird nur der Tab geschlossen
- Signal oder Nichtnullstatus bleibt sichtbar und deaktiviert weitere Eingabe
- nichtblockierender Fensterabschluss: alle Tabs erhalten `SIGHUP`, nach 1,5 s `SIGKILL`, nach 2,5 s endgültige UI-Freigabe
- VTE-Typen ausschließlich im privaten Adapter `terminal/vte_backend.rs`

## Geprüft

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --all-features` (17 Tests: 12 TabCollection + 5 Terminal)
- `cargo build`
- Unit-Tests für `TabCollection`: hinzufügen, entfernen, aktiv setzen, next/prev, Randfälle
- nativer Start in einer Linux-X11-Desktop-Sitzung mit bash (Phase 0)
- ungültige `SHELL`-Variable und sichtbarer `/bin/sh`-Fallback (Phase 0)
- Tastatureingabe und Terminalausgabe (Phase 0)
- Unicode, Emoji und breite CJK-Zeichen (Phase 0)
- Fenster- und Terminal-Resize von 1022 × 722 auf 780 × 540 Pixel (Phase 0)
- Auswahl sowie Copy und Paste über die vorgesehenen Tastenkürzel (Phase 0)
- interaktive Nutzung von `less`, `htop` und Vim über `vi` (Phase 0)
- regulärer Shell-Exit mit Status 0 (Phase 0)
- sichtbarer Nichtnullstatus 130 ohne unkontrollierten Anwendungsabbruch (Phase 0)
- Fensterschließen während laufender und verschachtelter Shell-Prozesse (Phase 0)
- anschließend keine verbleibenden IV-, Shell- oder Zombie-Prozesse (Phase 0)

Die manuellen Desktopprüfungen liefen unter X11. Wayland bleibt die primäre Zielumgebung, wurde in dieser Umgebung aber nicht manuell ausgeführt.
Die neuen Tab-Funktionen (T10) benötigen manuelle Desktop-Prüfung vor Abschluss.

## Noch offen

- manuelle Wiederholung der Phase-0- und Phase-1-Desktopprüfungen unter Wayland
- ein automatisierter echter VTE-/PTY-Integrationstest
- reproduzierbare Prüfung des seltenen `SIGKILL`-Timeoutpfads
- Splits (horizontal/vertikal)
- Suche im Scrollback, anklickbare Links
- Schrift- und Theme-Einstellungen
- Startprofile und Layoutpersistenz
- KI-Provider, Streaming oder Kontextfilter
- Vollbild
- reproduzierbare Performance-Messungen
- Paketierung oder Releaseprozess

## Nächster technischer Schritt

Splits (Phase 1, nächster Teil): Horizontale und vertikale Teilung eines Tabs in mehrere Panes, Pane-Fokuswechsel, Pane-Größenänderung. Danach Suche, Links und Einstellungen.

## Regel zur Aktualisierung

Diese Datei wird aktualisiert, wenn:

- ein Meilenstein beginnt oder abgeschlossen wird,
- ein bisher nur geplantes Modul tatsächlich implementiert ist,
- sich die aktuelle Priorität ändert,
- eine bestätigte Einschränkung den nächsten Schritt beeinflusst.

Roadmap und Fachdokumente dürfen zukünftige Zustände beschreiben. Diese Datei bleibt dagegen bewusst kurz und gegenwartsbezogen.
