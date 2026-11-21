# Aosu Home-Assistant-Integration (inoffiziell, Gerüst)

**Status: Gerüst/Demo.** Alle Entitäten sind gegen `mock_backend.py` verdrahtet und zeigen Demo-Geräte. Kein echter Transport ist angeschlossen — siehe `../docs/adr-0002-path-reassessment.md` für die offenen Blocker.

## Installation (zum Entwickeln/Testen)

```bash
cp -r custom_components/aosu /config/custom_components/aosu
# Home Assistant neu starten
# Einstellungen -> Geräte & Dienste -> Integration hinzufügen -> "Aosu (inoffiziell)"
```

Nicht über HACS verfügbar (noch kein Release, kein öffentliches Repository verlinkt).

## Entitäten (Demo)

- `camera.*` — zwei Demo-Kameras (C7S), liefern aktuell kein Bild (`async_camera_image` gibt `None` zurück).
- `binary_sensor.*_bewegung` — statisch `false`, bis die Benachrichtigungs-Brücke (`../tools/notification_bridge.py`) angebunden ist.
- `sensor.*_akku`, `sensor.*_wlan-signal` — Demo-Werte.

Bewusst **keine PTZ-Entität**: Alle am Testgerät beobachteten C7S-Kameras zeigten keine Schwenk-/Neige-Steuerung in der offiziellen App.

## Tests

Die reine Python-Logik (`mock_backend.py`) ist ohne installiertes `homeassistant`-Paket testbar:

```bash
python -m pytest ../tests/test_ha_mock_backend.py
```

Die Entity-Plattformen (`camera.py`, `binary_sensor.py`, `sensor.py`, `config_flow.py`, `__init__.py`) importieren `homeassistant` und sind nur in einer echten HA-Entwicklungsumgebung lauffähig testbar (noch nicht eingerichtet).
