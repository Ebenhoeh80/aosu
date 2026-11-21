# Aosu Desktop (inoffiziell)

Tauri + React + TypeScript Desktop-Client. **Aktueller Stand: zeigt nur Demo-/Mock-Daten.** Siehe `../docs/architecture.md` und `../docs/ROADMAP.md` für den Grund (offene Protokollfragen, siehe `../docs/adr-0002-path-reassessment.md`).

## Entwicklung

```bash
npm install
npm run dev          # Vite-Dev-Server, http://localhost:1420
npm run tauri dev    # volle Desktop-App (braucht funktionierenden Rust-Build)
```

## Tests

```bash
npm test             # Vitest (Unit/Komponenten)
npx playwright install --with-deps chromium   # einmalig
npm run test:e2e     # Playwright (gegen den Vite-Dev-Server, nicht die gebündelte App)
npx tsc --noEmit     # Typprüfung
```

## Build

```bash
npm run build         # Frontend
npm run tauri build   # volles Desktop-Bundle (braucht einen funktionierenden Rust-Build, siehe ../CONTRIBUTING.md für den Windows-Workaround ohne MSVC)
```

## Architektur

UI → Tauri-Kommandos (`src-tauri/src/commands.rs`) → `aosu-core` (`../crates/aosu-core`). Die UI kennt keine Transport-Details, nur `Device`/`CapabilityReport`-Typen. Aktuell ist der verdrahtete Transport `MockTransport`; `list_devices`/`recent_events` liefern Demo-Daten, `run_network_diagnostics` führt einen echten, rein lesenden Port-Scan aus.
