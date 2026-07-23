# Definition of Done

Eine Aufgabe ist erst abgeschlossen, wenn alle zutreffenden Punkte erfüllt sind.

## Allgemein

- Umfang entspricht README und AGENTS.md.
- Keine IDE-Funktion oder ungeplante Erweiterung wurde eingeschleust.
- Keine Platzhalter, toten Codepfade oder unnötigen Abhängigkeiten.
- Fehlerfälle sind sichtbar und verständlich behandelt.
- Öffentliche Schnittstellen und ungewöhnliche Entscheidungen sind dokumentiert.

## Codequalität

- `cargo fmt --check` erfolgreich.
- `cargo clippy --all-targets --all-features -- -D warnings` erfolgreich.
- `cargo test --all-targets` erfolgreich.
- Neue Zustandslogik besitzt passende Tests.
- Unsichere Blöcke sind begründet, klein und dokumentiert.

## Terminaländerungen

- Shell startet und beendet sich sauber.
- Kein Zombie-Prozess nach Pane- oder Fensterschließung.
- Resize und Fokus funktionieren.
- Copy/Paste und Auswahl bleiben intakt.
- Unicode und breite Zeichen wurden geprüft.
- `vim`, `less` und `htop` funktionieren weiterhin.
- Längere Ausgabe friert die UI nicht ein.

## Tabs und Splits

- Fokus folgt eindeutig dem aktiven Pane.
- Schließen eines Panes erzeugt keinen ungültigen Split-Baum.
- Größenverhältnisse bleiben gültig.
- Tastaturbedienung funktioniert ohne Maus.
- Letztes Pane/letzter Tab wird kontrolliert behandelt.

## KI-Funktionen

- KI ist optional; Terminal funktioniert ohne Konfiguration und Netzwerk.
- Gesendeter Kontext ist vorab sichtbar.
- Secrets werden maskiert oder ausgeschlossen.
- API-Schlüssel liegen nur im System-Keyring.
- Streaming ist abbrechbar.
- Fenster- oder Pane-Schließung hinterlässt keine unkontrollierte Anfrage.
- Befehle werden niemals automatisch ausgeführt oder mit Enter bestätigt.

## UI und Barrierearmut

- Funktion ist vollständig per Tastatur erreichbar.
- Fokuszustand ist sichtbar.
- Kleine Fenstergrößen und lange Titel brechen das Layout nicht unkontrolliert.
- Terminalfläche bleibt visuell dominant.
- Keine unnötigen Animationen oder permanenten Panels.

## Abschlussbericht

Der Agent nennt:

1. geänderte Dateien und Verhalten,
2. ausgeführte Prüfungen,
3. nicht automatisch prüfbare Punkte,
4. verbleibende Risiken,
5. bewusst nicht umgesetzte Erweiterungen.
