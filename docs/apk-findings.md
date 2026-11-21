# APK-Befunde (statische Analyse) – com.glazero.android 5.11.20

Methode: Entpacken mit REA 6.10.1, Dekompilierung mit JADX 1.5.7 (JDK 21): 43.336 Quelldateien, 264 Methoden mit Dekompilierfehlern. Nur statisch, keine Ausführung, keine Schlüsselextraktion.

## Belegt

- **Plattform:** Tuya-SDK unter dem Namespace `com.thingclips` (ca. 13.770 Quelldateien referenzieren ihn). Native Bibliotheken: `libThingCameraSDK`, `libThingP2PSDK`, `libthing_security`, `libThingSmartLink`. Das Tuya-Smart-Plugin-System (`thing_uni_plugins`, Tangram-Module) ist enthalten.
- **Eigene Cloud-Domain statt öffentlicher Tuya-Domains:** `AppChannel` (`com.glazero.biz.channel`) liefert `api.aosulife.com` (API), `home.aosulife.com` (H5), `aosulife.com` (Kanal-Host). Kanal-ID `aosu`, PID `glazero`, App-Kürzel `AS`. Öffentliche Tuya-Hosts (`tuyaeu.com` usw.) kommen im Code nicht als Klartext vor.
- **Regionale Ressourcen-Hosts:** `down-us.aosulife.com`, `down-eu.aosulife.com`, `down-cn.aosulife.com.cn` (Pfad `/app/res/<hash>`).
- **Weitere Dienste:** Firebase (`glazero-app-default-rtdb`), Getui-Push, Agora RTC/RTM, Sentry/Crashlytics, App Link `ulink.aosulife.com`.
- **Gerätearchitektur:** Neben WLAN-Kameras gibt es die Basisstation **aosuBase** (auch „aosuCortex"). Aufnahmen werden dort gespeichert, Kameras koppeln sich an sie. Kopplung per AP-Hotspot `AosuLife-XXXX`.

## Folgerungen (noch Hypothesen)

- Aosu läuft vermutlich in einer **privaten Tuya-Cloud-Instanz von Glazero**. Der öffentliche Tuya-Weg (Smart Life, Tuya-IoT-Projekt, go2rtc Smart API) funktioniert dann sehr wahrscheinlich **nicht**. Das bestätigt der Livetest an einer echten Kamera.
- Ein Client müsste die Glazero-API (`api.aosulife.com`) mit den App-Zugangsdaten ansprechen. Deren Schlüssel liegen in `libglazero-security` bzw. `libthing_security`. Eine Extraktion ist **nicht freigegeben** und wird nicht ohne ausdrückliche Entscheidung des Nutzers versucht.

## Beobachtung am Gerät (passiv, 2026-11-21)

Testgerät: Samsung SM-S901B, Android 16, App 5.11.20 (identisch mit der XAPK), angemeldet, 5 Geräte, Heim-LAN 192.168.10.0/24.
- Etablierte TCP-Verbindungen der App (UID 10319) gehen ausschließlich ins **IPv6-Internet**: `:443` (HTTPS) und **`:8883` (MQTT über TLS)**. Mindestens ein Ziel liegt im Bereich `2a05:d014::/..`, der nach meiner Einschätzung AWS eu-central-1 zugeordnet ist (nicht verifiziert).
- Im Leerlauf (Startseite, kein Livebild) keine Verbindung ins LAN. Die ARP-Tabelle des Telefons zeigt nur das Gateway.
- Folgerung (Hypothese): Steuerung und Ereignisse laufen über Cloud-MQTT. Ob das Livebild per P2P direkt zur Kamera oder über die Cloud läuft, zeigt erst eine Beobachtung während einer Livebild-Sitzung.

### Livebild-Messung (Solarkamera, Tab „Live")

- Kameras hängen hinter der Basisstation **H1L** und haben keine eigene LAN-IP. Die App hat Kamera/H1L im Test **nicht im LAN** kontaktiert: keine TCP/UDP-Verbindung zu 192.168.* während des Streams, obwohl das Telefon im selben WLAN war.
- Während des Livebilds blieben HTTPS (443), MQTT-TLS (8883) und ein langlebiger UDP-Socket nach `2600:1901::` (nach meiner Einschätzung Google-Cloud-Adressraum, nicht verifiziert) bestehen. Dazu kommt ein TCP-Ziel `57.131.x.x:1443`. Das Livebild läuft damit sehr wahrscheinlich über Cloud-/Relay-Infrastruktur und nicht lokal.
- Die App hält lokal UDP-Sockets auf **6666 und 6667**. Das entspricht den bekannten Tuya-LAN-Discovery-Ports (Broadcast). Ob die H1L darauf antwortet, ist offen und mit passivem Mitlauschen im LAN prüfbar.
- App-UI: Der Tab „Ereignisse" bietet **Cloud / Lokal**. „Lokal" spielt Aufnahmen von der Basisstation ab. Die Live-Ansicht hat Beleuchtung, Stimmwechsel, Gegensprechen und zwei Warn-Aktionen (nicht ausgelöst).
- Hinweis zum Test: Ein erster Tap traf versehentlich eine Black-Friday-Kachel und öffnete kurz die Shopseite im Browser. Dort wurde nichts bestätigt, und die App wurde per Zurück-Taste wiederhergestellt.

## Nicht geprüft / offen

- Login-Ablauf, Endpunktliste und Signaturverfahren (Klassen mit `com.glazero.*`, Retrofit-Interfaces).
- Ob Kameras/aosuBase lokale Ports oder RTSP/ONVIF anbieten (nur am Gerät im LAN prüfbar).
- Tatsächlicher Netzwerkverkehr (braucht Gerät, passive Beobachtung).
