# Roadmap

Siehe `docs/architecture.md` für die Begründung der Phaseneinteilung (gebaut nach Belegt/Vermutet/Unbekannt pro Schicht).

## Phase 0 — Projektgerüst (abgeschlossen)

- [x] Rust-Workspace mit `crates/aosu-core` (Transport-Trait, Capability-Modell, Mock/Local/Cloud-Transport-Grundgerüste, Diagnose-Modul, Plugin-Registry) — 16 Unit-Tests inkl. Property-Test.
- [x] Tauri+React+TypeScript+Vite+Tailwind-App-Grundgerüst unter `app/` mit Geräteliste und Diagnose-Ansicht gegen Mock-Daten — 7 Vitest-Tests, Playwright-Smoke-Test.
- [x] Home-Assistant-Custom-Component-Grundgerüst unter `homeassistant/custom_components/aosu/` (Config-Flow, camera/binary_sensor/sensor-Plattformen gegen Mock-Backend) — 3 pytest-Tests für die reine Python-Logik.
- [x] Push-Benachrichtigungs-Brücke (`tools/notification_bridge.py`) — 6 pytest-Tests.
- [x] CI-Pipeline (`.github/workflows/ci.yml`): Rust-Tests, Frontend-Tests+E2E, Python-Tests, HA-Lint.
- [x] Eigenes Git-Repository initialisiert, MIT-lizenziert.

## Phase 1 — ohne Protokoll-Geheimnisse (in Arbeit)

- [x] `LocalTransport`-Grundgerüst: Port-Erreichbarkeit, generische Tuya-Frame-Kodierung/-Dekodierung (unit-getestet, öffentliches Format).
- [x] `CloudTransport`-Grundgerüst: echte Hostnamen, Datentypen nach beobachteten Feldern, saubere `CredentialsRequired`-Rückgabe.
- [ ] Diagnose-Knopf in der UI verdrahten (Tauri-Kommando `run_network_diagnostics` existiert bereits, UI-Anbindung fehlt).
- [x] Rust-Build lokal verifiziert (siehe gelöster Blocker unten) — voller Workspace inkl. Tauri/WebView2 baut, 17/17 Tests grün, Clippy sauber.

## Phase 2 — abhängig von externen Entscheidungen

- [ ] Echte `LocalTransport`-Authentifizierung — blockiert durch fehlenden `local_key`-Beschaffungsweg.
- [ ] Echte `CloudTransport`-Authentifizierung — blockiert durch offenes, nicht freigegebenes Pinning-Experiment.

## Sicherheitsprüfungen (Stand 2026-11-21)

- `npm audit`: 0 Schwachstellen (nach Upgrade von vite/vitest/@vitejs/plugin-react, siehe Commit-Historie).
- `cargo audit`: 0 Schwachstellen (CVE-Ebene). 3 Hinweise auf unmaintained/unsound transitive Abhängigkeiten von Tauri/wry selbst (`chrono`, `proc-macro-error`, `glib`), kein Fix stromaufwärts verfügbar — außerhalb unserer Kontrolle, in CI als Information mitlaufend.

## Bekannte Blocker

| Blocker | Auswirkung | Workaround |
|---|---|---|
| ~~MSVC-Build-Tools fehlen, Installation erfordert Admin-Elevation~~ **gelöst** | — | Workaround statt MSVC: `winget install BrechtSanders.WinLibs.POSIX.UCRT --scope user` (kein Admin nötig) + `rustup toolchain install stable-x86_64-pc-windows-gnu` + `rustup default stable-x86_64-pc-windows-gnu` + projektlokale `.cargo/config.toml` mit MinGW-Linkerpfad. Damit baut der volle Workspace inkl. Tauri/WebView2 lokal. CI nutzt weiterhin natives MSVC auf `windows-latest`. |
| Kein GitHub-Remote konfiguriert | Keine echten Issues/Milestones anlegbar, CI läuft nicht wirklich | Roadmap lokal als Markdown geführt; GitHub-Repo-Erstellung ist eine Nutzerentscheidung (Name, öffentlich/privat) |
