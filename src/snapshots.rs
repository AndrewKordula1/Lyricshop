//! Snapshots: timestamped copies of the whole song.
//! Port of `Snapshots.kt`. They live in a `snapshots` folder next to the
//! song file, so they travel with the song. Snapshots need a saved file
//! first — without one there is nowhere to put them.

use crate::serialization::{decode_song, encode_song};
use crate::song::Song;
use std::path::{Path, PathBuf};

/// The snapshots folder next to the current song file, if any.
pub fn snapshot_dir(song_file: Option<&Path>) -> Option<PathBuf> {
    song_file
        .and_then(|f| f.parent().map(|p| p.to_path_buf()))
        .map(|p| p.join("snapshots"))
}

pub fn save_snapshot(song: &Song, song_file: Option<&Path>) -> bool {
    let Some(dir) = snapshot_dir(song_file) else {
        return false;
    };
    if std::fs::create_dir_all(&dir).is_err() {
        return false;
    }
    let stamp = chrono_stamp();
    std::fs::write(dir.join(format!("{}.json", stamp)), encode_song(song)).is_ok()
}

fn chrono_stamp() -> String {
    // "2026-11-08_11-30-00" from local time, without a time crate.
    let now = std::time::SystemTime::now();
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Local-time offset is not available without a crate; use UTC and note
    // it. The filename is only a label — display_name() parses it back.
    let (y, mo, d, h, mi, s) = utc_parts(secs);
    format!("{:04}-{:02}-{:02}_{:02}-{:02}-{:02}", y, mo, d, h, mi, s)
}

fn utc_parts(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let days = secs / 86400;
    let rem = secs % 86400;
    let h = rem / 3600;
    let mi = (rem % 3600) / 60;
    let s = rem % 60;
    // Civil date from days since epoch (Howard Hinnant's algorithm).
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    (y as u64, mo, d, h, mi, s)
}

pub fn list_snapshots(song_file: Option<&Path>) -> Vec<PathBuf> {
    let Some(dir) = snapshot_dir(song_file) else {
        return Vec::new();
    };
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_file() && p.extension().map(|e| e == "json").unwrap_or(false))
                .collect()
        })
        .unwrap_or_default();
    // Newest first (by filename timestamp, which sorts the same way).
    files.sort_by(|a, b| b.cmp(a));
    files
}

pub fn load_snapshot(song: &mut Song, file: &Path) -> bool {
    match std::fs::read_to_string(file) {
        Ok(text) => match decode_song(&text) {
            Some(loaded) => {
                song.copy_from(&loaded);
                true
            }
            None => false,
        },
        Err(_) => false,
    }
}

pub fn delete_snapshot(file: &Path) -> bool {
    std::fs::remove_file(file).is_ok()
}

/// "2026-11-08_11-30-00.json" -> "Nov 8, 2026 11:30 AM" (UTC).
pub fn display_name(file: &Path) -> String {
    let base = file
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let parts: Vec<&str> = base.split('_').collect();
    if parts.len() == 2 {
        let date: Vec<&str> = parts[0].split('-').collect();
        let time: Vec<&str> = parts[1].split('-').collect();
        if date.len() == 3 && time.len() == 3 {
            if let (Ok(y), Ok(mo), Ok(d), Ok(h), Ok(mi), Ok(_s)) = (
                date[0].parse::<u64>(),
                date[1].parse::<u64>(),
                date[2].parse::<u64>(),
                time[0].parse::<u64>(),
                time[1].parse::<u64>(),
                time[2].parse::<u64>(),
            ) {
                let months = [
                    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov",
                    "Dec",
                ];
                let month = months.get((mo as usize).saturating_sub(1)).unwrap_or(&"?");
                let (h12, ampm) = if h == 0 {
                    (12, "AM")
                } else if h < 12 {
                    (h, "AM")
                } else if h == 12 {
                    (12, "PM")
                } else {
                    (h - 12, "PM")
                };
                return format!("{} {}, {} {}:{:02} {}", month, d, y, h12, mi, ampm);
            }
        }
    }
    base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_name_formats() {
        let p = Path::new("2026-11-08_11-30-00.json");
        assert_eq!(display_name(p), "Nov 8, 2026 11:30 AM");
        let p = Path::new("2026-01-02_00-05-00.json");
        assert_eq!(display_name(p), "Jan 2, 2026 12:05 AM");
    }

    #[test]
    fn snapshot_round_trip() {
        let dir = std::env::temp_dir().join(format!(
            "lyricshop-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let song_file = dir.join("song.json");
        let mut song = Song::new("Snap Test", "Rock / Metal");
        song.sections = crate::song::generate_sections("Rock / Metal");
        song.sections[1].lyrics = "original line".to_string();
        assert!(save_snapshot(&song, Some(&song_file)));
        let snaps = list_snapshots(Some(&song_file));
        assert_eq!(snaps.len(), 1);
        song.sections[1].lyrics = "ruined line".to_string();
        assert!(load_snapshot(&mut song, &snaps[0]));
        assert_eq!(song.sections[1].lyrics, "original line");
        assert!(delete_snapshot(&snaps[0]));
        assert!(list_snapshots(Some(&song_file)).is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn no_file_means_no_snapshots() {
        assert!(snapshot_dir(None).is_none());
        assert!(list_snapshots(None).is_empty());
        let song = Song::new("x", "Rock / Metal");
        assert!(!save_snapshot(&song, None));
    }
}
