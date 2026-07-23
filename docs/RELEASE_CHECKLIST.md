# Release-Checkliste

## Geltungsbereich

Diese Checkliste gilt für testbare Zwischenversionen und spätere MVP-Releases. Ein privates Projekt benötigt trotzdem reproduzierbare Freigabekriterien.

## Codequalität

- [ ] `cargo fmt --check` erfolgreich
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` erfolgreich
- [ ] `cargo test --all-targets --all-features` erfolgreich
- [ ] keine unbegründeten `unwrap()` oder `expect()` in Laufzeitpfaden
- [ ] keine neuen blockierenden Arbeiten im GTK-Hauptthread

## Terminalfunktion

- [ ] bash und zsh starten
- [ ] Copy und Paste funktionieren
- [ ] Resize funktioniert ohne Darstellungsfehler
- [ ] Unicode und breite Zeichen geprüft
- [ ] `vim`/`nvim`, `less` und `htop` geprüft
- [ ] SSH geprüft
- [ ] tmux und zellij geprüft
- [ ] Pi Agent geprüft
- [ ] Codex CLI oder Claude Code geprüft
- [ ] keine Zombie-Prozesse nach wiederholtem Schließen

## Tabs und Splits

- [ ] Tabs erstellen, wechseln und schließen
- [ ] horizontale und vertikale Splits
- [ ] verschachtelte Splits
- [ ] Fokusnavigation
- [ ] Pane-Größe ändern
- [ ] letztes Pane und letzter Tab sicher behandelt
- [ ] neues Pane im gewünschten Verzeichnis

## Konfiguration und Persistenz

- [ ] Start ohne Konfigurationsdatei
- [ ] beschädigte Konfiguration führt zu sicheren Standards
- [ ] Profile und Layouts laden und speichern
- [ ] ungültige Pfade verständlich behandelt
- [ ] keine Secrets in gespeicherten Dateien

## KI, sofern enthalten

- [ ] Terminal funktioniert ohne Netzwerk und Provider
- [ ] API-Key ausschließlich im Keyring
- [ ] Kontextvorschau vor dem Senden
- [ ] bekannte Secret-Muster ausgeschlossen oder gewarnt
- [ ] Streaming und Abbruch
- [ ] Schließen eines Panes bricht zugehörige Anfrage ab
- [ ] Vorschläge werden nicht automatisch ausgeführt
- [ ] Einfügen simuliert kein Enter
- [ ] Logs enthalten keine Prompts, Antworten oder Schlüssel

## Performance und Stabilität

- [ ] Leerlauf verursacht keine auffällige CPU-Last
- [ ] lange Ausgabe friert die UI nicht ein
- [ ] mehrere Tabs und Splits geprüft
- [ ] wiederholtes Öffnen und Schließen ohne auffälliges Speicherwachstum
- [ ] KI-Streaming beeinflusst Terminaleingabe nicht
- [ ] Langzeitsitzung durchgeführt

## UX und Barrierefreiheit

- [ ] Kernfunktionen vollständig per Tastatur
- [ ] Fokus jederzeit erkennbar
- [ ] ausreichender Kontrast
- [ ] wichtige Zustände nicht nur über Farbe
- [ ] reduzierte Bewegung und System-Theme respektiert
- [ ] Fehler verständlich und nicht unnötig modal

## Dokumentation

- [ ] README entspricht tatsächlichem Stand
- [ ] Roadmap aktualisiert
- [ ] bekannte Einschränkungen aktualisiert
- [ ] Architekturänderungen als ADR dokumentiert
- [ ] Konfiguration und Tastenkürzel stimmen mit Implementierung überein
- [ ] Versionshinweise oder Changelog ergänzt

## Sicherheit

- [ ] Security-Review für relevante Änderungen
- [ ] keine sensiblen Werte in Logs oder Fehlermeldungen
- [ ] TLS-Prüfung wird nicht umgangen
- [ ] Links und Endpunkte validiert
- [ ] Prozess- und Tasklebenszyklen geprüft

## Abschluss

Eine Version wird nicht als stabil bezeichnet, wenn kritische Fehler, Prozesslecks, automatische KI-Ausführung oder eine Abhängigkeit des Terminalkerns vom Netzwerk bekannt sind.