# Ökosystem-Integration: vom Aosu-Client zur Tuya-Plattform

Dieses Dokument hält fest, welche Teile des Projekts bewusst markenunabhängig gehalten wurden, und wo die Grenze zum Aosu-Spezifischen liegt. Kein Punkt hier verzögert den primären Aosu-Client — es sind Entscheidungen, die bei gleichem Aufwand die generischere Variante wählen.

## Was bereits markenunabhängig ist

| Baustein | Ort | Status |
|---|---|---|
| Tuya-LAN-Rahmenformat (Magic-Bytes, CRC, Framing) | `crates/tuya-lan-protocol` | **Eigene Crate**, kennt kein Aosu/Glazero, nur das offene Tuya-Protokoll. Jeder Tuya-basierte Client (auch für andere Marken) kann sie verwenden. |
| `Transport`-Trait, `Device`/`Capability`-Modell | `crates/aosu-core` | Markenunabhängig: `Device.model`/`Device.kind` sind freie Strings, `Capability`-Enum kennt keine Aosu-Spezifika. |
| Transport-Registry | `crates/aosu-core/src/registry.rs` | Keine geschlossene Aufzählung mehr: `TransportRegistry::register(name, factory)` erlaubt es, einen weiteren Marken-Adapter zu registrieren, ohne Kern-Code zu ändern (kein dynamisches Nachladen von `.dll`/`.so`, sondern dasselbe Vertrauensmodell wie jede andere Rust-Abhängigkeit). |
| Passive Netzwerk-Diagnose | `crates/aosu-core/src/diagnostics.rs` | Reiner Port-Scan, kein Aosu-Bezug. |

## Was bewusst Aosu-spezifisch bleibt

- `CloudTransport` (Hostnamen `api.aosulife.com`, `Gz-Sign`-Feldnamen) — das ist naturgemäß herstellerspezifisch, gehört in `aosu-core`, nicht in die generische Crate.
- Die Home-Assistant-Integration (`homeassistant/custom_components/aosu`) trägt bewusst die Domain `aosu`, weil sie *der* Aosu-Client ist, keine Plattform. Die wiederverwendbaren Bausteine darunter (`aosu-core`, `tuya-lan-protocol`) sind trotzdem bereits so geschnitten, dass eine zweite HA-Integration für eine andere Tuya-Marke dieselben Crates nutzen könnte.
- `tools/notification_bridge.py` kennt die Aosu-App-Paket-ID und Kanalnamen fest verdrahtet — das ist angemessen, weil Benachrichtigungsformate je App unterschiedlich sind; eine Verallgemeinerung ohne zweites Beispiel wäre Spekulation.

## Geplante Integrationspunkte (noch nicht implementiert, Design vorbereitet)

Sobald ein `Transport` echte Mediendaten liefert (`live()` liefert mehr als eine Fehlermeldung), sind folgende Anschlüsse ohne Architekturänderung möglich, weil `MediaStreamHandle` bereits als eigener Typ existiert statt direkt an eine UI-Komponente gekoppelt zu sein:

- **go2rtc/Frigate:** `MediaStreamHandle` könnte eine lokal erzeugte RTSP- oder WebRTC-Quelle beschreiben, die go2rtc/Frigate als Standard-Kamera-Input konsumieren — keine Sonderintegration nötig, nur ein Adapter, der den Handle in eine go2rtc-Stream-Konfiguration übersetzt.
- **Scrypted:** gleiche Überlegung über dessen Plugin-Schnittstelle.
- **Community-Geräteadapter:** Ein zweiter Markenclient bräuchte nur eine neue `Transport`-Implementierung plus eine Registrierung in der `TransportRegistry` — UI und Home-Assistant-Bridge bleiben unverändert, weil beide ausschließlich gegen `dyn Transport` und `CapabilityReport` arbeiten, nie gegen eine konkrete Marke.

## Bewusst nicht jetzt gebaut

- Echtes dynamisches Plugin-Laden (`.dll`/`.so` zur Laufzeit) — hohes Sicherheitsrisiko (Ausführung nicht vertrauenswürdigen nativen Codes) für einen Nutzen, der mit der Compile-Zeit-Registry bereits erreicht wird.
- Eine generische „Tuya-Cloud-Client"-Abstraktion — es gibt noch keinen zweiten Cloud-Anbieter als Vergleichspunkt; das wäre Spekulation statt Verallgemeinerung aus Beobachtung.
