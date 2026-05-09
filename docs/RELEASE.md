# Programm auf GitHub veröffentlichen

## 1. Repository sichtbar machen (falls privat)

- Auf GitHub: **Settings** → **General** → **Danger Zone** → **Change repository visibility** → **Make public**.

Damit ist der Quellcode und die **Actions**-Seite für alle erreichbar. Nutzer können unter **Actions** die neueste erfolgreiche Ausführung wählen und die **Artifacts** herunterladen – siehe README **Quick start / Installation → CI artifacts**.

---

## 2. Version festlegen und Tag setzen

Vor dem ersten Release die Version in allen Stellen auf den gewünschten Stand bringen (z.B. `1.0.0` oder `1.1.0`):

- `package.json` → `version`
- `src-tauri/Cargo.toml` → `version`
- `src-tauri/tauri.conf.json` → `version`

Optional: `npm run version:patch` / `version:minor` / `version:major` nutzen (siehe `docs/versioning.md`).

Dann Tag erstellen und pushen:

```bash
git tag v1.0.0
git push origin v1.0.0
```

---

## 3. GitHub Release erstellen (empfohlen)

**Manuell:**

1. Auf GitHub: **Releases** → **Create a new release**.
2. **Choose a tag:** den gerade gepushten Tag wählen (z.B. `v1.0.0`).
3. **Release title:** z.B. `v1.0.0` oder „Media File Renamer 1.0.0“.
4. **Describe:** Kurzbeschreibung oder Changelog (z.B. aus `CHANGELOG.md`).
5. Nach dem nächsten erfolgreichen **Build** (auf `main`): Bei der letzten Run unter **Actions** die **Artifacts** herunterladen.
6. Im Release-Formular bei **Attach binaries** die entpackten Dateien (z.B. `.exe`, `.msi`, `.deb`, `.AppImage`) hochladen.
7. **Publish release** klicken.

**Automatisch (dieses Repository):**  
Der Workflow [`.github/workflows/build.yml`](../.github/workflows/build.yml) baut bei Push auf `main` und bei Tags `v*`. Für Tags erzeugt der Job **release** mit `softprops/action-gh-release` ein GitHub Release und hängt die fertigen Windows- und Linux-Artefakte an (EXE, MSI, NSIS, Linux-Binary, AppImage, `.deb`). GitHub ergänzt automatisch die Quellcode-Zips (`Source code`).

---

## 4. Abgleich mit der GitHub-Projektseiten-PRD (`prd_github_project.md`)

Die Projekt-Doku im Repo-Wurzelverzeichnis (`README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, Ordner `screenshots/`) ist auf diese Empfehlungen ausgerichtet. Optional auf GitHub selbst noch ergänzen:

| Empfehlung | Hinweis |
|------------|---------|
| **Social Preview** (1200×630 px) | Unter *Settings* → *General* → *Social preview* hochladen. |
| **Repository-Logo / Avatar** | Organisations- oder Repo-Branding in GitHub UI. |
| **Topics** | Z. B. `tauri`, `rust`, `typescript`, `desktop`, `exif`, `batch-rename`, `photos`, `media`, `linux`, `windows`. |
| **`SHA256SUMS.txt`** | Aktuell nicht durch CI erzeugt; bei Bedarf nach Build manuell oder per Script aus den Release-Dateien erzeugen und am Release anhängen. |
| **`updater.json`** | Nur relevant, wenn der [Tauri Updater](https://v2.tauri.app/plugin/updater/) aktiv genutzt wird; dann separat bereitstellen/hosten. |

---

## Kurz-Checkliste

- [ ] Repo auf **public** gestellt (wenn gewünscht).
- [ ] Version per `npm run version:patch|minor|major` angepasst (`package.json`, `Cargo.toml`, `tauri.conf.json`, `package-lock.json`).
- [ ] `CHANGELOG.md` für die Version ergänzt.
- [ ] Änderungen committet und nach `main` gepusht.
- [ ] Tag gesetzt und gepusht: `git tag vX.Y.Z && git push origin vX.Y.Z`.
- [ ] Workflow **Build** auf dem Tag erfolgreich; Release mit Binaries ist automatisch angelegt (siehe **Releases**).
- [ ] Optional: Screenshots unter `screenshots/` ergänzen und im README verlinken.

Danach ist das Programm „veröffentlicht“: Quellcode und Builds sind einsehbar, Nutzer können über **Releases** oder **Actions** → Artifacts die fertigen Dateien herunterladen.
