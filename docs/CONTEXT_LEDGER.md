# Context Ledger — IV

<!-- Dauerhaftes Projektgedächtnis. Nur bestätigte, dauerhaft relevante
     Fakten. Keine Logs, Chats, Secrets, Rohdaten. Flüchtiger
     Arbeitszustand gehört in docs/PROJECT_STATE.md. -->

## Bestätigte Nutzerentscheidungen
- (keine Einträge)

## Architekturentscheidungen
- Tabs: `gtk::Notebook` mit `TabCollection`-Domänenschicht (Phase 0, akzeptiert)
- Splits: `gtk::Paned` rekursiv gemäß `PaneTree` (Phase 1, akzeptiert)
- Letzter Tab: Fenster schließen (Option A, akzeptiert)
- Fokusnavigation: räumlich, nicht-zyklisch (akzeptiert)
- Workspace-Grundlage: getrenntes `workspace/`-Modul mit versionierten Profil-/Layoutdateien; UI-Anbindung offen (ADR-001)

## Nicht-Ziele
- Sitzungs-Wiederherstellung
- Tab-Umordnung per Drag-and-Drop (kann später folgen)
- Pane-Größenänderung per Tastatur (nur Maus-Drag über `gtk::Paned`-Handle)
- Drag-and-Drop von Panes zwischen Tabs
- Suche über mehrere Panes gleichzeitig
- Suchen und Ersetzen
- Umschalter für Groß-/Kleinschreibung (MVP: immer case-sensitive)
- Umschalter zwischen Regex und Literal (MVP: immer literal Substring-Suche)
- persistente Suchhistorie
- Trefferanzahl-Label (MVP: nur next/prev Navigation)
- Suchleiste konfigurierbar machen
- Datei-Explorer, Projektanalyse, Build-System- oder Git-Erkennung
- Kommandopalette (`Ctrl+Shift+P` o. ä.) für die Profilauswahl
- mehrere Profile parallel starten
- Wiederaufnahme laufender Prozesse über Anwendungsneustarts hinweg
- dauerhafte Speicherung von Terminalinhalten oder Scrollback
- Profilimport über URL oder externe Dateien jenseits `workspace/`
- API-Schlüssel oder andere Secrets in Profilen
- mehrere Layouts parallel; nur das jeweils letzte wird gespeichert
- Verschlüsselung der Konfigurationsdateien
- Anlegen oder Bearbeiten von Profilen über CLI (`iv --create-profile` o. ä.)
- automatische Migration von Konfiguration aus Phase 1
- Erweiterung von `PaneTree` oder `PaneNode` um Pfade, Shells oder Startbefehle
- harte Validierung der Verzeichnisexistenz beim Anlegen eines Profils
- Erweiterung von `PaneTree` oder `PaneNode` um Pfade, Shells oder
- harte Validierung der Verzeichnis-Existenz beim Anlegen eines Profils

## Bekannte Einschränkungen
- VTE ist das einzige Terminal-Backend im MVP
- Wayland primäre Zielplattform; X11 nur soweit GTK4/VTE es tragen
- `Alt+Arrow` für Fokusnavigation kollidiert potenziell mit TUI-Anwendungen (bash, vim, tmux, htop)
- `Ctrl+Shift+Q` für Pane-Schließen kann mit manchen Anwendungen kollidieren
- `Alt+1`…`Alt+9` für Profil-Schnellauswahl ist noch nicht auf TUI-Konflikte geprüft

## Offene Risiken
- `gtk::Paned`-Kompatibilität mit VTE bei tief verschachtelten Bäumen
- Fokuszuverlässigkeit nach Split-/Close-Operationen (Fokus muss auf VTE-Widget landen)
- Shortcut-Konflikte mit Terminalprogrammen (insb. Alt+Arrow, Ctrl+Shift+Q)
- Wayland- und Langzeitprüfung der Pane- und Prozess-Lebenszyklen steht aus
- `profiles.toml` mit Top-Level `schema_version = 1` und `[[profile]]`-Liste.
- `layout.toml` mit `schema_version = 1` und `[layout]`-Sektion
- `active_profile_id` ist **getrennt** vom Tab-Baum gespeichert. Wird das
- Unbekannte `schema_version` führt zu sicherem Fallback und
- prüft **nur Schema**: muss absolut sein, `~` darf nur als Präfix
- prüft **keine** reale Existenz beim Anlegen
- Existenz wird ausschließlich beim Anwenden geprüft: schlägt die
- Symlink-Loop-Erkennung erfolgt beim Anwenden mit maximaler Tiefe (16)
- HeaderBar-Menü „Profil" mit drei Einträgen: „Anlegen…", „Öffnen ▸"
- Tastenkürzel `Alt+1` bis `Alt+9` für die ersten neun Profile in stabiler
- „Kein Profil" hat bewusst kein Schnellkürzel und wird ausschließlich
- Profilanlage erfolgt über einen `AdwPreferencesPage`-basierten Dialog

## Offene Fragen
- (keine Einträge)

## Wichtige Projektregeln
- (keine Einträge)

## Aktuelle Prioritäten
- T18: Sicherheitsreview gemäß agents/security-reviewer.md
- T19: Manuelle Desktop-Prüfung:
- T22: Abschlussbericht gemäß DEFINITION_OF_DONE.md.

## Verworfene Optionen
- `adw::TabView` + `adw::TabBar` (zugunsten `gtk::Notebook`, einfacher)
- Neuen leeren Tab automatisch öffnen beim Schließen des letzten (zugunsten Fenster schließen)
- Schließen des letzten Tabs verhindern (zugunsten Fenster schließen)

<!-- CONTEXT-LEDGER-META: {"schemaVersion":1,"lastCheckpoint":"2026-07-24T07:26:49.702Z","lastTrigger":"session-shutdown","planHash":"99a4cc4a0bf2502693c3a935ba4102c651d959e6f515a63ea515692d2db91406"} -->
