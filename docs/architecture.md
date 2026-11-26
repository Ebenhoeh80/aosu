# Architektur: Aosu-Desktop-Client (Stand 2026-11-26, nach H1L-Netzwerkbefunden und Cloud-/Tuya-Prüfung)

## Doppeltes Ziel: Aosu-Client und Tuya-Plattform

Jede Design-Entscheidung hier wird gegen zwei Fragen geprüft: (1) Baut das einen funktionierenden Aosu-Client? (2) Könnte dasselbe Stück auch die Grundlage einer breiteren Open-Source-Tuya-Kameraplattform sein? Wo beides mit vertretbarem Mehraufwand möglich ist, wird die generischere Variante gewählt — siehe `docs/ecosystem-integration.md` für die konkreten Konsequenzen (eigenständige `tuya-lan-protocol`-Crate, erweiterbare Transport-Registry statt geschlossenem Enum, geplante Integrationspunkte zu go2rtc/Frigate/Scrypted).

Grundsatz unverändert: Ein Transport gilt erst als „unterstützt", wenn er validiert ist. Jede Schicht unten trennt **belegt** (direkt beobachtet) von **vermutet** (abgeleitet) und **unbekannt**.

## 1. Desktop-Anwendungsarchitektur

```
UI (React, TypeScript, Vite, Tailwind, shadcn/ui)
   |  Tauri-Befehle / Events
Kern (Rust): Geräte-Modell, Ereignisbus, Konfiguration, Diagnose, Plugin-Registry
   |  trait Transport
Transporte (austauschbar, siehe Abschnitt 3-4)
```

- **Belegt:** Dies ist die bereits in ADR 0001 festgelegte Zielarchitektur, keine neue Annahme.
- **Unknown:** keiner, reine Design-Entscheidung.
- **Heute vollständig baubar:** Projektgerüst (Tauri+Rust+React/TS/Vite/Tailwind), CI mit Lint/Test, Vitest+Playwright, komplette UI (Geräteliste, Geräte-Detail inkl. Medien-Tab, Einstellungen, Ereignis-Feed) gegen eine **Mock-Transport-Implementierung** für Inventar/Ereignisse. Der Medien-Tab hat bewusst **keinen** Mock-Fallback: Streams, Audio und Befehle kommen immer vom aktiven Transport, sonst erscheint eine ehrliche Fehlermeldung.

## 2. Abstraktionsschicht: `trait Transport`

```rust
trait Transport {
    fn discover(&self) -> Vec<Device>;
    fn devices(&self) -> Vec<Device>;
    fn events(&self, since: Timestamp) -> Stream<Event>;
    fn live(&self, device: &Device) -> Result<MediaStream, Unsupported>;
    fn talk(&self, device: &Device) -> Result<AudioChannel, Unsupported>;
    fn recordings(&self, device: &Device, range: TimeRange) -> Vec<Recording>;
    fn command(&self, device: &Device, cmd: Command) -> Result<(), Unsupported>;
    fn capability_report(&self) -> CapabilityReport; // pro Fähigkeit: supported/unsupported/unknown
}
```

- **Belegt:** Reine Design-Entscheidung, keine externe Abhängigkeit.
- **Heute baubar:** vollständig, inkl. Unit-Tests gegen Mock-Implementierung.

## 3. Geräte-Kommunikationsschicht (lokal: H1L)

| | Status |
|---|---|
| H1L erreichbar, TCP 6668 offen (Tuya-Standard-Lokalport) | **Belegt** |
| RTSP(554)/HTTP(80/443/8080)/MQTT(8883)/8554/8899 geschlossen | **Belegt** |
| Generisches Tuya-Framing (Magic-Bytes, CRC) wird auf Transportebene angenommen, dann sofort ohne Antwort geschlossen | **Belegt** |
| Sitzungsbelegung (Livebild aktiv/inaktiv) verändert das Verhalten nicht | **Belegt** |
| Ursache: Protokoll-/Versions-Mismatch (führende Hypothese) vs. herstellereigene Abwandlung | **Vermutet**, nicht unterscheidbar |
| Ob Medien-/Statuszugriff je lokal (ohne Cloud) funktioniert | **Unbekannt** |
| `local_key` auf rechtmäßigem, nicht-destruktivem Weg erhältlich | **Unbekannt**, bisher kein Weg identifiziert |

- **Abhängigkeitsrisiko: hoch** — vollständig blockiert ohne `local_key`.
- **Heute baubar ohne Schlüssel:** `LocalTransport`-Grundgerüst, das die bereits geschriebene und getestete Frame-Kodierung/-Dekodierung (Magic-Bytes, CRC, generisches Format aus `tools/tuya_fingerprint*.py`) als wiederverwendbares, unit-getestetes Modul produktiviert und sauber `Unsupported`/`Unauthenticated` zurückgibt. Das ist echter, testbarer Fortschritt, ohne das Geheimnis zu benötigen.

## 4. Cloud-Kommunikationsschicht (Glazero-API / Tuya-Medienebene)

| | Status |
|---|---|
| API-Host `api.aosulife.com`, H5-Host `home.aosulife.com` | **Belegt** (App-Strings) |
| MQTT-TLS:8883, HTTPS:443, UDP-Relay während Livebild, kein LAN-Verkehr | **Belegt** (eigene Messung) |
| `SignInterceptor` mit statischem Salt für Glazero-API-Signatur existiert | **Belegt** (Code gelesen; Salt und AES-Material werden 2026-11-26 zur Laufzeit aus der nutzerbereitgestellten APK gelesen, nie ins Repo geschrieben) |
| Login-Antwort enthält `sid`, `uid`, `countryCode`, `tuyaUsername`, `tuyaUid` | **Belegt** (Feldnamen einer eigenen, lesenden Anmeldung; keine Werte gespeichert) |
| Network-Security-Config ohne Pin-Set, `cleartextTrafficPermitted` nur als Basis | **Belegt** |
| Programmatischer `ThingCertificatePinner` in der Tuya-SDK-Schicht aktiv genutzt | **Belegt** |
| Ob die Glazero-Schicht selbst (`api.aosulife.com`) zusätzlich programmatisch gepinnt ist | **Unbekannt**, Experiment dafür vorbereitet, aber (noch) nicht freigegeben/ausgeführt |
| Medien-Handshake erfordert Tuya-SDK-Sitzungsschlüssel aus OEM-Secret | **Vermutet für den OEM-Pfad** (allgemeines Tuya-Architekturmuster, nicht Aosu-spezifisch verifiziert) |
| `/v1/event/getImageList` antwortet `errno=0` mit `data.list[].timeInfo`/`eventType`/`uniformType`/`aiVision`/`sn` | **Belegt** (eigene lesende Prüfung 2026-11-26; Testkonto liefert aktuell 0 Zeilen) |
| Offizielle Tuya-OpenAPI mit eigenem IoT-Projekt und „Link App Account" ist der lizenzrechtlich saubere Medienpfad | **Belegt** (Tuya-Doku 2026-11-26); Adapter implementiert, aber nicht live verifiziert |

- **Abhängigkeitsrisiko: mittel** — Login, Gerätebestand, Firmware und der Ereignis-Feed sind ohne Pinning-Umgehung real erreichbar. Medien- und Steuerpfade bleiben über die AOSU-API unbelegt; der Tuya-Adapter ist implementiert, aber ohne eigenes IoT-Projekt nicht live geprüft.
- **Heute real funktionierend (2026-11-26):** `CloudTransport` mit echter Signaturberechnung, Login, Geräteliste, Firmware-Abfrage und normalisiertem Ereignis-Feed; verifiziert mit HTTP 200/`errno=0`, 5 Geräten und 5 Firmware-Versionen. Zugangsdaten, Session-Tokens und APK-Material bleiben ausschließlich im RAM; Bild-URLs, Event-IDs und KI-Payloads werden verworfen. `live`, `talk`, `recordings` und `command` geben weiterhin ehrlich `Unsupported` zurück. Die Tuya-Identitätsfelder der Login-Antwort sind ein belegter, aber nicht freigeschalteter Ansatz für einen OEM-Adapter.
- **Offizieller Tuya-Adapter (2026-11-26):** `tuya-cloud` in der Registry nutzt nur `client_id`/`client_secret` aus einem eigenen IoT-Projekt, HMAC-SHA256, Token und dokumentierte Endpunkte für Live-HLS, go2rtc-WebRTC, DP-Befehle, PTZ-Payloads und Cloud-Playback. Keine APK-Secrets, keine Geheimnisse auf Platte oder im Log. Live-, Talk-, Aufnahme- und Befehls-UI sind darüber bedienbar, aber erst nach erfolgreicher Live-Prüfung als „unterstützt" zu betrachten. Constraints: go2rtc für Zwei-Wege-Audio, öffentliche HTTPS-Callback-URL für HLS-Schlüssel, PTZ erst nach STOP-Test.

## 5. Optionale Home-Assistant-Integrationsschicht

- **Belegt:** HA unterstützt Custom-Components mit Config-Flow und Entity-Plattformen (Standardwissen, kein Aosu-spezifisches Risiko).
- **Belegt:** Keine PTZ-Entität nötig (alle vier Testkameras sind C7S ohne PTZ-Steuerung in der App).
- **Vermutet:** Push-Benachrichtigungen könnten Ereignis-Entitäten speisen, sobald Inhalt bestätigt ist.
- **Unbekannt:** Ob Benachrichtigungen je Kamera-Zuordnung enthalten (0 von erwarteten ~1/h in zwei Messfenstern erfasst, Ursache ungeklärt).
- **Heute baubar:** Vollständiges HA-Custom-Component-Gerüst (`manifest.json`, `config_flow.py` für Konto/optionale H1L-IP, Entity-Plattformen `camera`/`binary_sensor`/`sensor`/`light`), verdrahtet gegen die Mock-Transport-Implementierung — in einer echten HA-Entwicklungsinstanz installierbar und testbar, bevor irgendein echter Transport steht.

## 6. Plugin-Architektur

- Design-Entscheidung, kein externes Abhängigkeitsrisiko: Registrierung weiterer `Transport`-Implementierungen über `TransportRegistry` (u. a. `cloud`, `push-log`, `tuya-cloud`; künftig z. B. eine offizielle Aosu-API, falls je gewährt, oder ein Thingino-basierter lokaler Client für entsprechend geflashte Hardware).
- **Heute baubar:** Trait-Object-basierte Registry, Konfigurationsauswahl, dokumentierter Erweiterungspunkt. Die UI zeigt nur echte Antworten des aktiven Transports; nicht unterstützte Aktionen erscheinen als Fehler, nicht als simulierte Daten.

## 7. Logging und Diagnose

- **Heute baubar, hoher Nutzen, kein Risiko:** Strukturiertes Logging (`tracing` in Rust) mit fester Redaktionsregel (nie Zugangsdaten/Token/`local_key` loggen — deckt sich mit `SECURITY.md`). Ein „Diagnose"-Knopf in der UI, der die bereits geprüften, rein passiven Sonden (`tools/lan_probe.py`: Port-Scan, optionales UDP-Lauschen) produktiviert und einen Fähigkeits-Report (`CapabilityReport`) anzeigt.

## Entwicklungs-Roadmap: was heute ohne Protokoll-Auth baubar ist

**Phase 0 — sofort, keine externe Abhängigkeit:**
1. Projektgerüst (Tauri/Rust/React/TS/Vite/Tailwind), CI, Vitest+Playwright-Grundgerüst.
2. `trait Transport` + `CapabilityReport`-Modell, vollständig unit-getestet.
3. Mock-Transport-Implementierung (Fake-Geräte/-Ereignisse) → komplette, vorführbare UI.
4. Logging/Diagnose-Modul inkl. Redaktionsregel und „Diagnose"-Knopf (Port-Scan produktiviert).
5. Plugin-Registry-Grundgerüst.

**Phase 1 — weiterhin ohne Schlüssel/Secret:**
6. `LocalTransport`-Grundgerüst: TCP-Connect + generische Tuya-Frame-Kodierung/-Dekodierung, unit-getestet gegen das öffentliche Format, sauberer `Unauthenticated`-Rückgabewert.
7. `CloudTransport`-Grundgerüst: echte Hostnamen, Datentypen nach beobachteten Feldern, sauberer `CredentialsRequired`-Rückgabewert.
8. HA-Custom-Component-Gerüst, verdrahtet gegen Mock-Transport, in Entwicklungsinstanz installierbar.
9. Push-Benachrichtigungs-Brücke (Weg A) als eigenständiges, unabhängig lauffähiges Modul.

**Phase 2 — abhängig von externen Entscheidungen, nicht mir allein überlassen:**
10. Echte `LocalTransport`-Authentifizierung — blockiert durch fehlenden `local_key`-Beschaffungsweg.
11. ~~Echte `CloudTransport`-Authentifizierung — blockiert durch offenes Pinning-Experiment.~~ **Erledigt 2026-11-26:** Login, Geräteliste, Firmware und Ereignis-Feed laufen live ohne Pinning-Umgehung; nur APK-Material zur Laufzeit, keine Geheimnisse auf Platte.

Phase 0 und 1 sind vollständig umgesetzt und liefern eine lauffähige, testbare, vorführbare Anwendung. Phase 2 ist nicht mehr vollständig blockiert: Inventar/Ereignisse sind echt, Medien/Befehle benötigen entweder den noch nicht live verifizierten offiziellen Tuya-Adapter oder eine ausdrückliche Nutzerentscheidung zum OEM-Pfad.
