//! Shared data types (APP-TAG-R2, R3).
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Editable fields, by the key the UI and the database use.
pub const FIELDS: &[&str] = &[
    "title", "artist", "album", "albumartist", "composer", "genre", "year", "track", "tracktotal", "disc", "disctotal", "comment",
];
pub const NUMERIC: &[&str] = &["year", "track", "tracktotal", "disc", "disctotal"];

/// Field values; a missing key means the field is empty.
pub type Fields = BTreeMap<String, String>;

/// Supported audio extensions (APP-TAG-R1). WMA is listed only (APP-TAG-R11).
pub const AUDIO_EXT: &[&str] = &["mp3", "flac", "m4a", "mp4", "aac", "ogg", "opus", "aif", "aiff", "wav", "ape", "wv", "dsf", "wma"];
pub const LIST_ONLY_EXT: &[&str] = &["wma"];

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrackTags {
    pub fields: Fields,
    pub has_art: bool,
    pub format: String,
    pub duration_ms: Option<u64>,
    /// WAV only: RIFF INFO disagrees with ID3 (APP-TAG-R10)
    pub riff_mismatch: bool,
    /// Album Artist stored under several keys with different values (APP-TAG-R13)
    pub aa_mismatch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackRow {
    pub path: String,
    pub source_id: i64,
    pub dir: String,
    pub file: String,
    pub ext: String,
    pub size: u64,
    pub mtime: i64,
    pub fields: Fields,
    pub has_art: bool,
    pub format: String,
    pub duration_ms: Option<u64>,
    pub riff_mismatch: bool,
    pub aa_mismatch: bool,
    /// "Lỗi đọc: …" (APP-SCAN-R6)
    pub error: Option<String>,
    /// listed but not editable (APP-TAG-R11)
    pub readonly: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub excludes: Vec<String>,
    pub status: String,
    pub last_scan: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StagedChange {
    pub path: String,
    pub field: String,
    pub orig: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageInput {
    pub path: String,
    pub field: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunInfo {
    pub id: i64,
    pub kind: String,
    pub label: Option<String>,
    pub started: i64,
    pub finished: Option<i64>,
    pub status: String,
    pub files_ok: i64,
    pub files_conflict: i64,
    pub files_error: i64,
    pub summary: String,
    pub undo_of: Option<i64>,
}
