# Aosu Home-Assistant-Integration (inoffiziell)

**Zwei Betriebsarten**, wählbar im Config-Flow:

1. **Mock** (Standard): Alle Entitäten gegen `mock_backend.py`, zeigt Demo-Geräte. Kein echter Transport.
2. **Notification-Bridge** (echte Daten): liest dieselbe normalisierte JSONL-Datei, die `tools/notification_bridge.py --jsonl-out` erzeugt — siehe `../docs/event-pipeline.md`. Das ist der erste tatsächlich mit echten Daten funktionierende Pfad, nicht nur eine Demo. Video/PTZ bleiben weiterhin unerreicht — siehe `../docs/adr-0002-path-reassessment.md`.

## Installation (zum Entwickeln/Testen)

```bash
cp -r custom_components/aosu /config/custom_components/aosu
# Home Assistant neu starten
# Einstellungen -> Geräte & Dienste -> Integration hinzufügen -> "Aosu (inoffiziell)"
# Im Config-Flow "Notification-Bridge" wählen und den Pfad zur JSONL-Datei angeben
```

Nicht über HACS verfügbar (noch kein Release, kein öffentliches Repository verlinkt).

**Wichtige Einschränkung:** `adb` läuft typischerweise auf dem Rechner, an dem das Telefon angeschlossen ist — nicht auf dem Home-Assistant-Host. Die JSONL-Datei muss also von dort aus für HA erreichbar sein (z. B. Netzwerkfreigabe oder ein Pfad innerhalb des HA-Konfigurationsverzeichnisses, falls die Brücke dort direkt schreiben kann). `tools/notification_bridge.py` selbst läuft **nicht** innerhalb von Home Assistant.

## Entitäten

**Mock-Modus:**
- `camera.*` — sechs Demo-Geräte über alle Health-Zustände (C7S + ein fiktiver zweiter Hersteller), liefern kein Bild.
- `binary_sensor.*_bewegung` — statisch `false`.
- `sensor.*_akku`, `sensor.*_wlan-signal` — Demo-Werte.

**Notification-Bridge-Modus:**
- `binary_sensor.<kamera>_bewegung` — `on`, wenn das letzte gelesene Ereignis für diese Kamera kein Tamper-Ereignis ist. Attribute `last_event_type`/`last_confidence`/`last_channel` zeigen auch unbekannte Ereignistypen (`UnknownVendorEvent`) an, statt sie zu verwerfen.
- `binary_sensor.<kamera>_manipulation` — `on` bei `TamperDetected`.
- Eine Kamera entsteht erst, sobald mindestens ein Ereignis für sie in der Datei steht — das Backend kennt keine Geräte im Voraus.

Bewusst **keine PTZ-Entität**: Alle am Testgerät beobachteten C7S-Kameras zeigten keine Schwenk-/Neige-Steuerung in der offiziellen App.

## Bekannte Lücke

Die Push-Log-Sensoren lesen die Datei bei jedem HA-Update-Zyklus neu (Polling), es gibt noch keinen echten Push-Mechanismus von der Datei in HA hinein. Für den aktuellen Entwicklungsstand ausreichend, siehe `../docs/ROADMAP.md`.

## Tests

Die reine Python-Logik (`mock_backend.py`, `push_log_backend.py`) ist ohne installiertes `homeassistant`-Paket testbar:

```bash
python -m pytest ../tests/test_ha_mock_backend.py ../tests/test_ha_push_log_backend.py
```

Die Entity-Plattformen (`camera.py`, `binary_sensor.py`, `sensor.py`, `config_flow.py`, `__init__.py`) importieren `homeassistant` und sind nur in einer echten HA-Entwicklungsumgebung lauffähig testbar (noch nicht eingerichtet).
