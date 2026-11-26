# Roadmap

Siehe `docs/architecture.md` für die Begründung der Phaseneinteilung (gebaut nach Belegt/Vermutet/Unbekannt pro Schicht).

## Phase 0 — Projektgerüst (abgeschlossen)

- [x] Rust-Workspace mit `crates/aosu-core` (Transport-Trait, Capability-Modell, Mock/Local/Cloud-Transport-Grundgerüste, Diagnose-Modul) — 19 Unit-Tests.
- [x] **`crates/tuya-lan-protocol`**: generisches, markenunabhängiges Tuya-LAN-Framing als eigenständige, wiederverwendbare Crate herausgelöst (5 Tests inkl. Property-Test). Siehe `docs/ecosystem-integration.md`.
- [x] Transport-**Registry** statt geschlossenem Enum: `TransportRegistry::register(name, factory)` erlaubt Dritt-Code, eigene Transporte zu registrieren, ohne Kern-Code zu ändern.
- [x] Tauri+React+TypeScript+Vite+Tailwind-App-Grundgerüst unter `app/` mit Geräteliste und Diagnose-Ansicht gegen Mock-Daten — 7 Vitest-Tests, Playwright-Smoke-Test.
- [x] Home-Assistant-Custom-Component-Grundgerüst unter `homeassistant/custom_components/aosu/` (Config-Flow, camera/binary_sensor/sensor-Plattformen gegen Mock-Backend) — 3 pytest-Tests für die reine Python-Logik.
- [x] Push-Benachrichtigungs-Brücke (`tools/notification_bridge.py`) — 6 pytest-Tests.
- [x] CI-Pipeline (`.github/workflows/ci.yml`): Rust-Tests, Frontend-Tests+E2E, Python-Tests, HA-Lint.
- [x] Eigenes Git-Repository initialisiert, MIT-lizenziert.

## Phase 1 — ohne Protokoll-Geheimnisse (in Arbeit)

- [x] `LocalTransport`-Grundgerüst: Port-Erreichbarkeit, generische Tuya-Frame-Kodierung/-Dekodierung (unit-getestet, öffentliches Format).
- [x] `CloudTransport`-Grundgerüst: echte Hostnamen, Datentypen nach beobachteten Feldern, saubere `CredentialsRequired`-Rückgabe.
- [x] Diagnose-Knopf in der UI verdrahten (`run_network_diagnostics`, Teil des Diagnose-Centers).
- [x] Rust-Build lokal verifiziert (siehe gelöster Blocker unten) — voller Workspace inkl. Tauri/WebView2 baut, 17/17 Tests grün, Clippy sauber.
- [x] **Echter Cloud-Login + Gerätebestand + Firmware-Abfrage** (2026-11-26): `crates/aosu-core/src/transports/cloud/client.rs` liest die AOSU-API mit zur Laufzeit aus der nutzerbereitgestellten APK extrahiertem Signatur-/Verschlüsselungsmaterial. Live-Smoke-Test gegen das eigene Konto: HTTP 200, `errno=0`, 5 Geräte, 5 Firmware-Versionen. Zugangsdaten und Sessions bleiben nur im RAM; das Pinning-Experiment der Android-App ist für diesen Pfad **nicht** erforderlich.
- [x] **Lesender Cloud-Ereignis-Feed** (2026-11-26): `/v1/event/getImageList` wurde mit `sn[]` + `count` live geprüft (`errno=0`, Felder `data.list[].timeInfo.begin/end`, `eventType`, `uniformType`, `aiVision`, `sn`). Normalisierung erlaubt nur bekannte Kategorien; Bild-URLs, Event-IDs und KI-Payloads werden verworfen. Das Testkonto lieferte aktuell 0 Zeilen — der Endpunkt ist belegt, echte Ereignisinhalte nicht. Unit-Tests prüfen Zeitformate und Zuordnung ohne Live-Daten.

## Phase 2 — abhängig von externen Entscheidungen

- [ ] Echte `LocalTransport`-Authentifizierung — blockiert durch fehlenden `local_key`-Beschaffungsweg. Der lesende Cloud-Detail-Endpunkt `/v1/dev/getInfoBySn` wurde 2026-11-26 mit vier Seriennummern-Varianten und vier Parameternamen geprüft: durchgehend `errno=1`, ein Schlüsselfeld ist dort nicht erreichbar.
- [x] **Offizieller Tuya-Cloud-Adapter angelegt und verdrahtet** (2026-11-26): `crates/aosu-core/src/transports/tuya_cloud.rs` nutzt ausschließlich ein eigenes IoT-Projekt des Nutzers (`client_id`/`client_secret`), HMAC-SHA256-Signatur, Token und die dokumentierten OpenAPI-Endpunkte. Konten werden über „Link App Account" verknüpft; keine Secrets aus der AOSU-APK. Live-URL (HLS), WebRTC-Konfiguration via go2rtc, DP-Befehle, PTZ-Payloads und Cloud-Playback sind implementiert und unit-getestet, aber **nicht live verifiziert**. PTZ ist standardmäßig gesperrt und benötigt einen erfolgreichen STOP-Test; Aufnahmen benötigen eine öffentliche HTTPS-Callback-URL.
- [ ] Tuya-OEM-Adapter aus den AOSU-Identitätsfeldern (`countryCode`, `tuyaUsername`, `tuyaUid`) als **optionaler, nutzerfreigegebener** Pfad: Er setzt OEM-App-Zugangsdaten aus der APK und eine Zustimmung zur Tuya-Nutzung voraus (Konto-/ToS-Risiko) — nicht ohne ausdrückliche Nutzerentscheidung freigeschaltet. Der offizielle Adapter oben ist die lizenzrechtlich saubere Alternative.

## Event-Pipeline (abgeschlossen, 2026-11-22)

Erste reale Push-Benachrichtigung einer Aosu-Kamera empfangen und zu einer vollständigen, in zwei Sprachen parallel verifizierten Normalisierungs-Pipeline ausgebaut — siehe `docs/event-pipeline.md` für die Zuordnungstabelle und den Korrekturbefund (Kanalname sagt nicht zuverlässig den Inhalt voraus).

- [x] `tools/event_pipeline.py`: normalisiertes Ereignis-Vokabular (`MotionDetected`…`UnknownVendorEvent`), textbasierte Klassifikation mit Konfidenzstufe, Rohdaten nie verworfen — 16 Tests.
- [x] `crates/aosu-core/src/notification_pipeline.rs`: dieselben Regeln in Rust, Paritätstests gegen dieselben Fälle — 11 Tests.
- [x] `EventKind` um `Vehicle`, `Package`, `IntercomCall` erweitert; `wire_name()` übersetzt in das kanonische Vokabular für alles Konsumentenseitige.
- [x] `Event.metadata`: Rohdaten/Konfidenz/Klassifikationsnotiz, nie verworfen, auch im Rust-Modell.
- [x] **Echter End-to-End-Beweis**: `crates/aosu-core/tests/real_data.rs` liest eine anonymisierte, aber sonst unveränderte echte Aufzeichnung; dieselbe Datei testet auch die Python-Seite (`tests/test_ha_push_log_backend.py`).
- [x] `NotificationLogTransport` (Registry-Name `push-log`): liest die JSONL-Datei, wählbar in den Desktop-Einstellungen (Pfadfeld erscheint automatisch).
- [x] Home-Assistant-Modus „Notification-Bridge": `binary_sensor.py` erzeugt pro beobachteter Kamera Bewegungs- und Manipulations-Sensoren aus derselben Datei — der erste mit echten Daten funktionierende HA-Pfad, nicht nur Mock.
- [ ] Bekannte Lücke: HA liest per Polling, kein echter Push von der Datei in HA hinein.
- [ ] Noch unbeobachtet: ob andere Kanäle (`lowpower.v2`, `dismantle.v2`, `call.v2`) ebenfalls den Kameranamen im Text tragen, und ob spezifischere Schlüsselwörter (Person/Fahrzeug/Paket) je in der Praxis auftreten.

## Event Platform (abgeschlossen, 2026-11-22)

Die Event-Pipeline als reifste Produktionsfunktion genommen und zu einer vollständigen, persistenten Plattform ausgebaut — explizit *keine* weitere Protokoll-/Auth-/Transport-Recherche, dafür Tiefe auf echten Daten. Begründung/Schema siehe `docs/event-pipeline.md` Abschnitt „Event Platform: persistenter Store".

- [x] **Persistenter Event-Store**, zwei Sprachen, gleiches Schema, Schema-Versionierung dokumentiert und fehlschlagsicher geprüft: `crates/aosu-core/src/event_store.rs` (SQLite über `rusqlite`, 18 Tests) und `tools/event_store.py` (stdlib `sqlite3`, 10 Tests inkl. Parität gegen die echte Fixture).
- [x] Idempotente Ingestion (Dedup über Inhalts-Hash), Abfrage mit Filter (Kamera/Typ/Zeitraum/Volltext) und Pagination, Geräteaktivität, lokale Analytics (heute/nach Kamera/nach Typ/14-Tage-Verlauf), Export (JSON/CSV), konfigurierbare Aufbewahrung (nach Alter und/oder Zeilenzahl) — überlebt App-Neustarts.
- [x] `NotificationLogTransport` auf den Store umgebaut (vorher: Datei bei jedem Aufruf neu geparst, keine Persistenz). `rusqlite::Connection` ist nicht `Sync`, der `Transport`-Trait verlangt es aber — Store intern per `Mutex` gekapselt.
- [x] `Transport`-Trait um `query_events`/`event_analytics`/`device_activity`/`export_events`/`apply_event_retention` erweitert, Standard-Implementierung `Unsupported` für Transporte ohne Event-Store (Mock/Local/Cloud/Kestrel) — kein `Option<&EventStore>`-Downcasting nötig, Mutex-Zugriff bleibt vollständig in `NotificationLogTransport` gekapselt.
- [x] Tauri-Kommandos (`query_events`, `device_activity`, `event_analytics`, `export_events`, `apply_event_retention`) ohne Mock-Fallback — ein Fehler bedeutet hier meist „Transport hat keinen Store", nicht „keine Tauri-Laufzeit", und soll nicht durch erfundene Demo-Zahlen verschleiert werden.
- [x] Neuer App-Bereich **„Event-Plattform"** (`app/src/EventPlatform.tsx`): Timeline mit Filter/Suche/Pagination, Geräteaktivität, Analytics (14-Tage-Balken), Export-Buttons (JSON/CSV als Datei-Download), Aufbewahrungs-Steuerung — zeigt ehrlich `ErrorNotice`, wenn der aktive Transport (z. B. Mock) keinen Store hat, statt leere/erfundene Daten.
- [x] Vollständige Testabdeckung: 7 neue Rust-Unit-Tests in `notification_log.rs` für die Trait-Erweiterung, 10 neue Python-Tests, 6 neue Vitest-Tests (`EventPlatform.test.tsx`), 1 neuer Playwright-E2E-Test (ehrlicher Fehlerzustand im Mock-Modus), 4 neue `api.ts`-Tests (kein stiller Mock-Fallback).

## Produktentwicklungsphase (parallel zu Phase 2, nicht davon blockiert)

Ziel: vollständige Nutzerreise gegen Mock-Geräte, unabhängig vom Ausgang von Phase 2.

- [x] **Gerätebestand**: Suche, Filter (Hersteller/Status), Sortierung, Gruppierung nach Hersteller, Statusübersicht (`app/src/DeviceInventory.tsx`).
- [x] **Geräte-Detail**: Übersicht/**Medien & Steuerung**/Diagnose/Transport/Ereignisse/Benachrichtigungen/Metadaten-Tabs, unbekannte Felder werden explizit als „unbekannt" gezeigt statt versteckt (`app/src/DeviceDetail.tsx`). Der Medien-Tab ruft echte Tauri-Kommandos auf; ohne Unterstützung durch den Transport erscheint eine ehrliche Fehlermeldung statt eines erfundenen Streams oder einer erfundenen Bewegung.
- [x] **Ereignisverlauf**: Zeitachse, Such-/Schweregrad-/Art-Filter, Gruppierung aufeinanderfolgender gleicher Ereignisse (`app/src/EventHistory.tsx`).
- [x] **Benachrichtigungs-Center**: Schweregrad-Filter, Gelesen/Ungelesen, Bestätigungs-Workflow (`app/src/NotificationCenter.tsx`).
- [x] **Diagnose-Center**: Fähigkeits-Report + Netzwerk-Scan + Laufzeitinfo, JSON-Export (`app/src/DiagnosticsCenter.tsx`, `aosu_core::DiagnosticsBundle`).
- [x] **Einstellungen**: echte Transport-Umschaltung über die Registry, inklusive session-only Konfiguration für `cloud` und `tuya-cloud` (kein Passwort/`client_secret` auf Platte); Abschnitte ohne echten Unterbau sind explizit als „noch nicht implementiert" markiert, nicht versteckt (`app/src/Settings.tsx`).
- [x] **Onboarding**: geführte Einführung (was ist echt/simuliert/experimentell), überspringbar, einmalig (`app/src/Onboarding.tsx`).
- [x] **Offline-Modus**: `app/src/api.ts` fällt für Inventar, Ereignisse, Benachrichtigungen, Transportliste und Diagnose auf Mock-Daten zurück; die gesamte App inkl. E2E-Tests läuft ohne jede Tauri-Laufzeit. Medien, Befehle und Event-Plattform-Aufrufe haben bewusst **keinen** Mock-Fallback und zeigen echte Fehler.
- [x] **Fehlerbehandlung**: `ErrorNotice`-Komponente (Erklärung + wahrscheinliche Ursache + Vorschlag) statt roher Fehlermeldungen.
- [x] **Mock-Ökosystem erweitert**: 6 Geräte über alle Health-Zustände (Healthy/Degraded/Offline/Unsupported/PartiallySupported), simulierbarer Netzwerkausfall (`MockTransport::simulate_network_failure`).
- [x] **Multi-Vendor-Validierung**: zweiter, bewusst fiktiver Hersteller „Kestrel Labs" (`crates/aosu-core/src/transports/vendor2_mock.rs`), eigenständig implementiert, über die Registry als `kestrel-mock` erreichbar — beweist die Architekturgrenze ohne echtes zweites Gerät.
- [x] **Echter installierbarer Windows-Build verifiziert**: `npx tauri build` erzeugt sowohl ein MSI (`aosu-desktop_0.1.0_x64_en-US.msi`, 5,7 MB) als auch ein NSIS-Setup (`aosu-desktop_0.1.0_x64-setup.exe`, 3,7 MB), lokal über den GNU-Toolchain-Workaround gebaut. Damit ist „installieren und ausprobieren ohne Entwicklungsumgebung" real möglich, nicht nur `npm run dev`.
- [ ] Barrierefreiheit: Baseline vorhanden (semantische Rollen, `aria-label`, tastaturbedienbare Tabs/Buttons), kein systematischer Screenreader-/Kontrast-Audit.
- [ ] Zuordnung einzelner Geräte zu einem eigenen Transport (aktuell: ein globaler aktiver Transport für die ganze App).

## Sicherheitsprüfungen (Stand 2026-11-21)

- `npm audit`: 0 Schwachstellen (nach Upgrade von vite/vitest/@vitejs/plugin-react, siehe Commit-Historie).
- `cargo audit`: 0 Schwachstellen (CVE-Ebene). 3 Hinweise auf unmaintained/unsound transitive Abhängigkeiten von Tauri/wry selbst (`chrono`, `proc-macro-error`, `glib`), kein Fix stromaufwärts verfügbar — außerhalb unserer Kontrolle, in CI als Information mitlaufend.

## Bekannte Blocker

| Blocker | Auswirkung | Workaround |
|---|---|---|
| ~~MSVC-Build-Tools fehlen, Installation erfordert Admin-Elevation~~ **gelöst** | — | Workaround statt MSVC: `winget install BrechtSanders.WinLibs.POSIX.UCRT --scope user` (kein Admin nötig) + `rustup toolchain install stable-x86_64-pc-windows-gnu` + `rustup default stable-x86_64-pc-windows-gnu` + projektlokale `.cargo/config.toml` mit MinGW-Linkerpfad. Damit baut der volle Workspace inkl. Tauri/WebView2 lokal. CI nutzt weiterhin natives MSVC auf `windows-latest`. |
| ~~Kein GitHub-Remote konfiguriert~~ **erledigt 22.11.2026** | — | `origin` = `https://github.com/Ebenhoeh80/aosu` (public); PR #14 mit read-only Cloud-Transport am 26.11. gemergt, alle Checks grün |
| Kein eigenes Tuya-IoT-Projekt / keine verknüpften Konten | Der offizielle Tuya-Adapter ist kompiliert und unit-getestet, aber nicht live verifiziert | Nutzer legt ein IoT-Projekt an und verknüpft die App-Konten über „Link App Account"; Geheimnisse bleiben session-only |
| Keine öffentliche HTTPS-Callback-URL / kein lokales go2rtc / kein STOP-Test | Cloud-Aufnahmen, Gegensprechen und PTZ bleiben auch mit Tuya-Projekt blockiert | Optional `callbackBase`, `go2rtcBase` und `ptzEnabled` erst nach erfolgreichem STOP-Test setzen |
| Kein Release-Tag und kein Release-Workflow | Kein Release autonom auslösbar | Releases nur per Tag `v*` nach Manifest-Prüfung; Infrastruktur muss zuerst geschaffen werden |
