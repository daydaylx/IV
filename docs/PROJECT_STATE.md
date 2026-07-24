# Aktueller Projektstand

> Diese Datei beschreibt den tatsächlichen Stand des Repositories. Zielbilder und spätere Phasen stehen in den jeweiligen Fachdokumenten und in der Roadmap.

## Status

- **Projektstatus:** Phase-2-Funktionskern (Startprofile und Auto-Restore) implementiert; UI-Integration und Akzeptanztests laufen
- **Aktuelle Phase:** Phase 2 – Startprofile und Layouts
- **Implementierungsstand:** lauffähiger Rust-/GTK4-/libadwaita-/VTE-Anwendungskern mit Tabs, verschachtelten Pane-Splits, Suche, Links, Einstellungen, Startprofilen und Layout-Persistenz
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
- stabile Notebook-Seiten; strukturelle Pane-Änderungen verwenden dieselben VTE-Widgets ohne Doppelstart oder Reparenting-Warnung
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
- mehrere gleichzeitige Schließanforderungen werden bis zum tatsächlichen Prozessende gemeinsam abgeschlossen
- VTE-Typen ausschließlich im privaten Adapter `terminal/vte_backend.rs`
- Pane-Domänenschicht in `pane/` (`tree.rs`, `node.rs`, `navigation.rs`) ohne GTK-/VTE-Abhängigkeiten
- Pane-Tree mit Terminal-Blättern und stabil identifizierten Split-Knoten (Horizontal/Vertikal)
- horizontaler Split: `Ctrl+Shift+Right`
- vertikaler Split: `Ctrl+Shift+Down`
- Pane schließen: `Ctrl+Shift+Q`; letztes Pane schließt den Tab
- Pane-Fokuswechsel: `Alt+Left/Right/Up/Down` (nicht-zyklisch)
- visuelle Hervorhebung des aktiven Panes via CSS-Klasse `active-pane`
- rekursiver Widget-Baum aus `PaneTree`: `gtk::Paned`-Container, Pane-Resize per Drag-Handle
- Split-Verhältnisse werden im Zustandsmodell aktualisiert und auf 0,05 bis 0,95 begrenzt
- PaneTree-Invarianten: kein leeres Split, Hochziehen nach Schließen, aktives Pane immer gültig
- Suche im Scrollback via `Ctrl+Shift+F` mit `gtk::SearchBar` und `gtk::SearchEntry`
- Literal-Substring-Suche mit VTE-Regex; `Enter` = nächster Treffer, `Shift+Enter` = vorheriger Treffer; `Escape` = Suche schließen
- Suche wird bei Pane-/Tab-Wechsel, Split und Schließen automatisch zurückgesetzt
- Suchleiste räumt per `search_set_regex(None)` sauber auf; nach Schließen Fokus auf Terminal
- Anklickbare URLs: `https?://`-Regex-Matching via VTE `match_add_regex` + OSC-8-Hyperlinks; nur vollständige HTTP(S)-URIs mit Host werden weitergegeben
- `Ctrl+Click` auf URL öffnet diese im Standardbrowser via `gio::AppInfo::launch_default_for_uri`
- Mauszeiger wechselt zu Pointer über erkannten URLs
- asynchron über GIO geladene TOML-Einstellungsdatei unter `$XDG_CONFIG_HOME/iv/config.toml`
- Schriftart (`font.family`, Default: `monospace`) und Schriftgröße (`font.size`, Default: `12.0`)
- Farbschema (`appearance.theme`: `system`/`light`/`dark`) via `adw::StyleManager`
- feldweise Validierung mit sichtbaren, typisierten Warnungen und sicheren Standardwerten
- Schriftzoom: `Ctrl++` / `Ctrl+-` / `Ctrl+0` via VTE `font_scale` (0.5–4.0)
- getrennte UI-Module in `ui/` für Aktionen, Links, Suche, Profile, Fenster-Lifecycle, Pane-Widgets, Tabs und Terminal-Events
- testbare Phase-2-Grundlage in `workspace/`: versionierte Profile und Layout-Snapshots, Schema-/Pfadvalidierung sowie asynchrones, ersetzendes GIO-Schreiben
- Workspace-Dateien mit unbekannter Version oder beschädigtem TOML werden nicht überschrieben
- eigenes Modul `src/workspace/` mit `WorkspaceStorage` (Async-I/O via `gio::File`), `StartProfile`, `StartConfig` inkl. `validate_path` (Schema-only, keine Existenzprüfung beim Anlegen) und `LayoutSnapshot` mit `active_profile_id` getrennt vom Tab-Baum
- `LayoutDebouncer` für 1 s Debounce auf Strukturänderungen; synchroner Flush beim Schließen
- `LaunchConfig` parametrisiert (`program`, `args`, `working_directory`); `LaunchConfig::for_pane(&StartConfig)` und `Terminal::start_with(&LaunchConfig)` ergänzt
- `TabInfo` um `custom_title: Option<String>` und `start_config: Option<StartConfig>` erweitert; `TabCollection::from_tabs` für Snapshot-Wiederherstellung
- `PaneTree::from_root` validiert und rekonstruiert `next_id` / `next_split_id` aus gespeicherten IDs
- Anlege-Dialog (`gtk::Window`-basiert) für neue Profile mit Feldern Name, Verzeichnis, optionale Shell, optionaler Startbefehl und Live-Validierung
- Tastenkürzel `Alt+1`…`Alt+9` für alphabetische Schnellauswahl der ersten neun Profile, `Ctrl+Shift+N` öffnet den Anlege-Dialog
- `app::startup::bootstrap_workspace` lädt Profile und Layout asynchron beim Start; `save_layout_now` flusht synchron vor Shell-Beendigung
- `connect_close_request` ruft `save_layout_now` vor `terminal.request_close`

## Geprüft

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --all-features` (91 Unit-Tests)
- `cargo build`
- Unit-Tests für `PaneTree`: Split, Close, Fokusnavigation, Invarianten, IDs
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
- automatisierter Xvfb-Smoke-Test am 24. Juli 2026: zwei verschachtelte Splits, neuer Tab, Nichtnull-Exit, Schließen des beendeten Tabs, Suche und In-App-Fensterabschluss
- dabei erwartete Shellprozesszahlen 1 → 2 → 3 → 4 → 3 und keine GTK-, GLib-, GDK- oder VTE-Warnungen

Die manuellen Desktopprüfungen liefen unter X11. Wayland bleibt die primäre Zielumgebung, wurde in dieser Umgebung aber nicht manuell ausgeführt.
Xvfb meldete lediglich die erwarteten libEGL-Hinweise wegen fehlender DRI3-Hardwarebeschleunigung.

## Noch offen

- manuelle Desktop-Prüfung der Schrift- und Theme-Einstellungen
- manuelle Desktop-Prüfung des Schriftzooms
- manuelle Desktop-Prüfung der URL-Links (Ctrl+Click)
- manuelle Desktop-Prüfung der Suchleiste
- manuelle Desktop-Prüfung der Split-Funktionen
- manuelle Desktop-Prüfung der Profil-UI (Anlegen, Auswählen, Auto-Restore)
- Sicherheitsreview der Phase-2-Änderungen gemäß `agents/security-reviewer.md`
- Sicherheitsreview der Profil-Anlage und der TOML-Deserialisierung
- Konfliktprüfung der `Alt+1`…`Alt+9`-Tastenkürzel in bash, vim, tmux, htop
- manuelle Langzeitprüfung mit Pi Agent sowie Codex CLI oder Claude Code
- Vollbild
- manuelle Wiederholung der Phase-0- und Phase-1-Desktopprüfungen unter Wayland
- ein automatisierter echter VTE-/PTY-Integrationstest
- reproduzierbare Prüfung des seltenen `SIGKILL`-Timeoutpfads
- KI-Provider, Streaming oder Kontextfilter
- reproduzierbare Performance-Messungen
- Paketierung oder Releaseprozess

## Nächster technischer Schritt

Manuelle Desktop-Prüfung der Phase-2-Funktionen (Profil-UI, Tastenkürzel, Auto-Restore, Sicherheitsreview) und vollständige Phase-1-Desktop- und Langzeittestmatrix unter Wayland; danach Phase 3 (KI-Assistent).

## Regel zur Aktualisierung

Diese Datei wird aktualisiert, wenn:

- ein Meilenstein beginnt oder abgeschlossen wird,
- ein bisher nur geplantes Modul tatsächlich implementiert ist,
- sich die aktuelle Priorität ändert,
- eine bestätigte Einschränkung den nächsten Schritt beeinflusst.

Roadmap und Fachdokumente dürfen zukünftige Zustände beschreiben. Diese Datei bleibt dagegen bewusst kurz und gegenwartsbezogen.
