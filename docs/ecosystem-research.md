# Ökosystem-Recherche zu Aosu (Stand 2026-11-21)

Quellen: GitHub-Such-API (Repos, README, Dateibaum, Commits, Issues), Websuche, Aosu- und Home-Assistant-Foren. Codeberg lieferte HTTP 429 (nicht ausgewertet). GitLab lieferte keine relevanten Treffer. Code wurde nur zur Bewertung gelesen, nichts übernommen.

## Relevante Projekte

| Projekt | Inhalt | Status | Lizenz | Eignung |
|---|---|---|---|---|
| **tapion69/smart-aosu-cloud** (HA-Custom-Integration, Python, französisch) | Cloud-Integration für Aosu: Anmeldung mit AOSU-E-Mail/Passwort, Video und PTZ für **C5L und C9C hinter H1L** (laut Autor „confirmé" in 0.8.10 auf dem Entwicklungsaufbau), optionale H1L-IP für LAN-Wege, MQTT-/KCP-/ICE-Diagnosen | Sehr aktiv (ca. 100 Commits 11.–14.11.2026), 345 Dateien, 57 Tests, 4 Issues, keine Releases, Version 0.8.13-beta44-rc3. Selbst als **experimentell** und „nicht auf einer zweiten Installation validiert" bezeichnet | **PolyForm Strict 1.0.0**: private/nichtkommerzielle Nutzung erlaubt, **keine Weitergabe, keine Änderungen, keine abgeleiteten Werke** | Zur eigenen Nutzung am besten passend. Als Basis für ein eigenes Projekt nicht verwendbar (Lizenz). |
| R0M107/Aosu-API | Leere Hülle (README, 2 Dateien), Roadmap ohne Code. Warnt: Aosu-ToS verbieten inoffiziellen API-Zugriff, Kontosperre möglich | Ein Tag alt | MIT | Kein Nutzen |
| wltechblog/thingino-installers | Alternative Firmware (Thingino) für **C5L**, lokales RTSP durch Neuflashen | Aktiv | – | Nur anderes Modell, Garantieverlust, nicht für Ihre Geräte relevant |
| Dicklesworthstone/franken_surveillance_system | Rust-Forschungsprojekt, nennt „AOSU P1 Max" als Testkandidat | Aktiv | NOASSERTION | Kein Aosu-Protokollwissen erkennbar |
| idn-kop/AOSUNLOCKER, aosudh, übrige Treffer | Namensgleichheit, nicht verwandt | – | – | Irrelevant |

Foren (belegt): Aosu-Personal sagt, Akku-Kameras unterstützten weder ONVIF noch RTSP noch NAS/NVR, das Video sei nur über die App oder die lokale aosuBase zugänglich. Keine Antwort von Aosu auf die Anfrage nach einer Integration oder API. Keine Berichte über Kontosperren (nur Warnung im R0M107-README). Die Aussagen eines HA-Forenbeitrags zu Domains (`api-us/api-eu.aosulife.com`) stammen aus einer KI-Ausgabe und sind unbelegt, `api.aosulife.com` ist im APK belegt.

## Wichtige technische Erkenntnisse aus smart-aosu-cloud (Autorenangaben, nicht selbst geprüft)

- Die Video-/Steuerschicht läuft über **Tuya-Infrastruktur** (Thing-4-HTTP-Offer, MQTT `smart/mb/in`, WebRTC mit TURN/KCP), die Aosu-Cloud liefert Geräteliste und Zuordnung. Das passt zu unseren Messungen (HTTPS, MQTT-TLS, UDP-Relay).
- Der Autor liest die benötigten App-Daten **zur Laufzeit aus einer vom Nutzer selbst bereitgestellten APK (Version 5.10.12) und einem OEM-ZIP** und liefert keine Schlüssel im Repository aus. Andere App-Versionen werden abgelehnt. Wir haben 5.11.20.
- Es gibt LAN-Wege für die H1L (ICE/LAN-Signalisierung) mit optionaler privater H1L-IP. Ob sie ohne Cloud funktionieren, ist offen.
- Offener Punkt des Autors: eine **alleinstehende C9C** (ohne H1L) hat noch kein validiertes Video.

## Folgerung für uns

Der Weg „bestehendes Projekt nutzen" erfüllt die 70-Prozent-Regel (Video, PTZ, Geräteliste für genau Ihre Konstellation H1L plus Kameras), aber **nicht** als Code-Basis, nur zur eigenen Nutzung. Siehe Aktualisierung in `adr-0001-client-strategy.md`.
