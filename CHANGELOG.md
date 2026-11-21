# Changelog

## [Unveröffentlicht]

### Hinzugefügt
- Analyse der Aosu-App 5.11.20: `docs/apk-findings.md`, `docs/recommendation-report.md`, `docs/architecture.md`, `docs/ecosystem-research.md`, ADRs 0001/0002.
- `tools/lan_probe.py`: passive LAN-Prüfung der Basisstation (Tuya-Discovery mitlesen, TCP-Port-Check) mit Tests.
- Pflichtdateien: LICENSE (MIT), README, CONTRIBUTING, SECURITY, CODE_OF_CONDUCT.
- `crates/aosu-core`: `Transport`-Trait, Capability-Modell, Mock/Local/Cloud-Transport-Grundgerüste, passive Netzwerk-Diagnose, Plugin-Registry (17 Tests).
- `app/`: Tauri+React+TypeScript-Desktop-Shell mit Geräteliste und Diagnose-Ansicht gegen Mock-Daten (10 Vitest-, 2 Playwright-Tests).
- `homeassistant/custom_components/aosu`: HA-Integrationsgerüst gegen Mock-Backend (3 Tests).
- `tools/notification_bridge.py`: parst Aosu-Push-Benachrichtigungen aus `adb dumpsys` in strukturierte Ereignisse (6 Tests).
- `.github/workflows/ci.yml`: CI für Rust/Frontend/Python/HA-Lint.
