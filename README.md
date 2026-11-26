# aosu-desktop (Arbeitstitel)

[![Auf GitHub sponsern](https://img.shields.io/badge/%E2%9D%A4%20Auf%20GitHub%20sponsern-EA4AAA?style=for-the-badge&logo=githubsponsors&logoColor=white)](https://github.com/sponsors/Ebenhoeh80)

Unabhängiges Open-Source-Projekt (MIT-Lizenz) für einen Linux- und Windows-Desktop-Client für Tuya-basierte Kamera-Ökosysteme. Aosu ist das erste unterstützte Ökosystem, nicht das einzig mögliche. Nicht mit Aosu/Glazero verbunden. Es werden weder Quellcode, Grafiken noch Marken der Original-App verwendet.

## Status: vollständige Produktoberfläche, echte Datenpfade für Inventar und Ereignisse; Medien nur mit autorisiertem Adapter

Die Anwendung ist **vollständig nutzbar ohne jede echte Kamera, ohne Cloud-Konto und ohne gelösten `local_key`** — das ist Absicht: Gerätebestand, Ereignisverlauf, Benachrichtigungs-Center, Diagnose-Center und Einstellungen funktionieren vollständig gegen simulierte Geräte. Zusätzlich gibt es echte read-only Pfade: Cloud-Inventar/Firmware und der Cloud-Ereignis-Feed (`/v1/event/getImageList`; `errno=0` live geprüft, auf dem Testkonto aktuell keine Zeilen) sowie `push-log`. Livebild, Gegensprechen, Aufnahmen, PTZ und Befehle sind in der UI bedienbar, wirken aber nur mit einem lizenzierten, verifizierten Transport. Der offizielle Tuya-Adapter ist implementiert und unit-getestet, aber noch **nicht live verifiziert**. Warum echte Authentifizierung lange ausstand: `docs/adr-0001-client-strategy.md`, `docs/adr-0002-path-reassessment.md`.

### Was ein neuer Nutzer tun kann (ohne Hardware)

1. App starten → Einführung erklärt, was echt und was simuliert ist.
2. **Gerätebestand** durchsuchen, filtern (Hersteller/Status), sortieren, gruppiert nach Hersteller — inklusive eines zweiten, bewusst fiktiven Herstellers („Kestrel Labs") zur Architekturprüfung.
3. In ein Gerät **hineinklicken**: Übersicht, **Medien & Steuerung**, Diagnose, Transport, Ereignisse, Benachrichtigungen, Metadaten. Der Medien-Tab zeigt ausschließlich echte Antworten des aktiven Transports; nicht unterstützte Funktionen erscheinen als Fehler, nie als erfundener Stream oder Befehl.
4. **Ereignisverlauf** mit Zeitachse, Schweregrad-Filter, Suche.
5. **Benachrichtigungs-Center**: als gelesen markieren, nach Schweregrad filtern.
6. **Diagnose-Center**: Fähigkeits-Report, Netzwerk-Port-Scan, Export als JSON.
7. **Einstellungen**: zwischen den registrierten Transporten wechseln (`mock`, `kestrel-mock`, `local`, `cloud`, `push-log`, `tuya-cloud`). `push-log` liest echte Push-Aufzeichnungen aus einer JSONL-Datei und liefert daraus Ereignisse, Geräteaktivität, Analytics und Export. Für `cloud` und `tuya-cloud` bleiben Passwort bzw. Tuya-Client-Secret ausschließlich in dieser Sitzung im RAM.

## Was heute funktioniert

| Teil | Pfad | Stand (geprüft 2026-11-26) |
|---|---|---|
| Rust-Kern (`Transport`-Trait, Capability-Modell, Diagnose, Registry, Event-Store) | `crates/aosu-core` | 89/89 Tests grün, 5 live-Probes bewusst ignoriert (1 E2E mit echter Aufzeichnung) |
| Echter Cloud-Login, Geräteliste, Firmware-Abfrage und Ereignis-Feed | `crates/aosu-core/src/transports/cloud` | live verifiziert: HTTP 200, `errno=0`, 5 Geräte, 5 Firmware-Versionen; `/v1/event/getImageList` antwortet `errno=0` mit beobachteten Feldern (Testkonto: 0 Zeilen). Credentials nur im RAM, Bild-URLs/IDs werden verworfen |
| Offizieller Tuya-Cloud-Adapter (eigenes IoT-Projekt, „Link App Account") | `crates/aosu-core/src/transports/tuya_cloud.rs` | kompiliert und unit-getestet; Live, Talk, DP-Befehle, PTZ-Payloads und Cloud-Playback sind verdrahtet, aber **nicht live verifiziert**. Keine APK-Secrets; Konfiguration/Token/Stream-URLs nur prozesslokal |
| Wiederverwendbares Tuya-LAN-Protokoll | `crates/tuya-lan-protocol` | 5/5 Tests grün, markenunabhängig |
| Desktop-App (Tauri + React), volle Produktoberfläche inkl. Medien-Tab | `app/` | 58/58 Vitest, 7/7 Playwright; Offline-Fallback kennt alle 6 Registry-Transporte; DTOs ohne Mock-Fallback für Live/Talk/Aufnahmen/Befehle |
| Home-Assistant-Integration (Modell + Notification-Bridge) | `homeassistant/custom_components/aosu` | 12/12 Tests grün |
| Push-Brücke, Event-Pipeline und Store | `tools/notification_bridge.py`, `tools/event_pipeline.py`, `tools/event_store.py` | 44/44 Tests grün (14 Brücke, 16 Pipeline, 10 Store, 4 LAN-Probe) |
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

Keine Extraktion von OEM-App-Schlüsseln, kein Umgehen von Schutzmaßnahmen, kein Zugriff auf fremde Konten oder Geräte. Der Tuya-Adapter nutzt ausschließlich ein eigenes IoT-Projekt und die dokumentierte OAuth-Verknüpfung „Link App Account"; `client_secret`, Token, Stream-URLs und Sitzungsdaten bleiben im RAM und werden nie geloggt oder gespeichert. Live-Medien bleiben gesperrt, solange kein belegter, lizenzrechtlich nutzbarer Pfad existiert: Gegensprechen braucht lokales go2rtc, Cloud-Aufnahmen eine öffentlich erreichbare HTTPS-Callback-URL, PTZ einen erfolgreichen STOP-Test. Siehe [SECURITY.md](SECURITY.md).

## ❤️ Support this project

<div align="center">

**Ich entwickle und pflege dieses Projekt in meiner Freizeit und trage die laufenden Kosten selbst – meine Zeit, Strom, Werkzeuge und IT-Ausstattung.**

Sponsoring ist freiwillig und begründet keinen Anspruch auf Support, Funktionen oder Antwortzeiten. Jeder Beitrag, egal wie klein, hilft. Vielen Dank! 🙏

<a href="https://github.com/sponsors/Ebenhoeh80">
  <img src="https://img.shields.io/badge/%E2%9D%A4%20Auf%20GitHub%20sponsern-EA4AAA?style=for-the-badge&logo=githubsponsors&logoColor=white" alt="GitHub Sponsors" height="42">
</a>
<a href="https://www.paypal.com/pool/9p84qTHCFr?sr=wccr">
  <img src="https://img.shields.io/badge/%E2%9D%A4%20Spenden%20via%20PayPal-PayPal-00457C?style=for-the-badge&logo=paypal&logoColor=white" alt="Spenden via PayPal" height="42">
</a>
<a href="https://buymeacoffee.com/daneben">
  <img src="https://img.shields.io/badge/%E2%98%95%20Spendier%20mir%20einen%20Kaffee-FFDD00?style=for-the-badge&logo=buymeacoffee&logoColor=black" alt="Buy Me a Coffee" height="42">
</a>

</div>

## Lizenz

MIT, siehe [LICENSE](LICENSE). Alle eigenen Erkenntnisse, Dokumente und Skripte dieses Projekts stehen unter dieser Lizenz. Fremdprojekte (z. B. `smart-aosu-cloud`, PolyForm Strict) werden nur verlinkt und bewertet, ihr Code wird nicht übernommen.
