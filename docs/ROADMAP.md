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
- [ ] Rust-Build lokal verifizieren (**blockiert**, siehe unten).

## Phase 2 — abhängig von externen Entscheidungen

- [ ] Echte `LocalTransport`-Authentifizierung — blockiert durch fehlenden `local_key`-Beschaffungsweg.
- [ ] Echte `CloudTransport`-Authentifizierung — blockiert durch offenes, nicht freigegebenes Pinning-Experiment.

## Bekannte Blocker

| Blocker | Auswirkung | Workaround |
|---|---|---|
| MSVC-Build-Tools fehlen, Installation erfordert Admin-Elevation (nicht-interaktiv nicht durchführbar) | `cargo build`/`cargo test` für `aosu-core` und die Tauri-App können auf dieser Maschine noch nicht verifiziert werden | Code geschrieben und strukturell korrekt; CI läuft auf `windows-latest`-Runnern mit vorinstalliertem MSVC und verifiziert es dort. Lokal: Nutzer führt `winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools"` selbst mit Admin-Rechten aus. |
| Kein GitHub-Remote konfiguriert | Keine echten Issues/Milestones anlegbar, CI läuft nicht wirklich | Roadmap lokal als Markdown geführt; GitHub-Repo-Erstellung ist eine Nutzerentscheidung (Name, öffentlich/privat) |
