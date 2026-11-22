# Empfehlungsbericht: Open-Source-Desktop-Ersatz für Aosu (com.glazero.android 5.11.20)

> ⚠ Veraltet (2026-11-22): Die Empfehlung wurde durch ADR 0002 und den implementierten Code überholt. Maßgeblich ist `docs/adr-0002-path-reassessment.md` (Weg A „Push-Ereignisse" primär, `smart-aosu-cloud` zurückgestellt). Dieses Dokument bleibt als historischer Entwurf erhalten.

Stand: 2026-11-21 · Status: veraltet (Entwurf, vor Implementierung) · Quellen: APK-Inventar (REA 6.10.1), zwei Recherche-Worker (Websuche, teils nur Suchsnippets)

## 1. Belegte Fakten

- Die App enthält Tuya-SDK-Komponenten: `libThingCameraSDK`, `libThingP2PSDK`, `libthing_security`, `libThingSmartLink`, `assets/thing_uni_plugins`. Dazu kommen Agora RTC/RTM, React Native (Hermes), Getui-Push und eigene Bibliotheken `libglazero-iot` / `libglazero-security`.
- Der Hersteller-Tuya-Bezug ist damit belegt. **Nicht belegt** ist, ob die Geräte über die öffentliche Tuya-Cloud oder über eine eigene Glazero-Cloud (HomeBase-Geräte) laufen.
- Es gibt kein bekanntes Projekt und keine HA-Integration speziell für AOSU. Ein HA-Community-Thread (2025) nennt weder RTSP noch ONVIF noch eine lokale API (unverifiziert).

## 2. Projektinventar (Auszug, relevante zuerst)

| Projekt | Lizenz | Nutzen für uns | Hinweis |
|---|---|---|---|
| go2rtc (Tuya-Quelle) | vor Nutzung prüfen | WebRTC-Stream-Backend, Zwei-Wege-Audio | Smart API: Tuya-Smart-Konto, Smart Life nicht unterstützt; Cloud API braucht Abo „IoT Video Live Stream" |
| DanEng1982/tuya-rtsp-bridge | MIT (laut Worker) | Windows-Installer, RTSP, PTZ über LAN | Aktivität unklar |
| seydx/tuya-ipc-terminal | MIT (laut Worker) | Go-Engine WebRTC zu RTSP | Aktivität unklar |
| tuya/tuya-device-sharing-sdk | MIT (laut Worker) | Geräte-API, kein Video | Einsatz mit OEM-Apps unbestätigt |
| Offizielle HA-Tuya-Integration | HA-Core | Basis für Entitäten | Kamera-Livestream eingeschränkt |
| xtend_tuya (HACS) | prüfen | Zusatzentitäten | Nur falls Kamera im Tuya-Konto landet |
| Scrypted Tuya-Plugin | prüfen | Alternative | Braucht eigenes IoT-Projekt |

Lizenzangaben stammen aus Worker-Berichten und sind vor jeder Wiederverwendung an der Quelle zu prüfen.

## 3. Entscheidungsregel (70 %)

Kein Projekt deckt nachweislich 70 % der Zielfunktion für Aosu ab. Die Stream-Projekte decken Livebild ab, aber nur, wenn die Kamera in einem unterstützten Tuya-Konto erreichbar ist. Das ist für Aosu ungeprüft. Events, Aufnahmen, Geräteverwaltung und Pairing fehlen meist.

## 3a. Aktualisierung nach APK- und Gerätemessung (2026-11-21)

Details in `apk-findings.md`. Kurzfassung: Aosu nutzt das Tuya-SDK mit **Glazeros eigener Cloud** (`api.aosulife.com`). Beim Livebild gab es **keinen LAN-Verkehr** zu Kamera oder Basisstation H1L, Steuerung und Video liefen über Cloud-Verbindungen (HTTPS, MQTT-TLS, UDP/Relay). Die H1L steht an einem anderen Standort und ist nur remote erreichbar, ein LAN-Test ist daher derzeit nicht möglich. Damit gilt:

- Weg 2 (öffentliche Tuya-Route, go2rtc, Smart Life) ist für Aosu sehr wahrscheinlich **nicht** nutzbar. Offiziell geprüft ist das nicht.
- Eine lokale Schnittstelle der H1L (Tuya-LAN auf UDP 6666/6667, TCP 6668) ist **unbestätigt** und nur am Standort der H1L prüfbar.

## 4. Gerankte Wege (aktualisiert)

Neue Rangfolge: (a) Dokumentation und Architekturentwurf mit offener Schnittstellenschicht (jetzt machbar); (b) LAN-Prüfung der H1L am Standort, danach lokaler Client falls möglich (blockiert, Standort); (c) Client gegen die Glazero-Cloud nur mit Schlüsselextraktion (nicht freigegeben, rechtlich heikel). Die ursprüngliche Liste folgt zur Nachvollziehbarkeit.

### Ursprüngliche Liste

1. **Kompatibilitätstest zuerst (Konfidenz hoch, Aufwand klein):** Mit einer echten Aosu-Kamera prüfen, ob sie über Smart-Life-QR/Device-Sharing oder go2rtc erreichbar ist. Ergebnis entscheidet alles Weitere.
2. **Falls erreichbar: erweitern statt neu bauen.** Tauri/React-Desktop-Client um go2rtc oder tuya-rtsp-bridge als Stream-Backend. HA-Seite über go2rtc plus xtend_tuya, keine eigene Integration.
3. **Falls nicht erreichbar: statische APK-Analyse** der Cloud-Endpunkte und Auth-Abläufe (JADX, dokumentierend). Reverse Engineering der App-Schlüssel und des P2P-Protokolls nur nach separater Freigabe, wegen rechtlicher Risiken (ToS, Umgehung von Schutzmaßnahmen) und fehlender Validierbarkeit ohne Gerät.
4. **Eigener Voll-Nachbau:** nicht empfohlen.

## 5. Offene Punkte

- Besitzt der Nutzer eine Aosu-Kamera und ein Konto zum Test?
- Tuya-Stream-Kosten/Quoten und ToS zu Drittclients (offizielle Doku nicht gefunden).
- Lizenzen von go2rtc, xtend_tuya, device-sharing-sdk an der Quelle bestätigen.
- Aktivität der MIT-Projekte (letzter Commit) prüfen.
