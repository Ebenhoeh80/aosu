# Aosu Desktop (inoffiziell)

Tauri + React + TypeScript Desktop-Client. **Aktueller Stand: volle Produktoberfläche mit echten Datenpfaden für Inventar/Ereignisse.** Der Standardtransport ist `mock` (Demo-/Mock-Daten, ohne Hardware und Konto nutzbar). Über die Einstellungen sind zusätzlich `push-log` (echte Push-Aufzeichnungen aus JSONL mit persistenter Event-Plattform), `cloud` (read-only Login, Geräteliste, Firmware und Ereignis-Feed) und `tuya-cloud` (offizieller Adapter mit eigenem IoT-Projekt) wählbar. Der Medien-Tab für Live, Gegensprechen, Aufnahmen und Befehle hat bewusst **keinen** Mock-Fallback. `local` bleibt ohne `local_key` blockiert; der Tuya-Adapter ist kompiliert und unit-getestet, aber nicht live verifiziert — Fähigkeiten werden erst nach einem erfolgreichen echten Aufruf als verfügbar gemeldet. Siehe `../docs/ROADMAP.md`.

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

UI → Tauri-Kommandos (`src-tauri/src/commands.rs`) → `aosu-core` (`../crates/aosu-core`). Die UI kennt keine Transport-Details, nur `Device`/`CapabilityReport`-Typen. Die Registry registriert sechs Transporte: `mock`, `kestrel-mock` (fiktiver Zweithersteller zur Architekturprüfung), `local` (ohne `local_key` blockiert), `cloud` (echtes read-only Inventar/Firmware/Ereignisse), `push-log` (echte Push-Daten aus JSONL, persistenter SQLite-Event-Store) und `tuya-cloud` (offizieller Tuya-Adapter, noch nicht live verifiziert). Standard ist `mock`; `list_devices`/`recent_events` liefern dann Demo-Daten, `run_network_diagnostics` führt einen echten, rein lesenden Port-Scan aus. Die Medien-Kommandos (`live_stream`, `open_talk`, `list_recordings`, `send_device_command`) und die Event-Plattform-Kommandos (`query_events`, `event_analytics`, …) fallen bewusst **nicht** auf erfundene Daten zurück, sondern melden einen ehrlichen Fehler, wenn der aktive Transport die Fähigkeit nicht hat. Auch der Demo-Transport meldet Medien/Befehle als nicht unterstützt statt sie zu erfinden. Gegensprechen nutzt eine vom Nutzer lokal in go2rtc konfigurierte `tuya://`-Quelle; die App übergibt nur den Streamnamen, keine Zugangsdaten. Tuya-`client_secret`, Token und Stream-URLs bleiben ausschließlich im RAM.
