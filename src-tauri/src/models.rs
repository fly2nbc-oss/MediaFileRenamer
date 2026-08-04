use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RenameFormat {
    #[serde(rename = "YYYY_MM_DD__hhmmss")]
    Format1,
    #[serde(rename = "YYMMDD_hhmmss")]
    Format2,
    #[serde(rename = "YYMMDD_original")]
    Format3,
    #[serde(rename = "NO_RENAME")]
    NoRename,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DateSource {
    #[serde(rename = "exif")]
    Exif,
    #[serde(rename = "filesystem")]
    FileSystem,
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub filename: String,
    pub extension: String,
    pub date_source: DateSource,
    pub datetime: Option<String>,
    pub is_heic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewEntry {
    pub original_path: String,
    pub original_name: String,
    pub new_name: String,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameResult {
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<RenameErrorEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameErrorEntry {
    pub filename: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoEntry {
    pub original_path: String,
    pub new_path: String,
    #[serde(default)]
    pub was_heic_conversion: bool,
    #[serde(default)]
    pub backup_original_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoLog {
    pub entries: Vec<UndoEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressPayload {
    pub current: usize,
    pub total: usize,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsStatus {
    pub exiftool: bool,
    pub heif_convert: bool,
}

/// Maximum number of files accepted by a single scan request.
pub const MAX_SCAN_FILES: usize = 10_000;

/// JPEG quality (0-100) for HEIC-to-JPG conversion.
pub const HEIC_JPEG_QUALITY: u8 = 90;

/// Maximum numeric suffix tried before a conflicting rename is rejected.
pub const MAX_COLLISION_SUFFIX: u32 = 999;

pub const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "heic", "heif", "tiff", "tif", "webp", "gif", "bmp",
];
pub const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mov", "avi", "mkv"];
pub const HEIC_EXTENSIONS: &[&str] = &["heic", "heif"];

pub fn is_supported_extension(ext: &str) -> bool {
    let lower = ext.to_lowercase();
    IMAGE_EXTENSIONS.contains(&lower.as_str()) || VIDEO_EXTENSIONS.contains(&lower.as_str())
}

pub fn is_video_extension(ext: &str) -> bool {
    VIDEO_EXTENSIONS.contains(&ext.to_lowercase().as_str())
}

pub fn is_heic_extension(ext: &str) -> bool {
    HEIC_EXTENSIONS.contains(&ext.to_lowercase().as_str())
}
