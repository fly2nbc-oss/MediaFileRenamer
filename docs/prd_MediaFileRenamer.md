# One-pager: Media File Renamer

## 1. TL;DR
Eine Desktop-Anwendung (Tauri v2, Rust + Vanilla TypeScript) für Linux, die es Nutzern ermöglicht, große Mengen an Foto- und Videodateien aus unterschiedlichen Quellen automatisiert nach einheitlichen Namenskonventionen umzubenennen. Die App extrahiert Datum und Uhrzeit aus EXIF-Daten, bietet flexible Zeitanpassungen, schützt Originaldateien durch optionale Backups und kann HEIC-Dateien automatisch nach JPG konvertieren.

## 2. Goals

### Business Goals
* Entwicklung einer leichtgewichtigen, plattformübergreifenden Lösung für ein häufiges Problem bei der Medienverwaltung
* Aufbau einer soliden Basis für potenzielle zukünftige Erweiterungen (Cloud-Sync, etc.)
* Schaffung eines Werkzeugs, das sich durch Einfachheit und Zuverlässigkeit von bestehenden Lösungen abhebt

### User Goals
* Schnelles und zuverlässiges Umbenennen großer Mengen von Mediendateien mit konsistenter Namensgebung
* Automatische Extraktion von Aufnahmedatum und -zeit aus Metadaten
* Korrektur falscher Zeitstempel (z.B. bei Zeitzonenproblemen oder falsch eingestellten Kameras)
* Schutz vor Datenverlust durch optionale Backup-Funktion
* Automatische Konvertierung von HEIC-Dateien zu JPG

### Non-Goals
* Bildbearbeitung oder allgemeine Videokonvertierung
* Cloud-Storage-Integration im ersten Release
* Organisieren von Dateien in Ordnerstrukturen (nur Umbenennung)
* Unterstützung für macOS/Windows in Phase 1 (nur Linux)

## 3. User stories

**Persona: Hobby-Fotograf Michael**
* "Als Fotograf mit mehreren Kameras möchte ich Bilder aus verschiedenen Quellen einheitlich benennen, damit ich mein Archiv chronologisch sortieren kann."
* "Als jemand, der oft die Kamerazeit falsch einstellt, möchte ich Zeitstempel nachträglich korrigieren können, ohne jede Datei manuell zu bearbeiten."

**Persona: Familien-Archivar Sarah**
* "Als Person, die Familienfotos aus 20+ Jahren digitalisiert, möchte ich alle Dateien in ein einheitliches Format bringen, damit ich sie später leicht finden kann."
* "Als vorsichtige Nutzerin möchte ich sicherstellen, dass meine Originaldateien erhalten bleiben, falls beim Umbenennen etwas schiefgeht."
* "Als iPhone-Nutzerin möchte ich, dass meine HEIC-Bilder automatisch ins JPG-Format konvertiert werden."

**Persona: Event-Videograf Tom**
* "Als Videograf, der Material von mehreren Kameras zusammenführt, möchte ich Zeitstempel synchronisieren können, damit ich Clips leichter zuordnen kann."

## 4. Functional requirements

### Implementiert (Phase 1 + Phase 2)

* **Datei-Management**
  * Hinzufügen von Dateien per Button ("Add") mit nativem Dateiauswahl-Dialog
  * Hinzufügen von Dateien und Ordnern per Drag & Drop (rekursive Ordner-Durchsuchung)
  * Entfernen einzelner oder mehrerer Dateien aus der Liste (Checkboxen + "Remove"-Button)
  * Anzeige der zu bearbeitenden Dateien in scrollbarer Tabelle mit Spalten: Checkbox, #, Current Name, Date Found, New Name
  * Bei gleichen Ziel-Dateinamen fortlaufende Nummerierung (_1, _2, …)
  * Starten des Umbenennens per "Rename"-Button
  * Bestätigungsdialog bei >500 Dateien
  * Alle Buttons mit SVG-Icons

* **Namensformate**
  * Format 1: `YYYY_MM_DD__hhmmss` (z.B. 2024_03_15__143052) — voreingestellt
  * Format 2: `YYMMDD_hhmmss` (z.B. 240315_143052)
  * Format 3: `YYMMDD_originalname` (z.B. 240315_IMG_1234)
  * Auswahl per Dropdown in der Toolbar

* **EXIF-Verarbeitung**
  * Automatische Extraktion von Datum/Uhrzeit aus EXIF-Daten via `kamadak-exif` (JPEG, TIFF, HEIF)
  * Video-Datum: Parsing des MP4/MOV `mvhd`-Atoms für Creation-Time
  * Fallback auf Datei-Änderungsdatum, falls keine EXIF/Video-Metadaten vorhanden
  * Datumsquelle als farbiges Badge in der Dateiliste: EXIF (blau), File (gelb), None (rot)
  * EXIF-Schreiben: Bei Offset-Anwendung werden EXIF-Daten via `exiftool` (extern) aktualisiert

* **Zeit-Offset**
  * Kompaktes Eingabefeld für Offset in Sekunden (+/−) in der Toolbar
  * Aufklappbare erweiterte Eingabe für Jahre/Monate/Tage/Stunden/Minuten/Sekunden
  * Anwendung des Offsets auf: Dateinamen, Dateizeiten (Modified) und EXIF-Daten
  * Live-Preview des neuen Dateinamens (debounced, 100ms)

* **Backup-Funktion**
  * Optionale Checkbox "Backup" in der Toolbar
  * Automatische Erstellung eines Unterverzeichnisses `backup_YYYYMMDD_HHMMSS` pro Quellordner
  * Kopie der Originaldateien vor dem Umbenennen

* **HEIC-zu-JPG-Konvertierung**
  * Optionale Checkbox "HEIC→JPG" in der Toolbar (standardmäßig aktiviert)
  * Konvertierung via `libheif-rs` (eingebettet, kein externes Tool nötig)
  * JPEG-Qualität: 90%
  * EXIF-Metadaten werden automatisch übertragen
  * Original-HEIC-Datei wird nach erfolgreicher Konvertierung gelöscht
  * Bei aktivem Backup wird das HEIC-Original vorher gesichert

* **Fortschrittsanzeige (Phase 2)**
  * Modales Overlay mit Fortschrittsbalken während des Umbenennens
  * Anzeige: aktueller Dateiname, X von Y, Prozent-Balken
  * Rust-Backend emittiert `rename-progress` Events via Tauri

* **Undo-Funktion (Phase 2)**
  * "Undo Last"-Button in der Fußleiste
  * Speicherung eines Undo-Logs (JSON) im Tauri App-Data-Verzeichnis
  * Rückgängigmachen der letzten Umbenennung (Dateinamen zurücksetzen)
  * Nur die letzte Operation ist rückgängig machbar

* **Fehlerprotokoll (Phase 2)**
  * Fehler werden gesammelt und nach dem Umbenennen in einem ausklappbaren Panel angezeigt
  * Jeder Fehler zeigt: Dateiname + Fehlerbeschreibung

### Unterstützte Dateitypen
* **Bilder**: JPG, JPEG, PNG, HEIC, HEIF, TIFF, TIF, WEBP, GIF, BMP
* **Videos**: MP4, MOV, AVI, MKV

### Could-Have (Zukünftig)
* Benutzerdefinierte Namensformate
* Vorschau-Modus mit Before/After-Ansicht
* Automatisches Sortieren in Ordnerstrukturen
* Erkennung von Duplikaten
* Windows/macOS-Unterstützung

## 5. User experience

### UI-Layout
Kompakte, einzeilige Toolbar mit allen Controls, getrennt durch vertikale Separatoren:

```
[+ Add] [− Remove] │ [Format ▾] │ [Offset __s ▾] │ [☐ Backup] [☑ HEIC→JPG]
```

Darunter die scrollbare Dateiliste (oder Drag & Drop-Platzhalter im Leerzustand).

Fußleiste mit Undo-Button (links), Statustext (Mitte), Rename-Button (rechts).

### System-Theme
* Automatische Anpassung an Light/Dark-Mode via `prefers-color-scheme`
* CSS Custom Properties für konsistente Farbgebung

### Haupt-User-Journey
1. Nutzer startet die Anwendung
2. Drag & Drop von Dateien/Ordnern in die Hauptfläche ODER Klick auf "Add"
3. Dateiliste zeigt alle hinzugefügten Dateien mit aktuellem Namen, Datumsquelle und erkanntem Datum
4. Nutzer wählt gewünschtes Namensformat aus Dropdown
5. Optional: Nutzer gibt Zeit-Offset ein (z.B. "-7200" für −2 Stunden)
6. Optional: Nutzer aktiviert "Backup"
7. Preview zeigt neue Dateinamen live an
8. Klick auf "Rename X Files" führt Operation aus
9. Fortschrittsbalken zeigt den Fortschritt
10. Erfolgs-/Fehlermeldung wird angezeigt

### Edge Cases & UI-Hinweise
* **Keine EXIF-Daten**: Gelbes "File"-Badge, Warnung im Preview, Fallback auf Dateidatum
* **Kein Datum verfügbar**: Rotes "None"-Badge, Datei wird beim Umbenennen übersprungen
* **Namenskonflikte**: Bei Duplikaten innerhalb des Batches automatisch Suffix (_1, _2, …); bei Konflikten auf der Festplatte ebenfalls automatische Auflösung
* **Fehlgeschlagene Operationen**: Klare Fehlermeldungen mit Dateinamen und Grund im Error-Panel
* **Leere Liste**: Zentrierter Platzhalter mit Drag & Drop-Icon und Hinweistext
* **Ungültige Dateitypen**: Werden beim Scannen automatisch ignoriert
* **Große Dateimengen**: Bestätigungsdialog bei >500 Dateien
* **HEIC-Konvertierung**: Immer verfügbar (libheif-rs eingebettet)

## 6. Narrative

**Ein Tag im Leben von Michael**

Es ist Sonntagmorgen. Michael sitzt mit seinem Kaffee vor dem Laptop und schaut auf die 847 Fotos von seiner letzten Urlaubsreise. Drei Kameras, zwei Smartphones – und jede Datei hat ein anderes Namensschema. "IMG_4523.jpg", "DSC02891.jpg", "20240215_143052.jpg" – ein Durcheinander. Dazu kommen noch 120 HEIC-Bilder vom iPhone.

Er öffnet den Media File Renamer. Die kompakte Toolbar bietet alles auf einen Blick. Er zieht den gesamten Urlaubsordner per Drag & Drop in die App. Innerhalb von Sekunden erscheinen alle 967 Dateien in einer übersichtlichen Liste. Die App hat automatisch die EXIF-Daten ausgelesen – jedes Foto zeigt sein Aufnahmedatum mit blauem "EXIF"-Badge.

Dann fällt ihm auf: Die Kompaktkamera hatte die falsche Zeitzone eingestellt, alle Bilder sind 2 Stunden zu spät. Kein Problem. Er gibt "-7200" in das Offset-Feld ein. Die Preview aktualisiert sich sofort – perfekt.

Er wählt das Format "YYYY_MM_DD__hhmmss", lässt "HEIC→JPG" aktiviert, aktiviert "Backup" (sicher ist sicher), und klickt auf "Rename 967 Files". Der Fortschrittsbalken läuft durch. Fünfzehn Sekunden später ist alles erledigt. 967 Dateien, chronologisch perfekt sortiert, alle mit demselben Namensschema, alle HEIC-Bilder als JPG. Die Originale liegen sicher im Backup-Ordner.

Michael lehnt sich zurück. Was früher Stunden gedauert hätte, war in zwei Minuten erledigt. Zeit für mehr Kaffee.

## 7. Success metrics

### Nutzungsmetriken
* Durchschnittliche Anzahl umbenannter Dateien pro Session
* Häufigkeit der Nutzung der Offset-Funktion
* Adoption-Rate der Backup-Funktion
* Nutzungsrate der HEIC-Konvertierung

### Performance-Metriken
* Verarbeitungszeit für 1000 Dateien < 30 Sekunden
* App-Startzeit < 2 Sekunden
* Erfolgsrate der Umbenennung > 99%

### Qualitätsmetriken
* Fehlerrate bei EXIF-Extraktion < 1%
* Anzahl der gemeldeten Bugs im ersten Monat nach Release
* User-Satisfaction-Score (falls Feedback implementiert wird)

## 8. Technische Architektur

### Tech Stack
* **Framework**: Tauri v2 (Rust Backend + WebView Frontend)
* **Frontend**: Vanilla TypeScript, Custom CSS (kein Framework)
* **Styling**: CSS Custom Properties, automatisches System-Theme (Light/Dark)
* **EXIF-Lesen**: `kamadak-exif` Crate (Pure Rust, unterstützt JPEG/TIFF/HEIF)
* **Video-Datum**: Manuelles MP4/MOV Atom-Parsing (mvhd Creation-Time)
* **EXIF-Schreiben**: `exiftool` (externes Systemtool, optional)
* **HEIC-Konvertierung**: `libheif-rs` Crate (bindet libheif ein, kein externes Tool nötig)
* **Datei-Zeitstempel**: `filetime` Crate
* **Datums-Arithmetik**: `chrono` Crate
* **Verzeichnis-Traversierung**: `walkdir` Crate
* **Dateidialog**: `tauri-plugin-dialog`

### Systemvoraussetzungen (Linux)
* Tauri v2 Runtime-Abhängigkeiten (GTK, WebKit2GTK)
* Build-Abhängigkeit: `libheif-dev ≥ 1.17` (für HEIC→JPG, wird zur Compile-Zeit gelinkt)
* Optional: `sudo pacman -S perl-image-exiftool` (für EXIF-Schreiben bei Offset)

### Modulstruktur (Rust Backend)
* `models.rs` — Datentypen (FileEntry, RenameFormat, PreviewEntry, etc.)
* `exif_handler.rs` — EXIF-Lesen (Bilder), Video-Datum (MP4/MOV), EXIF-Schreiben
* `heic_converter.rs` — HEIC→JPG via libheif-rs
* `renamer.rs` — Namensformate, Duplikat-Handling, Offset-Berechnung
* `backup.rs` — Backup-Verzeichnis erstellen, Dateien kopieren
* `undo.rs` — Undo-Log verwalten, letzte Operation rückgängig machen
* `commands.rs` — Tauri Commands (scan_files, preview_rename, execute_rename, undo_last_rename, has_undo, check_tools)
* `lib.rs` — Tauri App-Setup, Plugin- und Command-Registrierung

### Frontend-Struktur
* `index.html` — Kompaktes Single-Page Layout
* `src/styles.css` — System-Theme, kompakte Toolbar, Dateiliste, Overlays
* `src/types.ts` — TypeScript-Interfaces (gespiegelt zu Rust-Modellen)
* `src/main.ts` — App-Initialisierung, Event-Handling, UI-Rendering, Tauri-Invoke-Aufrufe

## 9. Milestones & sequencing

### Phase 1 + 2: Implementiert ✅
* Tauri v2 Setup, Vanilla TypeScript Frontend
* Kompakte Toolbar-UI mit allen Controls in einer Zeile
* EXIF-Parsing (Bilder + Videos), 3 Namensformate
* Zeit-Offset (Sekunden + erweiterte Felder), Live-Preview
* Backup-Funktion (pro Quellordner)
* HEIC→JPG Konvertierung (90% Qualität)
* Fortschrittsbalken, Undo-Funktion, Fehlerprotokoll
* System-Theme (Light/Dark)

### Phase 3: Erweiterungen (Backlog)
* Benutzerdefinierte Namensformate
* Before/After-Preview
* Ordner-Organisation
* Duplikat-Erkennung
* Windows/macOS-Unterstützung

**Team**: 1 Fullstack-Developer

**Dependencies**: kamadak-exif, libheif-rs, image (Rust Crates), Tauri v2 Framework, Optional: exiftool (Systemtool)

## 10. Versioning strategy

Die Produktversionierung folgt Semantic Versioning (`MAJOR.MINOR.PATCH`):

* **MAJOR**: nicht abwärtskompatible (breaking) Änderungen
* **MINOR**: neue, abwärtskompatible Features
* **PATCH**: Bugfixes, Security-Härtung, kleinere Verbesserungen

Release- und Bump-Regeln sind bewusst als Entwicklungsrichtlinie ausgelagert und im Projektdokument `docs/versioning.md` beschrieben.
