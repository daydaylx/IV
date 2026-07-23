# UX-Prinzipien

## Produktcharakter

IV ist zuerst ein Terminal-Emulator. Die Oberfläche soll ruhig, direkt, modern und tastaturorientiert sein. Zusatzfunktionen dürfen den Terminalfluss nicht dominieren.

## Hierarchie

1. Terminalinhalt
2. aktives Pane und Tab
3. notwendige Navigation
4. Prozess- und Verzeichnisstatus
5. optionale KI-Unterstützung

Das Terminal erhält standardmäßig den größten verfügbaren Bereich.

## Reduktion

- keine dauerhafte IDE-Seitenleiste
- kein Datei-Explorer
- keine Chatblasen zwischen Terminalzeilen
- keine Blockdarstellung jedes Shell-Befehls
- keine dekorativen Animationen ohne funktionalen Nutzen
- seltene Aktionen gehören in Menüs oder eine kompakte Befehlssuche
- Statusinformationen werden nur angezeigt, wenn sie technisch belegbar und nützlich sind

## Tastatur zuerst

Alle Kernfunktionen sind ohne Maus erreichbar:

- Tab erstellen, wechseln und schließen
- Pane teilen, wechseln, skalieren und schließen
- Suche öffnen und bedienen
- KI-Seitenleiste öffnen und schließen
- Vorschläge kopieren oder in die Eingabe übernehmen
- Schriftgröße und Vollbild ändern

Tastenkürzel dürfen übliche Eingaben interaktiver Terminalprogramme nicht unnötig abfangen. Konflikte werden in `docs/KEYBINDINGS.md` geregelt.

## Fokus

- Fokus ist jederzeit visuell eindeutig
- Fokuswechsel erzeugt keine überraschenden Scroll- oder Auswahländerungen
- neue Panes erhalten nach bewusster Erstellung den Fokus
- das Öffnen der KI-Seitenleiste darf laufende Terminaleingaben nicht verlieren
- nach Schließen einer temporären Oberfläche kehrt der Fokus sinnvoll zurück

## Tabs und Splits

- Splits wirken wie Terminalflächen, nicht wie Editorgruppen
- aktive und inaktive Panes sind unterscheidbar, ohne starke Rahmen oder visuelles Rauschen
- Größenänderungen erfolgen direkt und vorhersehbar
- verschachtelte Splits bleiben verständlich
- ein Pane zeigt keine unnötige eigene Werkzeugleiste

## KI-Unterstützung

Die KI ist eine optionale, einklappbare Seitenfläche.

Regeln:

- nur auf bewusste Aktion öffnen
- Kontext vor dem Senden anzeigen
- klar zwischen Terminalinhalt und KI-Antwort unterscheiden
- Streaming abbrechbar machen
- Befehlsvorschläge mit Kopieren und „in Eingabe übernehmen“ anbieten
- niemals automatisch ausführen
- keine Behauptungen über interne Modellzustände anzeigen

Zulässige Statusangaben sind technisch beobachtbar, etwa „Anfrage läuft“ oder „Seit fünf Minuten keine neue Terminalausgabe“.

## Fehlermeldungen

- kurz und handlungsorientiert
- Ursache und nächste sinnvolle Aktion nennen
- keine technischen Details erzwingen, aber optional zugänglich machen
- Terminalbetrieb nicht mit modalen Dialogen blockieren, sofern keine Entscheidung zwingend erforderlich ist
- Fehler der KI nicht als Terminalfehler darstellen

## Einstellungen

- sichere und brauchbare Standardwerte
- Einstellungen nach Aufgaben gruppieren, nicht nach internen Modulen
- Änderungen mit sofort verständlicher Wirkung dürfen direkt angewendet werden
- riskante oder schwer rückgängig zu machende Aktionen benötigen Bestätigung
- API-Schlüssel werden über den Keyring verwaltet und nicht als normaler Klartextwert behandelt

## Barrierefreiheit

- ausreichender Kontrast
- sichtbare Fokusindikatoren
- keine reine Farbcodierung für wichtige Zustände
- skalierbare Schrift und UI
- verständliche zugängliche Namen für Bedienelemente
- Systempräferenzen für Theme und reduzierte Bewegung respektieren

## Responsive Verhalten

Bei wenig Platz wird priorisiert:

1. Terminal
2. aktiver Tab und grundlegende Navigation
3. wichtige Statusinformationen
4. optionale Bedienelemente

Die KI-Seitenleiste darf überlagern oder einklappen, statt das Terminal unbrauchbar schmal zu machen.

## Konsistenz

- gleiche Aktion verwendet überall denselben Begriff
- Icons erhalten bei Unklarheit Text oder Tooltip
- Schließen, Abbrechen und Zurück folgen konsistenten Mustern
- sichtbare Zustände entsprechen dem tatsächlichen Anwendungszustand

## UX-Prüfung für neue Funktionen

Vor Implementierung beantworten:

1. Verbessert die Funktion unmittelbar Terminalarbeit?
2. Ist sie ohne dauerhafte neue Oberfläche möglich?
3. Funktioniert sie vollständig per Tastatur?
4. Bleibt das Terminal ohne diese Funktion unabhängig nutzbar?
5. Erzeugt sie IDE- oder Agenten-Komplexität?
6. Kann eine bestehende, einfachere Interaktion dasselbe Ziel erreichen?

Bei unklarem Nutzen wird die Funktion nicht in das MVP aufgenommen.