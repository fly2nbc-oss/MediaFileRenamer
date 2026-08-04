use std::collections::HashMap;
use std::path::Path;

use chrono::NaiveDateTime;
use tauri::Emitter;
use walkdir::WalkDir;

use crate::backup;
use crate::exif_handler;
use crate::heic_converter;
use crate::models::*;
use crate::renamer;
use crate::undo;

#[tauri::command]
pub fn check_tools() -> ToolsStatus {
    ToolsStatus {
        exiftool: exif_handler::is_exiftool_available(),
        heif_convert: heic_converter::is_heif_convert_available(),
    }
}

#[tauri::command]
pub fn scan_files(paths: Vec<String>) -> Result<Vec<FileEntry>, String> {
    scan_files_with_limit(&paths, MAX_SCAN_FILES)
}

fn scan_files_with_limit(paths: &[String], max_files: usize) -> Result<Vec<FileEntry>, String> {
    let mut entries = Vec::new();

    for path_str in paths {
        let path = Path::new(path_str);

        if path.is_dir() {
            for dir_entry in WalkDir::new(path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if dir_entry.file_type().is_file() {
                    if let Some(entry) = scan_single_file(dir_entry.path()) {
                        ensure_scan_capacity(entries.len(), max_files)?;
                        entries.push(entry);
                    }
                }
            }
        } else if path.is_file() {
            if let Some(entry) = scan_single_file(path) {
                ensure_scan_capacity(entries.len(), max_files)?;
                entries.push(entry);
            }
        }
    }

    Ok(entries)
}

fn ensure_scan_capacity(current_count: usize, max_files: usize) -> Result<(), String> {
    if current_count >= max_files {
        return Err(format!(
            "Scan exceeds the safety limit of {} supported files. Select fewer files or a smaller directory.",
            max_files
        ));
    }
    Ok(())
}

fn scan_single_file(path: &Path) -> Option<FileEntry> {
    let ext = path.extension()?.to_str()?.to_lowercase();

    if !is_supported_extension(&ext) {
        return None;
    }

    let filename = path.file_name()?.to_string_lossy().to_string();
    let (datetime, date_source) = exif_handler::read_date_from_file(path);

    Some(FileEntry {
        path: path.to_string_lossy().to_string(),
        filename,
        extension: ext.clone(),
        date_source,
        datetime: datetime.map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string()),
        is_heic: is_heic_extension(&ext),
    })
}

#[tauri::command]
pub fn preview_rename(
    entries: Vec<FileEntry>,
    format: RenameFormat,
    offset_seconds: i64,
    convert_heic: bool,
) -> Result<Vec<PreviewEntry>, String> {
    Ok(renamer::generate_previews(
        &entries,
        &format,
        offset_seconds,
        convert_heic,
    ))
}

#[tauri::command]
pub async fn execute_rename(
    app: tauri::AppHandle,
    entries: Vec<FileEntry>,
    format: RenameFormat,
    offset_seconds: i64,
    create_backup: bool,
    convert_heic: bool,
) -> Result<RenameResult, String> {
    let total = entries.len();
    let mut success_count = 0;
    let mut errors: Vec<RenameErrorEntry> = Vec::new();
    let mut undo_entries: Vec<UndoEntry> = Vec::new();

    let previews = renamer::generate_previews(&entries, &format, offset_seconds, convert_heic);

    let backup_map: HashMap<String, String> = if create_backup {
        backup::create_backup(&entries)?
    } else {
        HashMap::new()
    };

    let has_exiftool = exif_handler::is_exiftool_available();

    for (i, (entry, preview)) in entries.iter().zip(previews.iter()).enumerate() {
        let _ = app.emit(
            "rename-progress",
            ProgressPayload {
                current: i + 1,
                total,
                filename: entry.filename.clone(),
            },
        );

        let original_path = Path::new(&entry.path);
        let parent = original_path.parent().unwrap_or(Path::new("."));
        let new_path = parent.join(&preview.new_name);

        if entry.date_source == DateSource::None {
            errors.push(RenameErrorEntry {
                filename: entry.filename.clone(),
                reason: "No date available, skipped".to_string(),
            });
            continue;
        }

        if original_path == new_path {
            apply_timestamp_offset(
                original_path,
                entry.datetime.as_deref(),
                offset_seconds,
                has_exiftool,
            );
            success_count += 1;
            continue;
        }

        // Ensure target doesn't already exist on disk (outside our batch)
        let final_path = match resolve_conflict(&new_path) {
            Ok(path) => path,
            Err(reason) => {
                errors.push(RenameErrorEntry {
                    filename: entry.filename.clone(),
                    reason,
                });
                continue;
            }
        };

        let mut actual_final_path = final_path.clone();
        let mut was_heic_conversion = false;
        let mut rename_failed = false;

        if entry.is_heic && convert_heic {
            match heic_converter::convert_heic_to_jpg(original_path, &final_path, HEIC_JPEG_QUALITY)
            {
                Ok(()) => {
                    was_heic_conversion = true;
                    if let Err(e) = std::fs::remove_file(original_path) {
                        errors.push(RenameErrorEntry {
                            filename: entry.filename.clone(),
                            reason: format!(
                                "Conversion succeeded but failed to remove original: {}",
                                e
                            ),
                        });
                    }
                }
                Err(convert_err) => {
                    // Fallback: rename keeping original extension
                    let fallback =
                        match resolve_conflict(&final_path.with_extension(&entry.extension)) {
                            Ok(path) => path,
                            Err(conflict_err) => {
                                errors.push(RenameErrorEntry {
                                    filename: entry.filename.clone(),
                                    reason: format!(
                                        "HEIC->JPG failed: {}; {}",
                                        convert_err, conflict_err
                                    ),
                                });
                                continue;
                            }
                        };
                    match std::fs::rename(original_path, &fallback) {
                        Ok(()) => {
                            actual_final_path = fallback;
                            errors.push(RenameErrorEntry {
                                filename: entry.filename.clone(),
                                reason: format!(
                                    "HEIC→JPG failed: {}, renamed without conversion",
                                    convert_err
                                ),
                            });
                        }
                        Err(e) => {
                            errors.push(RenameErrorEntry {
                                filename: entry.filename.clone(),
                                reason: format!("{}, rename also failed: {}", convert_err, e),
                            });
                            rename_failed = true;
                        }
                    }
                }
            }
        } else {
            match std::fs::rename(original_path, &final_path) {
                Ok(()) => {}
                Err(e) => {
                    errors.push(RenameErrorEntry {
                        filename: entry.filename.clone(),
                        reason: e.to_string(),
                    });
                    rename_failed = true;
                }
            }
        }

        if rename_failed {
            continue;
        }

        apply_timestamp_offset(
            &actual_final_path,
            entry.datetime.as_deref(),
            offset_seconds,
            has_exiftool,
        );

        undo_entries.push(UndoEntry {
            original_path: entry.path.clone(),
            new_path: actual_final_path.to_string_lossy().to_string(),
            was_heic_conversion,
            backup_original_path: backup_map.get(&entry.path).cloned(),
        });
        success_count += 1;
    }

    let undo_log = UndoLog {
        entries: undo_entries,
    };
    let _ = undo::save_undo_log(&app, &undo_log);

    Ok(RenameResult {
        success_count,
        error_count: errors.len(),
        errors,
    })
}

fn apply_timestamp_offset(
    path: &Path,
    datetime: Option<&str>,
    offset_seconds: i64,
    has_exiftool: bool,
) {
    if offset_seconds == 0 {
        return;
    }

    let Some(datetime) = datetime else {
        return;
    };
    let Ok(datetime) = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%dT%H:%M:%S") else {
        return;
    };

    let adjusted = renamer::apply_offset(&datetime, offset_seconds);
    let file_time = filetime::FileTime::from_unix_time(adjusted.and_utc().timestamp(), 0);
    let _ = filetime::set_file_times(path, file_time, file_time);

    if has_exiftool {
        let _ = exif_handler::write_exif_dates(path, &adjusted);
    }
}

fn resolve_conflict(target: &Path) -> Result<std::path::PathBuf, String> {
    resolve_conflict_with_limit(target, MAX_COLLISION_SUFFIX)
}

fn resolve_conflict_with_limit(
    target: &Path,
    max_suffix: u32,
) -> Result<std::path::PathBuf, String> {
    if !target.exists() {
        return Ok(target.to_path_buf());
    }

    let stem = target
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = target.extension().and_then(|s| s.to_str()).unwrap_or("");
    let parent = target.parent().unwrap_or(Path::new("."));

    for i in 1..=max_suffix {
        let candidate = if ext.is_empty() {
            parent.join(format!("{}_{}", stem, i))
        } else {
            parent.join(format!("{}_{}.{}", stem, i, ext))
        };
        if !candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(format!(
        "Too many conflicting files (limit {}). Cannot find a unique filename.",
        max_suffix
    ))
}

#[tauri::command]
pub async fn undo_last_rename(app: tauri::AppHandle) -> Result<String, String> {
    undo::execute_undo(&app)
}

#[tauri::command]
pub fn has_undo(app: tauri::AppHandle) -> bool {
    undo::load_undo_log(&app)
        .ok()
        .flatten()
        .map(|log| !log.entries.is_empty())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::time::UNIX_EPOCH;

    use tempfile::TempDir;

    #[test]
    fn scan_rejects_requests_above_the_file_limit() {
        let tmp = TempDir::new().unwrap();
        File::create(tmp.path().join("one.jpg")).unwrap();
        File::create(tmp.path().join("two.jpg")).unwrap();
        let paths = vec![tmp.path().to_string_lossy().to_string()];

        let error = scan_files_with_limit(&paths, 1).unwrap_err();

        assert!(error.contains("safety limit of 1"));
    }

    #[cfg(unix)]
    #[test]
    fn directory_scan_does_not_follow_symlinks() {
        use std::os::unix::fs::symlink;

        let root = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        File::create(root.path().join("inside.jpg")).unwrap();
        File::create(outside.path().join("outside.jpg")).unwrap();
        symlink(outside.path(), root.path().join("linked-directory")).unwrap();
        let paths = vec![root.path().to_string_lossy().to_string()];

        let entries = scan_files_with_limit(&paths, 10).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].filename, "inside.jpg");
    }

    #[test]
    fn conflict_resolution_uses_a_free_suffix() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("photo.jpg");
        File::create(&target).unwrap();

        let resolved = resolve_conflict_with_limit(&target, 2).unwrap();

        assert_eq!(resolved, tmp.path().join("photo_1.jpg"));
    }

    #[test]
    fn conflict_resolution_never_returns_an_existing_target() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("photo.jpg");
        File::create(&target).unwrap();
        File::create(tmp.path().join("photo_1.jpg")).unwrap();

        let error = resolve_conflict_with_limit(&target, 1).unwrap_err();

        assert!(error.contains("limit 1"));
    }

    #[test]
    fn offset_can_update_a_file_without_renaming_it() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("photo.jpg");
        File::create(&path).unwrap();
        let datetime = "2024-03-15T14:30:52";

        apply_timestamp_offset(&path, Some(datetime), 3600, false);

        let modified = std::fs::metadata(path)
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let expected = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%dT%H:%M:%S")
            .unwrap()
            .and_utc()
            .timestamp()
            + 3600;
        assert_eq!(modified, expected);
    }
}
