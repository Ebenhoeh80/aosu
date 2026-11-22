# Event-Pipeline: Push-Benachrichtigung → normalisiertes Ereignis

Eine Quelle der Wahrheit für die Zuordnungsregeln, unabhängig implementiert in Python (`tools/event_pipeline.py`, dort wo `adb` läuft) und Rust (`crates/aosu-core`, dort wo Desktop/HA konsumieren), mit Paritätstests gegen dieselben Fixtures. „Dieselbe Pipeline" heißt hier: dieselben Regeln, zweimal verifizierbar umgesetzt — nicht ein einziges gemeinsames Binary, da Python (adb-Zugriff) und Rust (Desktop-App) getrennte Prozesse sind.

## Wichtiger Korrekturbefund (2026-11-22)

Der Android-Kanalname von `event.v2` lautet laut Gerät **„Benachrichtigung über Klingelereignisse"** (klingt nach Klingel-spezifisch). Der tatsächlich beobachtete Text war aber „Ereignis vor Garten Süden entdeckt" — generisch, **nicht** „Bewegung erkannt" oder „Klingel gedrückt". Der Kanalname sagt also nicht zuverlässig voraus, was im Text steht. Die App selbst unterscheidet intern „Gesamtereignisse" von „BewegungEreignisse" (siehe Sicherheitsbericht in der App) — es gibt also vermutlich mehrere Ereignis-Unterarten, die der Kanalname allein nicht verrät.

**Konsequenz:** Klassifikation läuft primär über den **Textinhalt** (spezifische Schlüsselwörter), Kanal nur als schwacher Fallback-Hinweis. Wo keine spezifische Schlüsselwort-Übereinstimmung vorliegt, wird nicht spekulativ auf „Motion" geraten, sondern die Unsicherheit bleibt in den Metadaten sichtbar (`confidence`-Feld).

## Normalisiertes Vokabular

`MotionDetected`, `HumanDetected`, `VehicleDetected`, `PackageDetected`, `DoorbellPressed`, `TamperDetected`, `BatteryLow`, `IntercomCall`, `DeviceOffline`, `DeviceOnline`, `UnknownVendorEvent`.

Die letzten drei (`TamperDetected`, `BatteryLow`, `IntercomCall`) sind Ergänzungen zur Beispielliste der Aufgabenstellung, begründet durch real am Gerät gelesene Kanalnamen (`dismantle.v2`, `lowpower.v2`, `call.v2`) — Weglassen hieße, belegte Information zu verwerfen, was die Pipeline ausdrücklich nicht tun soll.

## Zuordnungstabelle

| Kanal (`mId`) | Android-Anzeigename (belegt, vom Gerät gelesen) | Textmuster (belegt oder vermutet) | Normalisierter Typ | Konfidenz |
|---|---|---|---|---|
| `event.v2` | „Benachrichtigung über Klingelereignisse" | `Ereignis vor <Kamera> entdeckt` (belegt, 1 Sample) | `MotionDetected` | **niedrig** — generischer Text, keine Unterart erkennbar |
| `event.v2` | — | enthält „Bewegung"/„Person"/„Mensch"/„Fahrzeug"/„Auto"/„Paket"/„Klingel" (noch nie beobachtet) | `MotionDetected`/`HumanDetected`/`VehicleDetected`/`PackageDetected`/`DoorbellPressed` je nach Treffer | mittel, erst bei realer Beobachtung hoch |
| `ring.v2` | „Benachrichtigung über Klingel" | beliebig (Kanalsemantik eindeutig) | `DoorbellPressed` | hoch (Kanalname eindeutig) |
| `dismantle.v2` | „Benachrichtigung über demontierte Klingel ohne Erlaubnis" | beliebig | `TamperDetected` | hoch |
| `lowpower.v2` | „Benachrichtigung über niedrigen Batteriezustand" | beliebig | `BatteryLow` | hoch |
| `call.v2` | „Anruf durch Familie" | beliebig | `IntercomCall` | hoch |
| `silence.v2` | „Sofortiges Benachrichtigung-Update" | i. d. R. kein sichtbarer Text | `UnknownVendorEvent` | — (Funktion unklar, siehe `apk-findings.md`) |
| `default.v2` | „Andere nicht eingeordnete Benachrichtigungen" | beliebig | `UnknownVendorEvent` | — |
| unbekannter Kanal | — | — | `UnknownVendorEvent` | — |

`DeviceOffline`/`DeviceOnline` kommen **nicht** aus Push-Benachrichtigungen, sondern aus Transport-/Verbindungsbeobachtung (bereits in `aosu-core::EventKind::Connectivity`/`System` vorhanden). In dieser Pipeline bleiben sie unbelegt/unbenutzt, bis eine reale Quelle dafür existiert.

## Nie verworfene Rohdaten

Die Python-Normalisierung (`tools/event_pipeline.py`) und der persistente Store (`StoredEvent.raw_json`) tragen den vollständigen Rohblock (Kanal, Titel, Text, `key`, Zeitstempel). Das Rust-`Event` aus `crates/aosu-core/src/notification_pipeline.rs` enthält dagegen nur ausgewählte Metadaten (`channel`, `raw_text`, `confidence`, `classification_note`, `event_id`) — der vollständige Rohblock bleibt in der JSONL-Datei bzw. im Store erhalten. Unbekannte Kanäle oder Textmuster erzeugen `UnknownVendorEvent`, nie ein verworfenes Ereignis.

## Integrationspunkt: JSONL-Datei

`tools/notification_bridge.py --jsonl-out <Pfad>` schreibt ein normalisiertes Ereignis pro Zeile (anhängend). Sowohl die Home-Assistant-Integration (`binary_sensor.py` im Modus „Notification-Bridge", `push_log_backend.py`) als auch der Desktop-Client (`NotificationLogTransport`, Registry-Name `push-log`, wählbar in Einstellungen) lesen dieselbe Datei — kein gemeinsamer Prozess, aber dieselbe Datenquelle und dasselbe Format.

## Kanonisches Vokabular im Rust-Kern

`aosu-core::EventKind` nutzt intern kurze, Rust-idiomatische Varianten (`Motion`, `Person`, `Vehicle`, `Package`, `Doorbell`, `LowBattery`, `Tamper`, `IntercomCall`). Für alles Konsumentenseitige (Tauri-DTOs, künftig auch Logs) übersetzt `EventKind::wire_name()` das in das kanonische Vokabular oben (`MotionDetected`, `HumanDetected`, …). Das ist eine bewusste Namenskonventions-Differenz zwischen den Sprachen, keine stillschweigende Abweichung — mit eigenem Test (`device::tests::wire_name_matches_the_canonical_vocabulary_from_event_pipeline_doc`) abgesichert.

## Verifizierte Parität (Stand 2026-11-22)

- `tools/event_pipeline.py::normalize()` und `crates/aosu-core/src/notification_pipeline.rs::normalize()` sind unabhängig gegen dieselben Testfälle verifiziert (gleiche Kanäle, gleiche Schlüsselwörter, gleiche Konfidenzstufen).
- `crates/aosu-core/tests/real_data.rs` liest eine **echte, nur im Kameranamen anonymisierte** Aufzeichnung (`tests/fixtures/real_capture_anonymized.jsonl`) — dieselbe Datei liegt auch unter `tests/fixtures/` für die Python-Seite (`test_ha_push_log_backend.py`). Damit ist die Parität nicht nur an synthetischen, sondern an echten Gerätedaten geprüft.

## Event Platform: persistenter Store (Stand 2026-11-22)

Die Event-Pipeline normalisiert einzelne Zeilen; der **Event-Store** macht daraus eine dauerhafte, abfragbare Historie. Zwei unabhängige, aber schema-identische Implementierungen, je eine eigene SQLite-Datei pro Prozess (kein gemeinsamer Schreibzugriff nötig, da beide Seiten dieselbe JSONL-Quelle unabhängig neu einlesen können):

- Rust: `crates/aosu-core/src/event_store.rs` (`rusqlite`, `bundled`-Feature) — genutzt vom Desktop-Client über `NotificationLogTransport` (Registry-Name `push-log`), das die SQLite-Datei `<jsonl-pfad>.sqlite` neben der JSONL-Quelle anlegt.
- Python: `tools/event_store.py` (nur Standardbibliothek, `sqlite3`) — gleiches Schema, eigenständig gegen dieselbe reale Fixture verifiziert (`tests/test_event_store.py`).

### Schema-Versionierung

Konstante `SCHEMA_VERSION = 1` (beide Sprachen). Tabelle `schema_meta` hält genau eine Zeile mit der Version, mit der die Datei angelegt wurde. Beim Öffnen einer bestehenden Datei wird die gespeicherte Version gegen die Konstante geprüft:

- stimmt sie, wird normal geöffnet.
- stimmt sie nicht, schlägt das Öffnen **laut fehl** (keine stille Migration, keine Datenverlust-Gefahr durch automatisches Schema-Ändern) — es existiert aktuell keine Migrationslogik. Eine künftige Schemaänderung muss entweder (a) `SCHEMA_VERSION` erhöhen und eine explizite Migrationsfunktion pro Sprache ergänzen, oder (b) rückwärtskompatibel bleiben (nur neue, optionale Spalten).

### Datenmodell (`StoredEvent` / `events`-Tabelle)

| Feld | Bedeutung |
|---|---|
| `id` | Eindeutige ID, deterministisch aus dem Rohinhalt der Zeile abgeleitet (Hash) — macht wiederholtes Einlesen derselben JSONL-Datei idempotent (`INSERT OR IGNORE`), kein Duplikat bei erneutem Start. |
| `timestamp_ms` | Zeitstempel in Millisekunden seit Epoch. |
| `event_type` | Kanonischer Typ aus dem Vokabular oben. |
| `channel` | Android-Kanalname (Rohwert, für Diagnose). |
| `camera_name` | Kameraname, falls bekannt (`None`/`null` möglich). |
| `confidence` | Klassifikations-Konfidenz (`hoch`/`niedrig`/…, siehe Zuordnungstabelle). |
| `classification_note` | Freitext-Begründung der Klassifikation (z. B. „generischer Text, defaulted"). |
| `raw_json` | Die komplette Original-Zeile — nie verworfen, auch nicht bei unbekanntem `event_type`. |

Funktionsumfang (identisch in beiden Sprachen, siehe Tests): Ingestion mit Dedup- und Fehlerbericht (`inserted`/`duplicates`/`malformed`, inkl. Fehlertext-Stichproben), Abfrage mit Filter (Kamera, Typ, Zeitraum, Volltextsuche) und Pagination, Geräteaktivität (Events pro Kamera, zuletzt gesehen, Kategorien), Analytics (heute, nach Kamera, nach Typ, 14-Tage-Verlauf), Export (JSON, CSV), Aufbewahrung (löschen nach Alter und/oder maximaler Zeilenzahl).
