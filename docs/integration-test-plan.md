# Test- und Einsatzplan: smart-aosu-cloud (Stand 2026-11-21)

Entscheidung: Wiederverwendung vor Eigenentwicklung (ADR 0001). Kein eigener Client, solange `tapion69/smart-aosu-cloud` den Bedarf abdecken kann.

## Befund zur Nutzbarkeit

| Prüfpunkt | Ergebnis | Evidenz |
|---|---|---|
| Zielkonstellation (Kameras hinter H1L, Video + PTZ) | passt grundsätzlich | README des Projekts (Autorenangabe, nur eigener Aufbau) |
| **Kameramodelle des Nutzers** | **alle 4 Kameras sind C7S**: Terasse Norden (SN …146346), Garten Süden (…145784), Einfahrt Süden (…146173), Garten Westen (…145084) | App, Geräteinfos je Kamera, am Testgerät gelesen |
| Einordnung C7S zu C5L | Nutzerangabe: „C7S ist nur die 3K-Variante der C5L mit 2K" (von mir nicht technisch verifiziert, aber plausibel als gleiche Hardware-/SDK-Familie mit höherer Auflösung) | Nutzerangabe 2026-11-21 |
| Validierte Modelle des Projekts | nur **C5L** und **C9C3CA11** hinter H1L | README, INSTALLATION_TEST.md |
| Basisstationsmodell | nicht ermittelt: Der App-Pfad zu „Basisstationinfos" führt in dieser Version abweichend zur „TrueSight Intelligenz"-Cloud-Abo-Seite statt zu Geräteinfos. Weitere Versuche abgebrochen (Risiko von Fehltipps auf Teilen/Abmelden/Store während der Navigation). | Testgerät, mehrere Navigationsversuche |
| Voraussetzungsdateien | „APK Lite" (ca. 1,7 MB) und „OEM-ZIP" (ca. 87 KB), **Version 5.10.12**, per Prüfsumme festgelegt; andere Versionen werden abgelehnt | INSTALLATION_TEST.md |
| Herkunft dieser Dateien | **nicht dokumentiert**; „modifizierte APK" laut README, Tester muss sie „rechtmäßig beschaffen". Aus unserer 5.11.20 nicht herstellbar | README, Doku-Durchsicht |
| Anmeldung | mit E-Mail/Passwort über ein „fiktives Android-Profil"; laut Autor **auf keiner externen Installation bestätigt**; Sperre von einer Stunde zwischen Versuchen | INSTALLATION_TEST.md |
| Testumgebung | Autor empfiehlt ein **zweites Home Assistant**, die Installation ist noch kein HACS-Release (manuelles Kopieren) | INSTALLATION_TEST.md |
| Lizenz | PolyForm Strict: nur eigene Nutzung, keine Änderungen/Weitergabe | LICENSE des Projekts |

## Bewertung

Erfolgswahrscheinlichkeit, dass die Integration **mit Ihren C7S-Kameras** funktioniert, ohne Anpassung: ca. **30 bis 40 %** (Schätzung, leicht angehoben, weil alle vier Kameras identisch C7S sind und laut Nutzerangabe eng mit der validierten C5L verwandt — gleiche Baureihe, nur höhere Auflösung). Weiterhin unverändert: Anmeldeprofil extern unbestätigt, Voraussetzungsdateien (APK Lite + OEM-ZIP für 5.10.12) nicht ohne Weiteres beschaffbar (siehe unten).

## Entscheidungen

1. **Nicht blind installieren.** Ohne die zwei Dateien läuft die Einrichtung nicht an, und selbst gebaute Ersatzdateien würden die Prüfsummen nicht erfüllen und wären eine unautorisierte Ableitung der Aosu-APK.
2. **Kontakt zum Autor zuerst** (Entwurf unten): Bezugsquelle der Dateien, Erwartung für C7S, Bereitschaft zu Fremdtests und zu einer freieren Lizenz für Beiträge. Wir wären seine erste externe Testinstallation, das ist ein Gewinn für ihn.
3. **Keine Anmeldung** mit den Testzugangsdaten, bevor die Voraussetzungen stehen (ToS-Risiko, einstündige Sperre, kein Nutzen). `Credential_aosu_test` wurde bisher **nicht** abgerufen.
4. **Weg A** (Ereignisse über Push-Benachrichtigungen) bleibt als risikofreie Ergänzung. Die App bietet pro Kamera „Nachrichtenwarnung" und „Bewegungserkennung" (bei „Terasse Norden" aktiviert). Der Monitor hat bisher keine Benachrichtigung erfasst.

## Herkunft der Voraussetzungsdateien (Code-Review, nicht ausgeführt)

Die Module `oem_resource_bundle.py` und `profile_bootstrap.py` des Fremdprojekts wurden **nur gelesen, nicht ausgeführt**. Befund: Das OEM-ZIP enthält fünf per SHA-256 geprüfte Dateien aus der **vollen** Aosu-APK (u. a. `AndroidManifest.xml`, `certificate.der`, `libthing_security_algorithm.so`), aus denen App-Key, App-Secret und weitere Identitätsdaten ausgelesen werden. Das Profil-Modul erzeugt zusätzlich ein Kunstprofil, das ein echtes Android-Gerät imitiert.

**Entscheidung:** Diese Extraktion führe ich nicht durch, auch nicht mit dem Testkonto und auch nicht, nachdem Sie die 5.10.12 selbst heruntergeladen haben. Das ist unabhängig von der Frage „eigenes Konto, eigene Geräte" eine bewusst gezogene Grenze, weil die urheberrechtliche Einordnung (§ 69e UrhG, ggf. § 95a UrhG) ungeklärt ist und eine Fehleinschätzung hier nicht revidierbar wäre (Kontosperre, Rechtsrisiko). Eine erneute Bitte ändert daran nichts; nötig wäre eine tatsächliche juristische Prüfung. Das 5.10.12-Download liegt unangetastet unter `Aosu_5.10.12_APKPure.xapk` im Projektordner.

## Risiken

- Kontosperre durch inoffiziellen Zugriff (README-Warnung anderer Projekte, bei diesem nicht ausgeschlossen). Mitigation: nur das Testkonto mit geteilten Geräten, Anmeldeversuche selten.
- Projekt kann sich schnell ändern oder eingestellt werden (Ein-Personen-Projekt, Betaphase).
- Unfertige PTZ-Freigabe bei C9-Modellen: Nutzung nur mit Sicherheitsvorkehrungen, bei C7S unbekannt.

## Nächste Schritte

1. Nutzer: Anfrage an den Autor senden oder mir das Posten im Projekt ausdrücklich erlauben (Entwurf in `outreach.md`, Abschnitt 3).
2. Nutzer: Falls Sie die zwei Dateien vom Autor bekommen, in einen Ordner legen und mir den Pfad nennen. Dann: Backup, Test-HA oder Entwicklungsinstanz, Installation, Anmeldung mit `Credential_aosu_test`.
3. Erledigt: Alle 4 Kameramodelle ermittelt (alle C7S). Basisstationsmodell am Gerät nicht erreichbar (App-Navigationsproblem); falls nötig, später per anderer Quelle (Typenschild, Rechnung) ermitteln.
4. Ich: Ergebnisse des Benachrichtigungs-Monitors auswerten und bei Erfolg den HA-Blueprint (Kanal → Ereignis) erstellen.
5. Ohne Antwort des Autors innerhalb von ca. 3 Wochen: Weg A plus Anfrage an Aosu als dauerhafte Lösung, Video bleibt in der App.
