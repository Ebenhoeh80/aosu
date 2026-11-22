# Aosu Desktop (inoffiziell)

Tauri + React + TypeScript Desktop-Client. **Aktueller Stand: volle Produktoberfläche.** Der Standardtransport ist `mock` (Demo-/Mock-Daten, ohne Hardware und Konto nutzbar). Über die Einstellungen ist zusätzlich `push-log` wählbar: dieser Transport liest echte Aosu-Push-Aufzeichnungen aus einer JSONL-Datei und stellt sie mit persistenter Event-Plattform bereit. Echte `local`-/`cloud`-Authentifizierung bleibt blockiert, siehe `../docs/adr-0002-path-reassessment.md` und `../docs/ROADMAP.md`.

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

UI → Tauri-Kommandos (`src-tauri/src/commands.rs`) → `aosu-core` (`../crates/aosu-core`). Die UI kennt keine Transport-Details, nur `Device`/`CapabilityReport`-Typen. Die Registry registriert fünf Transporte: `mock`, `kestrel-mock` (fiktiver Zweithersteller zur Architekturprüfung), `local` und `cloud` (Gerüste, Auth blockiert) sowie `push-log` (echte Push-Daten aus JSONL, persistenter SQLite-Event-Store). Standard ist `mock`; `list_devices`/`recent_events` liefern dann Demo-Daten, `run_network_diagnostics` führt einen echten, rein lesenden Port-Scan aus. Die Event-Plattform-Kommandos (`query_events`, `event_analytics`, …) fallen bewusst **nicht** auf erfundene Daten zurück, sondern melden einen ehrlichen Fehler, wenn der aktive Transport keinen Store hat.
