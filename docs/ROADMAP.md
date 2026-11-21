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

## Phase 2 — abhängig von externen Entscheidungen

- [ ] Echte `LocalTransport`-Authentifizierung — blockiert durch fehlenden `local_key`-Beschaffungsweg.
- [ ] Echte `CloudTransport`-Authentifizierung — blockiert durch offenes, nicht freigegebenes Pinning-Experiment.

## Produktentwicklungsphase (parallel zu Phase 2, nicht davon blockiert)

Ziel: vollständige Nutzerreise gegen Mock-Geräte, unabhängig vom Ausgang von Phase 2.

- [x] **Gerätebestand**: Suche, Filter (Hersteller/Status), Sortierung, Gruppierung nach Hersteller, Statusübersicht (`app/src/DeviceInventory.tsx`).
- [x] **Geräte-Detail**: Übersicht/Diagnose/Transport/Ereignisse/Benachrichtigungen/Metadaten-Tabs, unbekannte Felder werden explizit als „unbekannt" gezeigt statt versteckt (`app/src/DeviceDetail.tsx`).
- [x] **Ereignisverlauf**: Zeitachse, Such-/Schweregrad-/Art-Filter, Gruppierung aufeinanderfolgender gleicher Ereignisse (`app/src/EventHistory.tsx`).
- [x] **Benachrichtigungs-Center**: Schweregrad-Filter, Gelesen/Ungelesen, Bestätigungs-Workflow (`app/src/NotificationCenter.tsx`).
- [x] **Diagnose-Center**: Fähigkeits-Report + Netzwerk-Scan + Laufzeitinfo, JSON-Export (`app/src/DiagnosticsCenter.tsx`, `aosu_core::DiagnosticsBundle`).
- [x] **Einstellungen**: echte Transport-Umschaltung über die Registry; Abschnitte ohne echten Unterbau sind explizit als „noch nicht implementiert" markiert, nicht versteckt (`app/src/Settings.tsx`).
- [x] **Onboarding**: geführte Einführung (was ist echt/simuliert/experimentell), überspringbar, einmalig (`app/src/Onboarding.tsx`).
- [x] **Offline-Modus**: `app/src/api.ts` fällt bei jedem Tauri-Aufruf auf Mock-Daten zurück; die gesamte App inkl. E2E-Tests läuft ohne jede Tauri-Laufzeit.
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
| Kein GitHub-Remote konfiguriert | Keine echten Issues/Milestones anlegbar, CI läuft nicht wirklich | Roadmap lokal als Markdown geführt; GitHub-Repo-Erstellung ist eine Nutzerentscheidung (Name, öffentlich/privat) |
