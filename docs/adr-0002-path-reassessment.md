# ADR 0002: Neubewertung — ist smart-aosu-cloud wirklich der beste Weg?

Status: entschieden 2026-11-21. Ersetzt die Priorisierung in ADR 0001, Abschnitt „Aktualisierung 2026-11-21", **Punkt H**.

## Harter Fehler in der letzten Bewertung

Ich hatte `smart-aosu-cloud` stillschweigend als „Wiederverwendung statt Eigenentwicklung" behandelt und damit implizit als risikoärmer eingestuft als ein selbstgebauter Client. Das stimmt nicht. Geprüft (nur gelesen, `oem_resource_bundle.py`): Die Integration **extrahiert exakt dieselben Geheimnisse** (App-Key, App-Secret, Zertifikat, native Schlüsseldaten) aus der vollen Aosu-APK, die ich für einen eigenen Client als gesperrt eingestuft hatte. Es ist fremder Code, der dieselbe Grenze überschreitet, nicht ein Weg, der sie umgeht. Die rechtliche Unsicherheit (§ 69e/§ 95a UrhG) und das ToS-/Kontosperre-Risiko sind **identisch**, unabhängig davon, wer den Extraktionscode geschrieben hat. Reuse senkt hier den Entwicklungsaufwand, nicht das Risiko.

## Neubewertung aller Wege

| Weg | Erfolgswahrscheinlichkeit | Aufwand | Rechtliche Unsicherheit | Wartbarkeit | Abhängigkeiten |
|---|---|---|---|---|---|
| **A. Push-Benachrichtigungen → HA-Ereignisclient** | Technisch sicher machbar; ob Ereignisse inhaltlich reichen, offen (bisher 0 erfasst in 55 Min, Grund unklar: keine Ereignisse oder Benachrichtigung deaktiviert) | klein | **keine** (eigenes Telefon, eigene Benachrichtigungen lesen ist Standardfunktion, keine ToS-Berührung) | hoch (hängt nur daran, dass Aosu weiter Push sendet, was auch Alexa/Google brauchen) | Android-Gerät mit App, dauerhaft erreichbar |
| **B. smart-aosu-cloud nutzen** | 30–40 % für Video/PTZ, *wenn* man die Extraktion durchführt | klein (Code existiert) | **gleich hoch wie Eigenbau**, s. o. | **niedrig**: Ein-Personen-Projekt, 100 Commits in 4 Tagen (= hohe Fluktuation, nicht Stabilität), PolyForm Strict verbietet uns, es bei Bedarf selbst zu reparieren/forken | Autor, exakte 5.10.12-Dateien, unbestätigtes Login-Profil |
| **C. Zusammenarbeit mit dem Autor** | unklar, einmalig schnell zu fragen, Antwortzeit nicht beeinflussbar | sehr klein | ändert die Rechtslage von B nicht, nur wer den Code schreibt | abhängig von einer Einzelperson | — |
| **D. Alternative OSS-Projekte** | keine gefunden außer B und Thingino (anderes Problem, s. E) | — | — | — | bereits recherchiert, kein weiterer Treffer erwartbar |
| **E. Thingino-artiger lokaler Firmware-Tausch** | nur für C5L-Hardware gelistet, für C7S unbestätigt; kein Cloud-Client, sondern Gegenteil (lokal, kein Konto) | hoch, Hardware-Risiko (Bricking), Garantieverlust | **gering** (eigene Hardware umflashen ist rechtlich klar) | hoch (aktives Community-Projekt) | physischer Zugriff, verfehlt die gestellte Zielsetzung „Cloud-Client" |
| **F. Tiefere Analyse von Login/Cloud-Kommunikation** | würde ohnehin wieder bei der Extraktions-Grenze enden (TLS ist verschlüsselt, Pinning-Umgehung nötig) | mittel–hoch | steigt eher (Pinning-Bypass ist selbst schon eine Grauzone) | — | — |
| **G. Lokale Schnittstelle an H1L** | im Test bisher 0 LAN-Verkehr beobachtet | klein, aber **blockiert**: H1L nur remote, kein physischer Zugriff | gering | — | **hängt an Ihnen**: Zugang zur H1L |
| **H. Offizielle Anfrage an Aosu** | gering für API, aber einzige Möglichkeit, die Grenze legal zu verschieben | sehr klein | keine | — | Antwortbereitschaft von Aosu |

## Entscheidung

**Weg A wird der primäre, sofort umsetzbare Weg.** Er ist der einzige, der tatsächlich ein „funktionierender, wartbarer, legal unstrittiger Cloud-Client" ist, wenn auch vorerst nur für Ereignisse, nicht für Video. Das ist kein Kompromiss aus Bequemlichkeit, sondern das einzige Ergebnis, das die gestellten Kriterien (Aufwand, Risiko, Wartbarkeit) tatsächlich erfüllt.

**Weg B (smart-aosu-cloud) wird von „primärer Weg" auf „zurückgestellt" herabgestuft.** Er bleibt dokumentiert, aber nicht verfolgt, bis entweder (a) eine echte juristische Einschätzung zur Extraktionsfrage vorliegt, oder (b) Sie ausdrücklich entscheiden, das Risiko selbst zu tragen und ich nur noch bei der Konfiguration helfe (nicht bei der Extraktion selbst mitwirke). Das ist unverändert zur letzten Festlegung.

**Weg C (Autor kontaktieren)** bleibt als Nebenaktion bestehen, ändert aber nichts an der Priorität, weil sie das Kernproblem (Rechtslage) nicht löst.

**Video bleibt vorerst ungelöst.** Das ist ehrlicher als es mit B zu überdecken.

## Nächster konkreter Schritt (keine weitere Recherche)

1. Monitor für Push-Benachrichtigungen erneut laufen lassen, diesmal mit einem **bewussten Auslöser** (Bewegung vor einer Kamera), um echten Inhalt zu bekommen — das entscheidet, ob Weg A auch Kamera-Zuordnung kann.
2. Wenn Inhalt brauchbar ist: HA-Automatisierung/Blueprint bauen (Kanal- oder Text-Parsing → Ereignis-Entität), das ist der tatsächliche „Cloud-Client" für Ereignisse.
3. Video bleibt offen, bis H1L-Zugriff (Weg G) oder Aosu-Antwort (Weg H) vorliegt.
