# Architektur: Aosu-Desktop-Client (Stand 2026-11-21, nach H1L-Netzwerkbefunden)

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
- **Heute vollständig baubar:** Projektgerüst (Tauri+Rust+React/TS/Vite/Tailwind), CI mit Lint/Test, Vitest+Playwright-Grundgerüst, komplette UI (Geräteliste, Einstellungen, Ereignis-Feed, Live-Bild-Platzhalter) gegen eine **Mock-Transport-Implementierung** (Fake-Geräteliste, Fake-Ereignisse) — ermöglicht eine vollständig vorführbare App **ohne jede Protokoll-Auth**.

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
| `SignInterceptor` mit statischem Salt für Glazero-API-Signatur existiert | **Belegt** (Code gelesen, Wert nicht operationalisiert) |
| Network-Security-Config ohne Pin-Set, `cleartextTrafficPermitted` nur als Basis | **Belegt** |
| Programmatischer `ThingCertificatePinner` in der Tuya-SDK-Schicht aktiv genutzt | **Belegt** |
| Ob die Glazero-Schicht selbst (`api.aosulife.com`) zusätzlich programmatisch gepinnt ist | **Unbekannt**, Experiment dafür vorbereitet, aber (noch) nicht freigegeben/ausgeführt |
| Medien-Handshake erfordert Tuya-SDK-Sitzungsschlüssel aus OEM-Secret | **Vermutet** (allgemeines Tuya-Architekturmuster, nicht Aosu-spezifisch verifiziert) |

- **Abhängigkeitsrisiko: mittel-hoch** — abhängig vom noch offenen Pinning-Experiment.
- **Heute baubar ohne Secret:** `CloudTransport`-Grundgerüst mit echten Hostnamen, Request-/Response-Datentypen passend zu beobachteten Feldnamen (`uuid`, `t`, Header-Existenz `Gz-Sign`), das sauber `CredentialsRequired` zurückgibt. Die Signaturberechnung selbst bleibt als dokumentierte Lücke markiert, nicht implementiert.

## 5. Optionale Home-Assistant-Integrationsschicht

- **Belegt:** HA unterstützt Custom-Components mit Config-Flow und Entity-Plattformen (Standardwissen, kein Aosu-spezifisches Risiko).
- **Belegt:** Keine PTZ-Entität nötig (alle vier Testkameras sind C7S ohne PTZ-Steuerung in der App).
- **Vermutet:** Push-Benachrichtigungen könnten Ereignis-Entitäten speisen, sobald Inhalt bestätigt ist.
- **Unbekannt:** Ob Benachrichtigungen je Kamera-Zuordnung enthalten (0 von erwarteten ~1/h in zwei Messfenstern erfasst, Ursache ungeklärt).
- **Heute baubar:** Vollständiges HA-Custom-Component-Gerüst (`manifest.json`, `config_flow.py` für Konto/optionale H1L-IP, Entity-Plattformen `camera`/`binary_sensor`/`sensor`/`light`), verdrahtet gegen die Mock-Transport-Implementierung — in einer echten HA-Entwicklungsinstanz installierbar und testbar, bevor irgendein echter Transport steht.

## 6. Plugin-Architektur

- Design-Entscheidung, kein externes Abhängigkeitsrisiko: Registrierung weiterer `Transport`-Implementierungen (künftig z. B. eine offizielle Aosu-API, falls je gewährt, oder ein Thingino-basierter lokaler Client für entsprechend geflashte Hardware).
- **Heute baubar:** Enum- oder Trait-Object-basierte Registry, Konfigurationsauswahl, dokumentierter Erweiterungspunkt.

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
11. Echte `CloudTransport`-Authentifizierung — blockiert durch offenes Pinning-Experiment und die unverändert bestehende Grenze, ein gefundenes Signatur-Geheimnis nicht selbst zu operationalisieren.

Phase 0 und 1 sind vollständig unabhängig von jeder noch offenen Protokollfrage umsetzbar und liefern eine lauffähige, testbare, vorführbare Anwendung.
