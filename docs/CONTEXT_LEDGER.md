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

## Nicht-Ziele
- Startprofile oder Layout-Persistenz
- Startprofile oder Layout-Persistenz
- KI-Integration
- Sitzungs-Wiederherstellung
- Tab-Umordnung per Drag-and-Drop (kann später folgen)
- Vollbild
- Pane-Größenänderung per Tastatur (nur Maus-Drag über `gtk::Paned`-Handle)
- Drag-and-Drop von Panes zwischen Tabs

## Bekannte Einschränkungen
- VTE ist das einzige Terminal-Backend im MVP
- Wayland primäre Zielplattform; X11 nur soweit GTK4/VTE es tragen
- `Alt+Arrow` für Fokusnavigation kollidiert potenziell mit TUI-Anwendungen (bash, vim, tmux, htop)
- `Ctrl+Shift+Q` für Pane-Schließen kann mit manchen Anwendungen kollidieren

## Offene Risiken
- `gtk::Paned`-Kompatibilität mit VTE bei tief verschachtelten Bäumen
- Fokuszuverlässigkeit nach Split-/Close-Operationen (Fokus muss auf VTE-Widget landen)
- Shortcut-Konflikte mit Terminalprogrammen (insb. Alt+Arrow, Ctrl+Shift+Q)
- Prozess-Lebenszyklus bei Pane-Schließen: `request_close` wird pro Pane-Terminal aufgerufen

## Offene Fragen
- (keine Einträge)

## Wichtige Projektregeln
- (keine Einträge)

## Aktuelle Prioritäten
- Phase 1: Schrift- und Theme-Einstellungen abgeschlossen
- Nächster Schritt: Startprofile und Layoutpersistenz (Phase 2)
- Danach: KI-Assistent (Phase 3)

## Verworfene Optionen
- `adw::TabView` + `adw::TabBar` (zugunsten `gtk::Notebook`, einfacher)
- Neuen leeren Tab automatisch öffnen beim Schließen des letzten (zugunsten Fenster schließen)
- Schließen des letzten Tabs verhindern (zugunsten Fenster schließen)

<!-- CONTEXT-LEDGER-META: {"schemaVersion":1,"lastCheckpoint":"2026-07-25T00:00:00.000Z","lastTrigger":"work-complete","planHash":"96a5c75dc99889d35101cb321f668c109bf830524a4c36eaea9dafbe2695d377"} -->
