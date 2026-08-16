# Startabsturz v1.0.7 — Analyse

**Status:** Behoben im aktuellen Stand (vcpkg-overlay); noch nicht in einem Release
**Analysiert am:** 16.08.2026
**Betroffen:** Media File Renamer 1.0.7 (Windows, NSIS-Installation)
**Symptom:** Die App startet nach der Installation einmal und lässt sich danach nicht mehr starten. Kein Fenster, keine Fehlermeldung.

---

## Kurzfassung

v1.0.7 enthält einen latenten Speicherfehler. Ob er zuschlägt, hängt **ausschließlich von der Basisadresse ab, an die Windows die EXE lädt** (ASLR).

Die Adresse wird gezogen, wenn die Datei angelegt wird, und bleibt für diese Datei-Instanz konstant. Ein Neustart zieht sie neu. Etwa jede dritte Adresse ist „schlecht" — dann stürzt **jeder** Startversuch ab, bis die Adresse neu gezogen wird.

Der Absturz hat also nichts mit dem „zweiten Start" zu tun, sondern mit dem Reboot dazwischen.

---

## Absturzbild

| | |
|---|---|
| Ausnahmecode | `0xC0000005` (Zugriffsverletzung) |
| Fehleroffset (RVA) | `0x7a4e2c` — bei jedem Absturz identisch |
| Fehlerhaftes Modul | `media-file-renamer.exe` selbst |
| Zeit bis Absturz | ~2,5 s nach Prozessstart |
| WER-Bucket | `ea40e368c77e24aeecbe2985151a1ece` (alle Abstürze derselbe) |

Der Absturz passiert **vor der Erzeugung des WebView2-Fensters**:

- nur 26 geladene Module, kein `msedge*` / WebView2 darunter
- kein `msedgewebview2.exe`-Kindprozess (Kontrolle: CSV Viewer erzeugt ihn binnen 200 ms)
- das Profilverzeichnis `EBWebView` wird nicht neu angelegt
- `MainWindowHandle` bleibt 0

Damit liegt der Fehler im Rust-/Tauri-Startpfad, nicht im Frontend. Eigener Code läuft dort noch nicht: `lib.rs` ist in 1.0.6 und 1.0.7 identisch und enthält vor `Builder::run()` keine Logik, keine Statics, kein `unsafe`.

---

## Beweisführung

### 1. Bytegleiche Dateien verhalten sich unterschiedlich

Zwei Kopien derselben EXE, identischer SHA-256, getrennt angelegt, verschränkt gemessen mit sauberem Abräumen zwischen den Läufen:

```
k1.exe   Basis 0x7FF6655F0000   -> Absturz  Absturz  Absturz  Absturz
k2.exe   Basis 0x7FF6B9700000   -> läuft    läuft    läuft    läuft
```

### 2. Es ist die Datei-Instanz, nicht Pfad, Name oder Inhalt

```
ab\media-file-renamer.exe        -> XXXXXX   6/6 Abstürze
ab\andererName.exe               -> ......   0/6   (bytegleich!)
names\media-file-renamer.exe     -> ......   0/6   (gleicher Name, anderes Verzeichnis)
names\renamer.exe                -> XXXXXX   6/6
```

Löschen und Neukopieren an *demselben* Pfad kippt das Verhalten — die Eigenschaft hängt an der neu angelegten Datei, nicht am Pfad.

### 3. Die Basisadresse ist pro Instanz konstant

Gemessen über `MainModule.BaseAddress`: gleiche Datei → immer dieselbe Basis → immer dasselbe Ergebnis. Neue Datei → neue Basis → neue Ziehung.

### 4. Absturzrate

Über 10 frische Datei-Instanzen je Version:

| Version | Abstürze |
|---|---|
| 1.0.7 | 3 / 10 |
| 1.0.6 | 0 / 10 |

Über alle sauber gemessenen 1.0.7-Instanzen: rund 9 von 25 (~36 %).

---

## Passt zur beobachteten Historie

| Zeitpunkt | Ereignis |
|---|---|
| 15.08. 13:09 | Boot |
| 15.08. 13:12 | 4 Absturzversuche (Adresse dieses Boots war schlecht) |
| 15.08. 13:13:39 | Neuinstallation → neue Datei → gute Adresse |
| 15.08. 13:13–13:14 | App lief ~70 s, schrieb `undo_log.json` und EBWebView |
| 15.08. 20:29 / 16.08. 16:58 | Neustarts → Adresse neu gezogen |
| ab 16.08. 17:22 | jeder Start stürzt ab, durchgängig Basis `0x7FF74E810000` |

Der Cluster am 14.08. (23:28, 23:29, 00:35) folgt demselben Muster: ein Boot, eine schlechte Adresse, alle Versuche scheitern.

---

## Ausgeschlossen

Jeweils gemessen, nicht vermutet:

- **App-Daten** — `undo_log.json` beiseitegelegt: stürzt weiter ab. `EBWebView` beiseitegelegt: stürzt weiter ab. Beides zusammen entfernt: stürzt weiter ab. Das JSON ist gültig.
- **WebView2-Runtime** — 151.0.4129.86 installiert und intakt; CSV Viewer und VoxMD (Build vom selben Tag, 04.08.) starten sauber. Auch `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` explizit gesetzt ändert nichts.
- **Netzwerk / Updater** — keine einzige TCP-Verbindung während der Laufzeit.
- **AppCompat / IFEO** — keine Layers, keine Image File Execution Options, keine App-Paths-Einträge, kein `apphelp.dll` geladen.
- **Exploit-Schutz** — keine system- oder prozessspezifischen Mitigations gesetzt.
- **Arbeitsverzeichnis, Dateiname, Dateipfad** — alle einzeln kontrolliert.
- **Datei beschädigt?** Nein. Der 3-Byte-Unterschied zwischen installierter EXE und dem nackten Release-Asset ist der Bundler-Stempel `__TAURI_BUNDLE_TYPE_VAR_UNK` → `_NSS` (`tauri-utils/src/platform.rs:349`), also gewollt. Ein von Hand identisch gepatchtes Binary ist hashgleich zur Installation.
- **Datenträger** — alle vier Laufwerke `Healthy`, keine Ntfs-/Disk-Fehler.

---

## Abhilfe

### Sofort: Adresse neu ziehen

Löschen und neu kopieren erzeugt eine neue Datei-Instanz und damit eine neue Ziehung. Am 16.08. beim 3. Versuch erfolgreich (`0x7FF6FA910000`), Dateiinhalt unverändert.

```powershell
$e = "$env:LOCALAPPDATA\Media File Renamer\media-file-renamer.exe"
Copy-Item $e "$env:TEMP\mfr.bak" -Force
Remove-Item $e -Force; Copy-Item "$env:TEMP\mfr.bak" $e -Force
```

Schleife bis zum Erfolg: `scripts/rebase-workaround.ps1`.

**Hält nur bis zum nächsten Neustart.**

### Downgrade auf 1.0.6

Über 10 Ziehungen sauber. Aber vermutlich nur eine Maskierung: geänderter Code verschiebt das Binär-Layout und damit, welche Adressen unglücklich liegen. Der Defekt muss nicht in 1.0.7 entstanden sein.

### Nicht empfohlen

`Set-ProcessMitigation -Name media-file-renamer.exe -Disable BottomUp,HighEntropy` friert die Adresse über Neustarts ein — es ist aber Glückssache, ob es eine gute ist. Rückgängig mit `-Enable`.

---

## Behebung (im aktuellen Stand)

WER-Offset `0x7a4e2c` ist `libde265::fill_scan_pos`, aufgerufen aus dem C++-Static-Constructor `Register_Default_Plugins` → `de265_init()` **vor** `main()`.

Umgesetzt in `vcpkg-overlay/` und der Windows-CI:

1. **libheif:** Plugin-Registrierung nicht mehr im Static-Constructor, sondern in `heif_init()` (erster `LibHeif::new()` bei HEIC-Konvertierung).
2. **libde265:** MSVC-Optimierung für `scan.cc` aus, plus Abbruch wenn `lastSubBlock < 0` (kein unbegrenzter OOB-Walk mehr).
3. **CI:** `vcpkg install "libheif[core]:x64-windows-static-md" --overlay-ports=vcpkg-overlay` — ohne x265-Encoder.

Ein neues Windows-Release muss mit diesem Overlay gebaut werden. 1.0.7 bleibt betroffen.

---

## Nebenbefund (unabhängig vom Absturz)

1.0.7 hat `security.csp` von `null` auf

```
default-src 'self'; connect-src ipc: http://ipc.localhost https://ipc.localhost;
img-src 'self' data: blob:; style-src 'self' 'unsafe-inline'; script-src 'self';
object-src 'none'; base-uri 'self'; frame-ancestors 'none'
```

gesetzt, bei gleichzeitig `withGlobalTauri: true`. Prüfen, ob das Frontend damit noch vollständig lädt — `script-src 'self'` kollidiert typischerweise mit Inline-Skripten.

---

## Rohdaten

Messskripte der Analyse (Session-Scratchpad, nicht dauerhaft):
`abtest.ps1`, `nametest.ps1`, `ratetest.ps1`, `pathtest.ps1`, `basetest.ps1`, `versiontest.ps1`, `fix.ps1`

Dauerhaft übernommen: `scripts/rebase-workaround.ps1`
