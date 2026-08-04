use chrono::NaiveDateTime;
use std::collections::HashMap;
use std::path::Path;

use crate::models::{DateSource, FileEntry, PreviewEntry, RenameFormat};

pub fn generate_previews(
    entries: &[FileEntry],
    format: &RenameFormat,
    offset_seconds: i64,
    convert_heic: bool,
) -> Vec<PreviewEntry> {
    let mut base_names: Vec<(String, String, Option<String>)> = Vec::with_capacity(entries.len());

    for entry in entries {
        let datetime = entry
            .datetime
            .as_ref()
            .and_then(|s| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok());

        let adjusted = datetime.map(|dt| dt + chrono::Duration::seconds(offset_seconds));

        let ext = if entry.is_heic && convert_heic {
            "jpg".to_string()
        } else {
            entry.extension.clone()
        };

        let warning = if entry.date_source == DateSource::None {
            Some("No date found – file will not be renamed".to_string())
        } else {
            None
        };

        let stem = match adjusted {
            Some(dt) => format_stem(&dt, &entry.filename, format),
            None => {
                let p = Path::new(&entry.filename);
                p.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| entry.filename.clone())
            }
        };

        base_names.push((stem, ext, warning));
    }

    // Count how many times each full name appears
    let mut full_name_counts: HashMap<String, usize> = HashMap::new();
    for (stem, ext, _) in &base_names {
        let full = format!("{}.{}", stem, ext);
        *full_name_counts.entry(full).or_insert(0) += 1;
    }

    // Assign final names, adding _1, _2 etc. for duplicates
    let mut occurrence_index: HashMap<String, usize> = HashMap::new();
    let mut previews = Vec::with_capacity(entries.len());

    for (i, (stem, ext, warning)) in base_names.iter().enumerate() {
        let base_full = format!("{}.{}", stem, ext);
        let total = full_name_counts[&base_full];

        let new_name = if total > 1 {
            let idx = occurrence_index.entry(base_full.clone()).or_insert(0);
            let name = if *idx == 0 {
                base_full.clone()
            } else {
                format!("{}_{}.{}", stem, idx, ext)
            };
            *idx += 1;
            name
        } else {
            base_full
        };

        previews.push(PreviewEntry {
            original_path: entries[i].path.clone(),
            original_name: entries[i].filename.clone(),
            new_name,
            warning: warning.clone(),
        });
    }

    previews
}

fn format_stem(datetime: &NaiveDateTime, original_name: &str, format: &RenameFormat) -> String {
    match format {
        RenameFormat::Format1 => datetime.format("%Y_%m_%d__%H%M%S").to_string(),
        RenameFormat::Format2 => datetime.format("%y%m%d_%H%M%S").to_string(),
        RenameFormat::Format3 => {
            let stem = Path::new(original_name)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            format!("{}_{}", datetime.format("%y%m%d"), stem)
        }
        RenameFormat::NoRename => Path::new(original_name)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default(),
    }
}

pub fn apply_offset(datetime: &NaiveDateTime, offset_seconds: i64) -> NaiveDateTime {
    *datetime + chrono::Duration::seconds(offset_seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_datetime() -> NaiveDateTime {
        NaiveDateTime::parse_from_str("2024-03-15 14:30:52", "%Y-%m-%d %H:%M:%S").unwrap()
    }

    #[test]
    fn formats_all_supported_filename_variants() {
        let datetime = sample_datetime();

        assert_eq!(
            format_stem(&datetime, "IMG_1234.jpg", &RenameFormat::Format1),
            "2024_03_15__143052"
        );
        assert_eq!(
            format_stem(&datetime, "IMG_1234.jpg", &RenameFormat::Format2),
            "240315_143052"
        );
        assert_eq!(
            format_stem(&datetime, "IMG_1234.jpg", &RenameFormat::Format3),
            "240315_IMG_1234"
        );
        assert_eq!(
            format_stem(&datetime, "IMG_1234.jpg", &RenameFormat::NoRename),
            "IMG_1234"
        );
    }

    #[test]
    fn applies_positive_and_negative_offsets() {
        let datetime = sample_datetime();

        assert_eq!(
            apply_offset(&datetime, 3600)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            "2024-03-15 15:30:52"
        );
        assert_eq!(
            apply_offset(&datetime, -7200)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            "2024-03-15 12:30:52"
        );
    }
}
