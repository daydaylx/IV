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

## Bekannte Einschränkungen
- VTE ist das einzige Terminal-Backend im MVP
- Wayland primäre Zielplattform; X11 nur soweit GTK4/VTE es tragen
- `Alt+Arrow` für Fokusnavigation kollidiert potenziell mit TUI-Anwendungen (bash, vim, tmux, htop)
- `Ctrl+Shift+Q` für Pane-Schließen kann mit manchen Anwendungen kollidieren

## Offene Risiken
- `gtk::Paned`-Kompatibilität mit VTE bei tief verschachtelten Bäumen
- Fokuszuverlässigkeit nach Split-/Close-Operationen (Fokus muss auf VTE-Widget landen)
- Shortcut-Konflikte mit Terminalprogrammen (insb. Alt+Arrow, Ctrl+Shift+Q)
- Wayland- und Langzeitprüfung der Pane- und Prozess-Lebenszyklen steht aus

## Offene Fragen
- (keine Einträge)

## Wichtige Projektregeln
- (keine Einträge)

## Aktuelle Prioritäten
- Phase-1-Desktop- und Langzeitprüfung unter Wayland
- anschließend Workspace-Grundlage an Startsequenz, Terminalstart und Profil-UI anbinden

## Verworfene Optionen
- `adw::TabView` + `adw::TabBar` (zugunsten `gtk::Notebook`, einfacher)
- Neuen leeren Tab automatisch öffnen beim Schließen des letzten (zugunsten Fenster schließen)
- Schließen des letzten Tabs verhindern (zugunsten Fenster schließen)

<!-- CONTEXT-LEDGER-META: {"schemaVersion":1,"lastCheckpoint":"2026-07-24T07:00:22.521Z","lastTrigger":"plan-to-work","planHash":"d7e21bd3476f8886d570e21ffa3501a1118c3ac5a1824bc0690577787f1d1e0a"} -->
