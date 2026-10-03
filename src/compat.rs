// SPDX-License-Identifier: Apache-2.0

//! Checks that the installed COSMIC desktop uses the same theme config
//! version Stardust was built for. If they differ, applied themes would be
//! written to a folder the desktop never reads.

use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::cosmic_theme::ThemeBuilder;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const BUILDER_ID: &str = "com.system76.CosmicTheme.Dark.Builder";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compat {
    Match,
    DesktopNewer { desktop: u64, app: u64 },
    DesktopOlder { desktop: u64, app: u64 },
    /// No system theme defaults found, so there's nothing to compare against.
    Unknown,
}

pub fn check() -> Compat {
    compare(highest_version(&data_dirs()), ThemeBuilder::VERSION)
}

fn compare(desktop: Option<u64>, app: u64) -> Compat {
    match desktop {
        None => Compat::Unknown,
        Some(desktop) if desktop == app => Compat::Match,
        Some(desktop) if desktop > app => Compat::DesktopNewer { desktop, app },
        Some(desktop) => Compat::DesktopOlder { desktop, app },
    }
}

/// The COSMIC packages install theme defaults to
/// `<data dir>/cosmic/com.system76.CosmicTheme.Dark.Builder/v<N>/`.
/// The highest `N` is the version the installed desktop reads.
fn highest_version(data_dirs: &[PathBuf]) -> Option<u64> {
    data_dirs
        .iter()
        .filter_map(|dir| versions_in(&dir.join("cosmic").join(BUILDER_ID)))
        .max()
}

fn versions_in(dir: &Path) -> Option<u64> {
    fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            entry
                .file_name()
                .to_str()?
                .strip_prefix('v')?
                .parse::<u64>()
                .ok()
        })
        .max()
}

fn data_dirs() -> Vec<PathBuf> {
    let dirs = env::var("XDG_DATA_DIRS")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".to_string());
    env::split_paths(&dirs).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_versions(root: &Path, versions: &[u64]) {
        for v in versions {
            fs::create_dir_all(root.join("cosmic").join(BUILDER_ID).join(format!("v{v}"))).unwrap();
        }
    }

    #[test]
    fn finds_highest_version_across_dirs() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        make_versions(a.path(), &[1, 2]);
        make_versions(b.path(), &[3]);
        let dirs = [a.path().to_path_buf(), b.path().to_path_buf()];
        assert_eq!(highest_version(&dirs), Some(3));
    }

    #[test]
    fn ignores_non_version_entries() {
        let a = tempfile::tempdir().unwrap();
        make_versions(a.path(), &[2]);
        let builder_dir = a.path().join("cosmic").join(BUILDER_ID);
        fs::create_dir_all(builder_dir.join("vnext")).unwrap();
        fs::write(builder_dir.join("v9"), "a file, not a folder").unwrap();
        assert_eq!(highest_version(&[a.path().to_path_buf()]), Some(2));
    }

    #[test]
    fn missing_dirs_are_unknown() {
        let a = tempfile::tempdir().unwrap();
        assert_eq!(highest_version(&[a.path().to_path_buf()]), None);
        assert_eq!(compare(None, 2), Compat::Unknown);
    }

    #[test]
    fn compares_versions() {
        assert_eq!(compare(Some(2), 2), Compat::Match);
        assert_eq!(compare(Some(3), 2), Compat::DesktopNewer { desktop: 3, app: 2 });
        assert_eq!(compare(Some(1), 2), Compat::DesktopOlder { desktop: 1, app: 2 });
    }
}
