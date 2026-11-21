# aosu-desktop (Arbeitstitel)

Unabhängiges Open-Source-Projekt (MIT-Lizenz) mit dem Ziel eines Linux- und Windows-Desktop-Clients für Aosu-Kameras. Nicht mit Aosu/Glazero verbunden. Es werden weder Quellcode, Grafiken noch Marken der Original-App verwendet.

## Status

Lauffähiges **Projektgerüst** mit Mock-Daten; **kein echter Transport zur Kamera/Cloud ist authentifiziert.** Das ist Absicht, kein Versehen — siehe `docs/adr-0002-path-reassessment.md` für die offenen, bewusst nicht gelösten Engpässe.

- Nach der Analyse ist keine Schnittstelle validiert, auf der sich ein echter Client ohne offene Rechts-/Protokollfragen bauen ließe. Die App nutzt das Tuya-SDK mit einer eigenen Glazero-Cloud; der öffentliche Tuya-Weg (Smart Life, go2rtc) greift für Aosu nicht.
- Ein echter Port der Basisstation (H1L, TCP 6668, Tuya-Standardprotokoll) ist erreichbar, aber ohne Authentifizierungsschlüssel nicht nutzbar.
- Strategie und Entscheidungsverlauf: `docs/adr-0001-client-strategy.md`, `docs/adr-0002-path-reassessment.md`. Architektur: `docs/architecture.md`. Fortschritt: `docs/ROADMAP.md`.

## Was heute funktioniert

| Teil | Pfad | Stand |
|---|---|---|
| Rust-Kern (`Transport`-Trait, Capability-Modell, Diagnose) | `crates/aosu-core` | 17/17 Tests grün |
| Desktop-App (Tauri + React) | `app/` | 10/10 Vitest, 2/2 Playwright, zeigt Demo-Geräte |
| Home-Assistant-Integration (Gerüst) | `homeassistant/custom_components/aosu` | 3 Tests grün, Demo-Entitäten |
| Push-Benachrichtigungs-Brücke | `tools/notification_bridge.py` | 6 Tests grün, noch nicht an echte Ereignisse angeschlossen |
| Passive LAN-Diagnose | `tools/lan_probe.py` | 4 Tests grün, über die UI erreichbar |

Siehe jeweiliges `README.md` in `app/` und `homeassistant/` für Entwicklungsbefehle.

## Mitmachen: LAN-Prüfung der eigenen Basisstation

Wenn Sie eine H1L im eigenen Netz haben, hilft ein passiver Test (nur Mitlesen und Port-Check, keine Anmeldung, keine Cloud):

```
python tools/lan_probe.py --listen 120
python tools/lan_probe.py --host <IP der H1L>
```

Ergebnisse bitte ohne Seriennummern, MAC-Adressen und Konto-Daten als Issue posten.

## Grenzen

Keine Extraktion von App-Schlüsseln, kein Umgehen von Schutzmaßnahmen, kein Zugriff auf fremde Konten oder Geräte. Siehe [SECURITY.md](SECURITY.md).

## Lizenz

MIT, siehe [LICENSE](LICENSE). Alle eigenen Erkenntnisse, Dokumente und Skripte dieses Projekts stehen unter dieser Lizenz. Fremdprojekte (z. B. `smart-aosu-cloud`, PolyForm Strict) werden nur verlinkt und bewertet, ihr Code wird nicht übernommen.
