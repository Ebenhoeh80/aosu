# aosu-desktop (Arbeitstitel)

Unabhängiges Open-Source-Projekt (MIT-Lizenz) mit dem Ziel eines Linux- und Windows-Desktop-Clients für Aosu-Kameras. Nicht mit Aosu/Glazero verbunden. Es werden weder Quellcode, Grafiken noch Marken der Original-App verwendet.

## Status: Forschungsphase, kein lauffähiger Client

Es gibt bewusst noch keinen Client. Nach der Analyse ist keine Schnittstelle validiert, auf der sich ein Client ohne Spekulation bauen ließe:

- Die App nutzt das Tuya-SDK mit einer **eigenen Glazero-Cloud**. Die öffentlichen Tuya-Wege (Smart Life, go2rtc Smart API) sind für Aosu sehr wahrscheinlich nicht nutzbar.
- Im Test lief das Livebild ohne LAN-Verkehr zur Kamera oder Basisstation (H1L) über Cloud-Verbindungen.
- Ob die H1L eine lokale Schnittstelle anbietet, ist offen und nur an ihrem Standort prüfbar.

Strategie: [`docs/adr-0001-client-strategy.md`](docs/adr-0001-client-strategy.md). Kurz: kein nachgebauter Cloud-Client, sondern eine "Aosu Bridge" aus legalen Bausteinen (Ereignisse über die Push-Benachrichtigungen der offiziellen App, offizielle Anfrage an den Hersteller, lokale Prüfung der Basisstation).

Ergebnisse: [`docs/apk-findings.md`](docs/apk-findings.md), [`docs/recommendation-report.md`](docs/recommendation-report.md), Zielarchitektur: [`docs/architecture.md`](docs/architecture.md).

## Mitmachen: LAN-Prüfung der Basisstation

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
