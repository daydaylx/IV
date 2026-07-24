# ADR-001: Workspace-Modul und Persistenz für Phase 2

- Status: akzeptiert; Domänen-, Storage- und Produktanbindung (UI-Dialog, Shortcuts, Auto-Restore & Flush) umgesetzt
- Datum: 2026-07-24
- Verantwortlich: IV-Implementierung Phase 2

## Problem

Phase 2 der Roadmap verlangt Startprofile und die Wiederherstellung des
letzten Layouts. Beide Funktionen benötigen persistenten Zustand jenseits
der bestehenden Einstellungsdatei `~/.config/iv/config.toml`. Es muss
verbindlich entschieden werden, wo dieser Zustand lebt, wie er strukturiert
ist und welche Validierungs- und Sicherheitsregeln gelten.

## Rahmenbedingungen

- Sprache Rust, GTK4 + libadwaita, VTE hinter `terminal/`
- Persistenzformat TOML (`Cargo.toml` nutzt `toml = "0.8"` und `serde`)
- Keine neuen direkten Abhängigkeiten (`DEPENDENCY_POLICY.md`)
- GTK-Hauptthread darf nicht blockieren (`AGENTS.md`,
  `PERFORMANCE_BUDGET.md`)
- Wiederherstellung darf keine laufenden Prozesse wiederbeleben
  (`KNOWN_LIMITATIONS.md`, `STATE_MODEL.md`)
- API-Schlüssel und andere Secrets bleiben außerhalb der Profile
  (`SECURITY.md`)
- Beschädigte Konfiguration darf den Programmstart nicht verhindern
  (`ROADMAP.md` Phase 2)

## Optionen

### Option A: Erweiterung von `settings/`

Profile und Layouts werden in dieselbe `config.toml` integriert, die
bereits Schrift und Theme enthält.

Vorteile: ein zentraler Pfad, geringe Codeänderung, kein neues Modul.
Nachteile: `SettingsLoadOutcome` wächst weiter; Konfigurationsdatei wird
mit Phase 3 (KI) noch unübersichtlicher; Schreibkonflikte zwischen
Schrift/Theme und Layout beim asynchronen Speichern.

### Option B: Getrennte Dateien im bestehenden `settings/`-Modul

`config.toml` bleibt für Schrift und Theme, neue Dateien
`profiles.toml` und `layout.toml` im selben Verzeichnis, verwaltet durch
`settings/`.

Vorteile: saubere Trennung der Domänen.
Nachteile: `settings/` ist nach Name und Verantwortung das falsche Modul
für Profile; Architekturziel aus `ARCHITECTURE.md` wird nicht umgesetzt.

### Option C: Neues Modul `workspace/` mit eigener Verzeichnisstruktur

Eigenes Modul gemäß Architekturziel mit dediziertem Verzeichnis
`$XDG_CONFIG_HOME/iv/workspace/` für Profile, Layouts und zukünftige
Arbeitszustände. ADR dokumentiert die Trennung.

Vorteile: entspricht `ARCHITECTURE.md`; bereitet Phase 3 (KI) und
künftige Arbeitszustände vor; klare Modulgrenzen; keine
Vermischung mit Schrift/Theme.
Nachteile: höherer Initialaufwand; neues Modul muss eingeführt und gegen
`settings` abgegrenzt werden.

## Entscheidung

Option C wird umgesetzt.

- Neues Modul `src/workspace/` mit Dateien `mod.rs`, `error.rs`,
  `profile.rs`, `start_config.rs`, `layout.rs`, `storage.rs`.
- Verzeichnis `$XDG_CONFIG_HOME/iv/workspace/` mit den Dateien
  `profiles.toml` und `layout.toml`.
- `settings/` bleibt ausschließlich für Schrift und Theme zuständig.
- Profile und Layouts werden **nicht** untereinander oder mit `settings/`
  vermischt.

Zusätzlich werden die folgenden Detailentscheidungen Teil dieser ADR:

1. **Schema-Versionierung:** Beide TOML-Dateien tragen ein Top-Level
   `schema_version = 1`. Eine unbekannte Version führt zu
   `WorkspaceWarning::UnsupportedVersion` und sicherem Fallback; die
   Originaldatei wird in diesem Zustand **nicht** überschrieben.

2. **Atomares Schreiben:** Schreibvorgänge ersetzen die Zieldatei über
   `gio::File::replace_contents_future` mit Backup-Flag. GIO übernimmt
   das atomare Ersetzen; IV verwaltet keine eigene temporäre Datei.
   Unbekannte Schema-Versionen und beschädigte vorhandene Dateien werden
   vor dem Schreiben abgewiesen und nicht überschrieben.

3. **Asynchrones I/O:** Laden und Speichern laufen über
   `gio::File::load_contents_future` und
   `replace_contents_future`. Es wird **keine** `tokio`/`async-std`
   Runtime eingeführt; GTK's eigene Future-Infrastruktur reicht.

4. **Debounce:** Layout-Speicherung wird debounced (eine Sekunde) auf
   Strukturänderungen (Tab hinzufügen/schließen, Pane teilen/schließen,
   Profilwechsel, Tab-Titel ändern). Beim Shutdown koordiniert die App
   einen finalen asynchronen Flush, ohne Dateiarbeit im UI-Thread zu blockieren.

5. **Pfadvalidierung:** Eine eigene `validate_path(&str) -> Result<PathBuf, _>`
   prüft nur Schema (absolut, `..` abgewiesen, `~` als Präfix erlaubt).
   **Keine** Existenzprüfung beim Anlegen. Existenz wird ausschließlich
   beim Anwenden geprüft; fehlt das Verzeichnis, fällt die Anwendung auf
   das ursprüngliche `initial_working_directory` des Panes (oder HOME)
   zurück und blendet eine sichtbare Warnung ein.

6. **Layout ohne laufende Prozesse:** `LayoutSnapshot` enthält
   ausschließlich Struktur- und Startkonfigurationsdaten. Beim Restore
   wird `TabCollection` neu aufgebaut; jedes Pane wird über
   `Terminal::start_with(LaunchConfig)` frisch gestartet.

7. **`active_profile_id` getrennt:** Im Layout-Snapshot wird das aktive
   Profil **getrennt** vom Tab-Baum gespeichert. Wird das aktive Profil
   gelöscht, fallen die Tabs auf „kein Profil" zurück, ohne dass der
   Tab-Baum verändert wird.

8. **Strukturierte Befehle:** `StartConfig::command` ist
   `Option<Vec<String>>` (Programm + Argumente). Es wird **kein**
   `/bin/sh -c` als Wrapper verwendet.

9. **Trennung PaneTree / StartConfig:** `PaneTree` und `PaneNode` bleiben
   unverändert. `StartConfig` lebt im `workspace/`-Modul. `TabInfo`
   hält nur `custom_title: Option<String>` und
   `start_config: Option<StartConfig>`. Damit bleibt die Trennung
   zwischen Pane-Zustand und Startkonfiguration aus `STATE_MODEL.md`
   gewahrt.

10. **UI für Anlage und Auswahl:** Ein HeaderBar-Menü „Profil" mit
    Anlegen / Öffnen / Kein Profil sowie ein
    `AdwDialog`-basiertes Formular für Anlegen und Bearbeiten.
    Schnellkürzel `Alt+1`…`Alt+9` für die ersten neun Profile in
    alphabetischer Reihenfolge. `Alt+0` bleibt unbelegt. Keine
    Kommandopalette.

## Auswirkungen

- **Neues Modul `src/workspace/`**: keine GTK-, VTE- oder
  `PaneTree`-Abhängigkeiten nach unten; kennt nur Datenstrukturen und
  `gio::File`.
- **`src/terminal/`**: `LaunchConfig` wird parametrisiert (Programm,
  Args, Verzeichnis). Neuer Konstruktor `LaunchConfig::for_pane`.
  `Terminal::start_with(config)` ergänzt; `Terminal::start()` ruft
  intern `start_with(from_environment())`. `from_environment` bleibt
  rückwärtskompatibel.
- **`src/tab/`**: `TabInfo` erhält `custom_title: Option<String>` und
  `start_config: Option<StartConfig>`. Methoden `set_custom_title`,
  `set_start_config`, Lesemethoden für Snapshot.
- **`src/ui/`** (noch offen): HeaderBar wird durch ein „Profil"-Menü ergänzt;
  Profilanlage-Dialog (`AdwDialog`) wird hinzugefügt; bestehende
  modulare UI-Struktur (`window.rs`, `actions.rs`, `search.rs`,
  `links.rs`) wird durch eine neue `profile.rs` oder durch Erweiterung
  von `actions.rs`/`window.rs` ergänzt.
- **`src/app/`** (noch offen): Neue `startup.rs` für die asynchrone Lade-Sequenz
  (Workspace → Profile → Layout → UI) mit Fallback auf Phase-1-Verhalten
  und sichtbarer Warnung in der Statuszeile. `Shutdown`-Sequenz mit
  finalem Flush.
- **`Cargo.toml`**: keine neuen direkten Abhängigkeiten.
- **Dokumentation**: `docs/PROJECT_STATE.md` und
  `docs/CONTEXT_LEDGER.md` werden nach Abschluss aktualisiert.
  `docs/INDEX.md` bleibt unverändert (Modul ist über `ARCHITECTURE.md`
  erreichbar).

## Verifikation

- ADR-Status „akzeptiert" und Niederschrift in `docs/decisions/001-…`
- Modul `src/workspace/` kompiliert eigenständig ohne GTK-/VTE-Referenzen
  (Verifikation per `cargo build` + manueller Sichtung der `use`-Blöcke)
- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo test --all-targets --all-features`, `cargo build` lokal grün
- bestehende Phase-1-Tests bleiben grün
- Unit-Tests für `validate_path` (Schema-only) und
  `apply_profile_to_pane` (Existenzprüfung beim Anwenden) bestehen
- Sicherheitsreview gemäß `agents/security-reviewer.md`
- Manuelle Desktop-Prüfung auf Wayland (primär) und X11 (sekundär)

## Rückbau

- `src/workspace/` kann als Modul entfernt werden, ohne `settings/`,
  `pane/`, `terminal/` oder die UI-Schicht zu beschädigen.
- `TabInfo` enthält nach Rückbau keine Profile-bezogenen Felder mehr
  (`custom_title` und `start_config` werden entfernt).
- `LaunchConfig::for_pane` wird aus dem `terminal/`-Modul entfernt; die
  bestehende `from_environment`-Variante reicht für Phase-1-Verhalten.
- `$XDG_CONFIG_HOME/iv/workspace/` wird nicht mehr angelegt; ein
  bestehendes Verzeichnis bleibt liegen und wird ignoriert.
- Verloren geht ausschließlich die Profil- und Layout-Persistenz; alle
  Phase-1-Funktionen bleiben funktionsfähig.

## Nicht entschieden

- Verschlüsselung der Konfigurationsdateien
- Weitere Persistenzformate (z. B. JSON) oder alternative
  Konfigurations-Backends
- Synchronisation über mehrere Maschinen
- Migrationspfad für eine zukünftige `schema_version = 2`
- Tastenkürzel-Reihenfolge der Profile in den ersten neun Plätzen
  (wird in T12 final entschieden, derzeit alphabetische Reihenfolge
  angenommen)
