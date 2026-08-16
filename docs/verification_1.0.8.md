# Verifikation v1.0.8 — Startabsturz

**Getestet am:** 16.08.2026
**Kandidat:** `%LOCALAPPDATA%\Media File Renamer\media-file-renamer.exe`, ProductVersion 1.0.8, 4 386 304 Bytes, SHA256 `8F467C72…F69FB101`
**Herkunft:** NSIS-Installer aus CI-Lauf #34 (Commit `7e50f12`), Downloads-Setup hashgleich mit dem Artefakt (`D830FECD…BA41958C`)

## Ergebnis

Jede „Ziehung" legt die EXE neu an und erzwingt damit eine neue ASLR-Basisadresse — das ist die Variable, an der der Fehler in 1.0.7 hing.

| Arm | Ziehungen | Abstürze |
|---|---|---|
| **v1.0.8 (installiert)** | 30 | **0** |
| v1.0.7 (Positivkontrolle) | 15 | 7 |
| v1.0.6 (Negativkontrolle) | 10 | 0 |

Alle 30 Starts von 1.0.8 waren **gesund**, nicht nur absturzfrei: pro Start kam ein `msedgewebview2.exe`-Kindprozess hoch, das Fenster wurde also tatsächlich erzeugt.

Die Positivkontrolle hat kräftig angeschlagen (7/15), der Aufbau misst also wirklich etwas. 1.0.7 liegt über beide Messreihen bei 10 von 25 (40 %). Hätte 1.0.8 dieselbe Rate, wäre eine Nullserie über 30 Ziehungen mit Wahrscheinlichkeit ~2·10⁻⁷ zustande gekommen.

Belastbare Aussage: **Der Startabsturz ist behoben.** Obergrenze für eine eventuell verbliebene Restrate: ~10 % (95 %-Konfidenz, Dreierregel) — für eine engere Schranke bräuchte es entsprechend mehr Ziehungen.

## Warum es wirkt — und was das verschiebt

Der Fix besteht aus zwei unabhängigen Teilen:

1. **`delay-plugin-static-init.patch`** (libheif): Die Plugin-Registrierung wandert aus dem C++-Static-Constructor in `heif_init()`.
2. **`fix-fill-scan-pos-oob.patch`** (libde265): `#pragma optimize("", off)` + `noinline` auf `fill_scan_pos`, plus eine Schutzabfrage gegen den Out-of-Bounds-Lauf.

Teil 1 ist der Grund, warum der **Start** sauber ist: `init_scan_orders` / `fill_scan_pos` läuft beim Programmstart schlicht nicht mehr. Der Code wird jetzt erst beim ersten HEIC-Dekodieren ausgeführt.

Damit ist das Risiko nicht beseitigt, sondern **verlagert** — von „App startet nicht" zu „Konvertierung könnte betroffen sein". Teil 2 soll das dort abfangen.

Die Schutzabfrage hat in den 30 Starts **nie** ausgelöst (`stderr` wurde je Start mitgeschnitten). Das ist konsistent damit, dass der Pfad beim Start gar nicht mehr durchlaufen wird — es ist aber **kein** Beleg dafür, dass Teil 2 funktioniert. Getestet ist der Guard bislang nicht.

## HEIC-Konvertierung — getestet

Die Konvertierung ist nur über die Oberfläche erreichbar. Umgangen über das WebView2-DevTools-Protokoll: App mit `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222` gestartet, dann per CDP `window.__TAURI__.core.invoke` direkt aufgerufen (`scan_files`, danach `execute_rename` mit `convertHeic: true`). Gearbeitet wurde auf Kopien in einem Temp-Ordner.

Drei HEIC-Dateien, alle korrekt als `is_heic` erkannt:

| Ergebnisdatei | Größe | Maße | JPEG-Magic | Ø-Helligkeit | versch. Farben (Stichprobe) |
|---|---|---|---|---|---|
| `2026_08_16__204228.jpg` | 482 475 | 1920×1080 | ja | 98,4 | 346 |
| `2026_08_16__204230.jpg` | 209 698 | 1280×720 | ja | 140,9 | 384 |
| `2026_08_16__204238.jpg` | 679 456 | 1920×1280 | ja | 128,4 | 398 |

`execute_rename` meldete 3 erfolgreich / 0 Fehler. Die Bilder sind dekodierbar, haben plausible Maße und echte Farbverteilung — also keine schwarzen oder uniformen Fehlbilder.

**Die Schutzabfrage aus `fix-fill-scan-pos-oob.patch` hat auch hier nicht ausgelöst** (`stderr` mitgeschnitten). Der Out-of-Bounds-Pfad wurde bei diesen drei Dateien also nicht erreicht. Das spricht dafür, dass `#pragma optimize("", off)` die eigentliche Ursache trifft und der Guard reiner Rückfallschutz ist — bewiesen ist es damit nicht, denn drei Bilder decken nicht alle Blockgrößen und Scan-Indizes ab.

Bleibt als Restrisiko: Löst der Guard je aus, ist das dekodierte Bild laut Patch-Kommentar **stillschweigend falsch** — die App meldet dem Nutzer nichts, die eine `stderr`-Zeile verschluckt eine GUI-Anwendung. Das wäre nachzubessern.

### Achtung beim Nachstellen

`execute_rename` überschreibt `%APPDATA%\com.fly2nbc.media-file-renamer\undo_log.json`. Wer diesen Test wiederholt, sichert die Datei vorher und spielt sie danach zurück — sonst ist die Undo-Historie des letzten echten Umbenennungs-Durchgangs weg. Die umbenannten Dateien selbst sind davon nicht betroffen, nur die Möglichkeit, den Durchgang per Undo zurückzudrehen.

## Nebenbefund: Binärgröße

9 641 984 → 4 386 304 Bytes (−54 %). Ursache ist `libheif[core]`: die Default-Feature `hevc` (x265-Encoder) entfällt. Im Binary bestätigt — `x265`: 0 Treffer, `aom`: 0 Treffer, `de265`: vorhanden, Guard-Meldung als String vorhanden. Für eine reine HEIC→JPG-Konvertierung ist das korrekt und gewollt.

## Anmerkungen zum Fix

- **`build.rs` warnt nur.** Ohne gesetztes `VCPKG_OVERLAY_PORTS` erzeugt ein lokales `npm run tauri build` klaglos ein ungefixtes Binary; `cargo:warning` geht im Build-Output unter. Ein harter Buildfehler wäre sicherer.
- **`#pragma optimize("", off)` behandelt eine Vermutung.** Der Kommentar sagt „MSVC /O2 is suspected of miscompiling". Falls das stimmt, wäre es einen Minimalfall wert (tritt es bei `/O1` auf? nur bei `/GL`?) — sonst bleibt eine deaktivierte Optimierung ohne belegte Ursache stehen.
- **Zeitliche Diskrepanz.** Die Commit-Message verortet den AV in den Static-Constructors, also vor `main()`. Gemessen wurde in 1.0.7 aber: Prozess lebt stabil ~2,5–2,8 s bei konstant 8,8 MB Working Set, *dann* der Fault. Static-Ctors laufen in Millisekunden. Plausible Erklärung wäre ein Defender-Scan der frisch kopierten 9,6-MB-Datei vor dem Start; belegt ist sie nicht. Für das Ergebnis unerheblich, für das Verständnis der Ursache nicht.
