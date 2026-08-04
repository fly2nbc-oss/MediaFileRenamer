use std::fs;
use std::path::{Component, Path, PathBuf};

use tauri::Manager;

use crate::models::UndoLog;

fn undo_log_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Cannot resolve app data dir: {}", e))?;
    fs::create_dir_all(&data_dir).map_err(|e| format!("Cannot create app data dir: {}", e))?;
    Ok(data_dir.join("undo_log.json"))
}

pub fn save_undo_log(app: &tauri::AppHandle, log: &UndoLog) -> Result<(), String> {
    let path = undo_log_path(app)?;
    let json = serde_json::to_string_pretty(log)
        .map_err(|e| format!("Failed to serialize undo log: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write undo log: {}", e))
}

pub fn load_undo_log(app: &tauri::AppHandle) -> Result<Option<UndoLog>, String> {
    let path = undo_log_path(app)?;
    if !path.exists() {
        return Ok(None);
    }
    let json = fs::read_to_string(&path).map_err(|e| format!("Failed to read undo log: {}", e))?;
    let log: UndoLog =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse undo log: {}", e))?;
    Ok(Some(log))
}

pub fn clear_undo_log(app: &tauri::AppHandle) -> Result<(), String> {
    let path = undo_log_path(app)?;
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Failed to clear undo log: {}", e))?;
    }
    Ok(())
}

pub fn execute_undo(app: &tauri::AppHandle) -> Result<String, String> {
    let log = load_undo_log(app)?.ok_or_else(|| "No rename operation to undo".to_string())?;

    let mut success = 0;
    let mut errors = Vec::new();

    for entry in log.entries.iter().rev() {
        let new_path = Path::new(&entry.new_path);
        let original_path = Path::new(&entry.original_path);

        if !is_safe_undo_pair(original_path, new_path) {
            errors.push(format!(
                "Unsafe paths in undo log entry for {}",
                entry.original_path
            ));
            continue;
        }

        if entry.was_heic_conversion {
            match &entry.backup_original_path {
                Some(backup_path) => {
                    let backup_path = Path::new(backup_path);
                    if !is_safe_backup_path(original_path, backup_path) {
                        errors.push(format!(
                            "Unsafe backup path in undo log entry for {}",
                            entry.original_path
                        ));
                        continue;
                    }
                    if !backup_path.exists() {
                        errors.push(format!("Backup copy not found for {}", entry.original_path));
                        continue;
                    }

                    if new_path.exists() {
                        if let Err(e) = fs::remove_file(new_path) {
                            errors.push(format!(
                                "Failed to remove converted file {}: {}",
                                new_path.display(),
                                e
                            ));
                            continue;
                        }
                    }

                    if let Err(e) = fs::copy(backup_path, original_path) {
                        errors.push(format!(
                            "Failed to restore original from backup {}: {}",
                            original_path.display(),
                            e
                        ));
                    } else {
                        success += 1;
                    }
                }
                None => {
                    if !new_path.exists() {
                        errors.push(format!("File not found: {}", entry.new_path));
                        continue;
                    }

                    let fallback_jpg_path = original_path.with_extension("jpg");
                    if let Err(e) = fs::rename(new_path, &fallback_jpg_path) {
                        errors.push(format!(
                            "Failed to partially restore converted file {}: {}",
                            new_path.display(),
                            e
                        ));
                    } else {
                        success += 1;
                    }
                }
            }
        } else if !new_path.exists() {
            errors.push(format!("File not found: {}", entry.new_path));
            continue;
        } else if let Err(e) = fs::rename(new_path, original_path) {
            errors.push(format!("Failed to restore {}: {}", new_path.display(), e));
        } else {
            success += 1;
        }
    }

    clear_undo_log(app)?;

    if errors.is_empty() {
        Ok(format!("Successfully restored {} files", success))
    } else {
        Ok(format!(
            "Restored {} files with {} errors: {}",
            success,
            errors.len(),
            errors.join("; ")
        ))
    }
}

fn is_safe_undo_pair(original_path: &Path, new_path: &Path) -> bool {
    is_normal_absolute_file_path(original_path)
        && is_normal_absolute_file_path(new_path)
        && original_path.parent() == new_path.parent()
}

fn is_safe_backup_path(original_path: &Path, backup_path: &Path) -> bool {
    if !is_normal_absolute_file_path(backup_path)
        || backup_path.file_name() != original_path.file_name()
    {
        return false;
    }

    let Some(backup_dir) = backup_path.parent() else {
        return false;
    };
    let Some(backup_dir_name) = backup_dir.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    backup_dir_name.starts_with("backup_") && backup_dir.parent() == original_path.parent()
}

fn is_normal_absolute_file_path(path: &Path) -> bool {
    path.is_absolute()
        && path.file_name().is_some()
        && path
            .components()
            .all(|component| !matches!(component, Component::CurDir | Component::ParentDir))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn accepts_rename_inside_the_same_directory() {
        let tmp = TempDir::new().unwrap();
        let original = tmp.path().join("before.heic");
        let renamed = tmp.path().join("after.jpg");

        assert!(is_safe_undo_pair(&original, &renamed));
    }

    #[test]
    fn rejects_parent_segments_and_cross_directory_renames() {
        let tmp = TempDir::new().unwrap();
        let original = tmp.path().join("before.jpg");
        let parent_segment = tmp.path().join("nested/../after.jpg");
        let other_directory = tmp.path().join("nested/after.jpg");

        assert!(!is_safe_undo_pair(&original, &parent_segment));
        assert!(!is_safe_undo_pair(&original, &other_directory));
    }

    #[test]
    fn accepts_only_the_matching_generated_backup_path() {
        let tmp = TempDir::new().unwrap();
        let original = tmp.path().join("photo.heic");
        let valid = tmp.path().join("backup_20260314_205542/photo.heic");
        let wrong_name = tmp.path().join("backup_20260314_205542/other.heic");
        let wrong_directory = tmp.path().join("archive/photo.heic");

        assert!(is_safe_backup_path(&original, &valid));
        assert!(!is_safe_backup_path(&original, &wrong_name));
        assert!(!is_safe_backup_path(&original, &wrong_directory));
    }
}
