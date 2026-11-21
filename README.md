# aosu-desktop (Arbeitstitel)

Unabhängiges Open-Source-Projekt (MIT-Lizenz) für einen Linux- und Windows-Desktop-Client für Tuya-basierte Kamera-Ökosysteme. Aosu ist das erste unterstützte Ökosystem, nicht das einzig mögliche. Nicht mit Aosu/Glazero verbunden. Es werden weder Quellcode, Grafiken noch Marken der Original-App verwendet.

## Status: vollständige Produktoberfläche, echte Hardware (noch) nicht angebunden

Die Anwendung ist **vollständig nutzbar ohne jede echte Kamera, ohne Cloud-Konto und ohne gelösten `local_key`** — das ist Absicht: Gerätebestand, Ereignisverlauf, Benachrichtigungs-Center, Diagnose-Center und Einstellungen funktionieren vollständig gegen simulierte Geräte, bevor irgendeine Protokollfrage gelöst ist. Warum echte Authentifizierung aussteht: `docs/adr-0001-client-strategy.md`, `docs/adr-0002-path-reassessment.md`.

### Was ein neuer Nutzer tun kann (ohne Hardware)

1. App starten → Einführung erklärt, was echt und was simuliert ist.
2. **Gerätebestand** durchsuchen, filtern (Hersteller/Status), sortieren, gruppiert nach Hersteller — inklusive eines zweiten, bewusst fiktiven Herstellers („Kestrel Labs") zur Architekturprüfung.
3. In ein Gerät **hineinklicken**: Übersicht, Diagnose, Transport, Ereignisse, Benachrichtigungen, Metadaten.
4. **Ereignisverlauf** mit Zeitachse, Schweregrad-Filter, Suche.
5. **Benachrichtigungs-Center**: als gelesen markieren, nach Schweregrad filtern.
6. **Diagnose-Center**: Fähigkeits-Report, Netzwerk-Port-Scan, Export als JSON.
7. **Einstellungen**: zwischen den registrierten Transporten wechseln (`mock`, `kestrel-mock`, `local`, `cloud`).

## Was heute funktioniert

| Teil | Pfad | Stand |
|---|---|---|
| Rust-Kern (`Transport`-Trait, Capability-Modell, Diagnose, Registry) | `crates/aosu-core` | 31/31 Tests grün |
| Wiederverwendbares Tuya-LAN-Protokoll | `crates/tuya-lan-protocol` | 5/5 Tests grün, markenunabhängig |
| Desktop-App (Tauri + React), volle Produktoberfläche | `app/` | 41/41 Vitest, 6/6 Playwright |
| Home-Assistant-Integration (Gerüst) | `homeassistant/custom_components/aosu` | 16/16 Tests grün |
| Push-Benachrichtigungs-Brücke | `tools/notification_bridge.py` | 6/6 Tests grün |
| Passive LAN-Diagnose | `tools/lan_probe.py` | über die UI erreichbar |

Siehe jeweiliges `README.md` in `app/` und `homeassistant/` für Entwicklungsbefehle, `docs/ROADMAP.md` für den Fortschritt, `docs/ecosystem-integration.md` für die Plattform-Architektur.

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
