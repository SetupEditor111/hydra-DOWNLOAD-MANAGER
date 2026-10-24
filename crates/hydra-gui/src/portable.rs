// Copyright (C) 2026 Javad Rajabzadeh
// SPDX-License-Identifier: GPL-3.0-or-later

//! `Portable.ini` beside the executable: portable mode without a flag.
//!
//! The file's presence is the switch, so every way of starting the app (a
//! double click, the browser extension's auto-launch, a launcher) lands on
//! the same profile. Its parsing is mirrored by `hydra-host`, which has to
//! reach the same directory from the same file.

use std::path::{Path, PathBuf};

pub const MARKER: &str = "Portable.ini";
const DEFAULT_DATA_DIR: &str = "data";

/// The profile directory `Portable.ini` in `exe_dir` asks for, `None` when
/// the file is absent. `data_dir = PATH` overrides the default `data` folder;
/// a relative path is taken from `exe_dir`, so the folder can move as a unit.
pub fn data_dir(exe_dir: &Path) -> Option<PathBuf> {
    let text = std::fs::read_to_string(exe_dir.join(MARKER)).ok()?;
    let dir = parse_data_dir(&text).unwrap_or_else(|| PathBuf::from(DEFAULT_DATA_DIR));
    Some(if dir.is_absolute() {
        dir
    } else {
        exe_dir.join(dir)
    })
}

fn parse_data_dir(text: &str) -> Option<PathBuf> {
    text.trim_start_matches('\u{feff}')
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once('=')?;
            let value = value.trim().trim_matches('"');
            (key.trim().eq_ignore_ascii_case("data_dir") && !value.is_empty())
                .then(|| PathBuf::from(value))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exe_dir_with(ini: Option<&str>) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        if let Some(body) = ini {
            std::fs::write(dir.path().join(MARKER), body).unwrap();
        }
        dir
    }

    #[test]
    fn no_marker_means_an_ordinary_install() {
        let dir = exe_dir_with(None);
        assert_eq!(data_dir(dir.path()), None);
    }

    #[test]
    fn an_empty_marker_uses_the_data_folder_beside_the_exe() {
        let dir = exe_dir_with(Some(""));
        assert_eq!(data_dir(dir.path()), Some(dir.path().join("data")));
    }

    #[test]
    fn a_relative_data_dir_is_taken_from_the_exe_folder() {
        let dir = exe_dir_with(Some("[Portable]\ndata_dir = ..\\profile\n"));
        assert_eq!(data_dir(dir.path()), Some(dir.path().join("..\\profile")));
    }

    #[test]
    fn an_absolute_data_dir_is_kept_as_written() {
        let abs = std::env::temp_dir().join("hydra-profile-x");
        let dir = exe_dir_with(Some(&format!("data_dir = \"{}\"\n", abs.display())));
        assert_eq!(data_dir(dir.path()), Some(abs));
    }

    #[test]
    fn key_matching_ignores_case_and_a_byte_order_mark() {
        assert_eq!(
            parse_data_dir("\u{feff}[x]\nDATA_DIR=cfg"),
            Some(PathBuf::from("cfg"))
        );
    }

    #[test]
    fn a_blank_value_or_other_keys_fall_back_to_the_default() {
        assert_eq!(parse_data_dir("data_dir =\nother = 1"), None);
        assert_eq!(parse_data_dir("; data_dir is not set"), None);
    }
}
