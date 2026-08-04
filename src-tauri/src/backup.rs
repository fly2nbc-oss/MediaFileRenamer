use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::models::FileEntry;

pub fn create_backup(entries: &[FileEntry]) -> Result<HashMap<String, String>, String> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();

    let mut dirs: HashMap<PathBuf, Vec<&FileEntry>> = HashMap::new();
    for entry in entries {
        let path = Path::new(&entry.path);
        let parent = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        dirs.entry(parent).or_default().push(entry);
    }

    let mut backup_map: HashMap<String, String> = HashMap::new();

    for (dir, files) in &dirs {
        let backup_dir = dir.join(format!("backup_{}", timestamp));
        std::fs::create_dir_all(&backup_dir)
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;

        for file_entry in files {
            let src = Path::new(&file_entry.path);
            let safe_name = src
                .file_name()
                .ok_or_else(|| format!("Failed to resolve filename for {}", file_entry.path))?;
            let dst = backup_dir.join(safe_name);
            std::fs::copy(src, &dst)
                .map_err(|e| format!("Failed to backup {}: {}", file_entry.filename, e))?;
            backup_map.insert(file_entry.path.clone(), dst.to_string_lossy().to_string());
        }
    }

    Ok(backup_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    use crate::models::DateSource;
    use tempfile::TempDir;

    #[test]
    fn backup_uses_the_source_basename() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("photo.jpg");
        std::fs::write(&source, b"photo").unwrap();

        let entry = FileEntry {
            path: source.to_string_lossy().to_string(),
            filename: "../escaped.jpg".to_string(),
            extension: "jpg".to_string(),
            date_source: DateSource::FileSystem,
            datetime: None,
            is_heic: false,
        };

        let backups = create_backup(std::slice::from_ref(&entry)).unwrap();
        let backup = PathBuf::from(backups.get(&entry.path).unwrap());

        assert_eq!(backup.file_name(), Some(OsStr::new("photo.jpg")));
        assert_eq!(backup.parent().and_then(Path::parent), Some(tmp.path()));
        assert!(!tmp.path().join("escaped.jpg").exists());
    }
}
