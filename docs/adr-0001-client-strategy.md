# ADR 0001: Strategie für einen Aosu/H1L-"Cloud-Client"

Status: entschieden 2026-11-21 · Ziel des Nutzers: stabiler, wartbarer, legal vertretbarer Client; kein Reverse Engineering als Selbstzweck.

## Kontext (belegt)

- App = Tuya-SDK mit **Glazero-eigener Cloud** (`api.aosulife.com`), Anmeldung und Signaturen mit App-Schlüsseln in nativen Bibliotheken (`apk-findings.md`).
- Livebild lief im Test ohne LAN-Verkehr zur Kamera/H1L über Cloud-Verbindungen (HTTPS, MQTT-TLS 8883, UDP-Relay).
- Offizielle Drittanbieter-Anbindung laut App und Hersteller-Forum: **nur Alexa und Google Assistant** (Sprachsteuerung/Smart-Display). Der Hersteller schreibt im Support-Forum, Akku-Kameras unterstützten kein ONVIF/RTSP/NAS/NVR (Forenaussage, nicht offiziell geprüft).
- Keine öffentliche Entwickler-API gefunden. Kein Open-Source-Client oder HA-Integration für Aosu bekannt (HA-Community-Threads).
- Die App erzeugt Push-Benachrichtigungen mit getrennten Kanälen: `event`, `ring`, `lowpower`, `dismantle`, `call`, `default` (am Testgerät gemessen, alle aktiv).

## Bewertete Optionen

Wahrscheinlichkeiten sind meine Schätzungen, keine Messwerte.

| # | Option | Evidenz | Aufwand | Wartung | Risiko | Erfolg | Entscheidung |
|---|---|---|---|---|---|---|---|
| A | **Ereignisse über Push-Benachrichtigungen** (HA Companion App „Last notification" + Kanal-ID, optional eigener Listener) | Kanäle am Gerät belegt; Textinhalt noch nicht | sehr klein | klein (hängt an Notification-Format) | niedrig, nur eigene App und Konto | ~85 % für Ereignis/Klingel/Akku | **Ja, zuerst** |
| B | **Offizielle Anfrage an Aosu** (API-/Integrationszugang) und DSGVO-Auskunft/Datenübertragung (Art. 15, 20) | Keine öffentliche API bekannt | klein | keine | keines | ~10 % API, ~60 % irgendeine Antwort/Export | **Ja, parallel** (Entwurf in `outreach.md`) |
| C | **Basisstation lokal prüfen** (passiv, `lan_probe.py`, Forum nennt offenen Port 80 an der H1L, unbestätigt) | schwach | klein, braucht Standort | je nach Ergebnis | niedrig | ~15 % brauchbare Schnittstelle | **Ja, sobald Standort zugänglich** |
| D | App-Spiegelung (Android-Gerät mit offizieller App, Bildschirm → RTSP) für Livebild | funktioniert technisch, nicht getestet | mittel | hoch (UI-Automatisierung) | niedrig (unveränderte App) | ~60 % | Zurückgestellt, nur falls Livebild in HA zwingend |
| E | Alexa/Google als Videoweg | Video nur auf Echo Show/Nest Hub, keine Verbraucher-API | n/a | n/a | n/a | <5 % | **Verwerfen** |
| F | Tuya-Weg (Smart Life, go2rtc, Tuya-Cloud-Projekt) | Aosu nutzt eigene Cloud-Domain | klein | n/a | n/a | <10 % | **Verwerfen** |
| G | **Direkter Client gegen `api.aosulife.com`** mit aus der App gewonnenen Schlüsseln und nachgebautem Signatur-/Login-Verfahren | technisch plausibel, aber nur nach Schlüsselextraktion | groß | **sehr hoch** (privates API ändert sich ohne Ankündigung, Sperrrisiko für das Konto) | **rechtlich unklar** (s. u.) | ~50 % einmalig, langfristig schlecht | **Nicht verfolgt** |

**Korrektur 2026-11-21, zweite Aktualisierung:** Die hier unter „Option H" vorgenommene Einstufung von `smart-aosu-cloud` als risikoärmeren Weg war falsch (Begründung und neue Priorisierung in `adr-0002-path-reassessment.md`). Weg A (Push-Benachrichtigungen) ist der primäre Weg, nicht `smart-aosu-cloud`.

## Aktualisierung 2026-11-21 (nach vertiefter Recherche, `ecosystem-research.md`)

Neue Option **H: bestehende Integration `tapion69/smart-aosu-cloud` nutzen.** Sie deckt Video und PTZ für Kameras hinter einer H1L laut Autor ab (Entwicklungsaufbau, nicht von Dritten validiert). Zeitaufwand: klein. Wartung: liegt beim Autor, Projekt sehr aktiv, aber experimentell und an APK 5.10.12 gebunden. Risiken: (1) Anmeldung mit Ihrem Aosu-Konto über einen nicht offiziellen Client, die ToS verbieten inoffiziellen Zugriff, Kontosperre möglich; (2) die Integration braucht eine eigene Kopie der APK 5.10.12, die Sie rechtmäßig beziehen müssen; (3) Lizenz PolyForm Strict erlaubt nur private Nutzung, **kein Forken, Erweitern oder Weitergeben**. Erfolgswahrscheinlichkeit für Ihre Konstellation (H1L plus Kameras): ca. 60 % (Schätzung, wegen fehlender Fremdvalidierung). Damit gilt: **Nutzen statt nachbauen**; ein eigener Client wäre eine Dublette ohne Lizenzgrundlage zur Wiederverwendung und ist verworfen. Offen ist die Nutzerentscheidung zu Konto und Risiko. Weg A (Ereignisse über Push-Benachrichtigungen) und B (Anfrage an Aosu) bleiben als konto-risikofreie Ergänzung bestehen.

## Entscheidung (Stand vor der Aktualisierung)

Architektur „**Aosu Bridge**", gebaut aus legalen Bausteinen statt aus einem nachgebauten Cloud-Protokoll:

1. **Ereignisse (A):** Android-Gerät mit der offiziellen Aosu-App, darauf die HA-Companion-App. Benachrichtigungen des Pakets `com.glazero.android` werden nach Kanal-ID in HA-Ereignisse (Bewegung, Klingel, Akku schwach, Demontage) übersetzt. Kein Eingriff in die App, kein Protokollwissen nötig.
2. **Offizieller Weg (B):** Anfrage an Aosu und DSGVO-Auskunft/Datenübertragung. Entwurf: `docs/outreach.md`.
3. **Lokaler Weg (C):** passive Prüfung der H1L am Standort. Nur wenn sie eine Schnittstelle zeigt, entsteht ein `LocalTransport` (siehe `architecture.md`).
4. **Livebild (D)** erst, wenn A bis C den Bedarf nicht decken.

Der Transport-Vertrag aus `architecture.md` bleibt gültig. Ein `CloudTransport` wird nur gebaut, wenn Aosu Zugang bietet (B).

## Warum G nicht

Der Weg verlangt, Schlüssel aus den nativen Bibliotheken der App zu holen und die App gegenüber der Cloud zu imitieren. Nach meinem Verständnis (keine Rechtsberatung) berühren das: die enge Ausnahme des § 69e UrhG (Dekompilierung nur zur Interoperabilität, strenge Zweckbindung; § 69d Abs. 3 deckt nur Beobachten/Testen im normalen Betrieb), Nutzungsbedingungen von Aosu (Vertragsverstoß, Konto-Sperre) und § 202c StGB, falls Vorbereitungswerkzeuge für unbefugten Zugriff entstehen. Mit eigenem Konto und eigenen Geräten ist die Strafbarkeit unwahrscheinlich, aber die Lage ist unklar, und das Wartungsrisiko spricht ohnehin gegen G. **Bei Interesse an G ist vorher eine juristische Einschätzung nötig.**

## Kleinste nötige Validierung (entscheidungsrelevant)

Eine echte Aosu-Benachrichtigung abfangen und prüfen, ob Titel/Text Kamera und Ereignistyp enthalten (Kanal-IDs sind bekannt, Textinhalt nicht). Ohne das ist unklar, ob A pro Kamera unterscheiden kann. Vorgehen: nach dem nächsten Bewegungsereignis in HA das Attribut „Last notification" der Companion-App ansehen oder per `adb shell dumpsys notification --noredact` lesen, bevor die Meldung weggewischt wird.

## Arbeitspakete (priorisiert)

1. Benachrichtigungsformat validieren (klein).
2. HA-Blueprint/Automation für Kanal → Ereignis, mit Test gegen aufgezeichnetes Beispiel.
3. Anfrage an Aosu senden (Nutzer).
4. H1L-LAN-Prüfung am Standort.
5. Erst danach entscheiden über D oder `LocalTransport`.
