// SPDX-License-Identifier: Apache-2.0

//! Installs icon themes from folders and archives into the user's icons folder.

use super::is_icon_theme;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// How deep to look for themes inside a picked folder or archive. Packs like
/// `Flat-Remix-master/Flat-Remix-Blue-Dark/` need two levels.
const SEARCH_DEPTH: usize = 3;

/// `~/.local/share/icons`, where COSMIC and other desktops look for user themes.
pub fn user_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("icons"))
}

#[derive(Clone, Debug, Default)]
pub struct InstallReport {
    /// Folder names of the themes that were installed.
    pub installed: Vec<String>,
    /// Themes that weren't installed, with the reason.
    pub failed: Vec<(String, String)>,
}

impl InstallReport {
    fn merge(&mut self, other: InstallReport) {
        self.installed.extend(other.installed);
        self.failed.extend(other.failed);
    }
}

/// Copies every icon theme in `source` (the folder itself, or themes inside
/// it) into `dest`. Symlinks are kept as symlinks, since themes use lots of them.
pub fn install_dir(source: &Path, dest: &Path) -> io::Result<InstallReport> {
    let themes = find_themes(source)?;
    fs::create_dir_all(dest)?;

    let mut report = InstallReport::default();
    for theme in themes {
        report.merge(place(&theme, dest, |from, to| {
            let result = copy_tree(from, to);
            if result.is_err() {
                // Don't leave half a theme behind.
                let _ = fs::remove_dir_all(to);
            }
            result
        }));
    }
    Ok(report)
}

/// Unpacks a `.tar.*` archive with the system `tar` and installs the themes
/// inside it into `dest`.
pub fn install_archive(archive: &Path, dest: &Path) -> io::Result<InstallReport> {
    fs::create_dir_all(dest)?;
    // Unpacking next to the final folder makes moving themes into place a rename.
    let staging = dest.join(format!(".stardust-import-{}", std::process::id()));
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir(&staging)?;

    let result = unpack(archive, &staging).and_then(|()| {
        let mut report = InstallReport::default();
        for theme in find_themes(&staging)? {
            report.merge(place(&theme, dest, |from, to| fs::rename(from, to)));
        }
        Ok(report)
    });

    let _ = fs::remove_dir_all(&staging);
    result
}

fn unpack(archive: &Path, into: &Path) -> io::Result<()> {
    // GNU tar picks the compression from the file itself and refuses
    // absolute paths and `..` in member names.
    let output = Command::new("tar")
        .arg("-xf")
        .arg(archive)
        .arg("-C")
        .arg(into)
        .arg("--no-same-owner")
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(io::Error::other(
            stderr.lines().next().unwrap_or("tar failed").to_string(),
        ))
    }
}

/// Moves or copies one theme into `dest` unless a theme with that folder
/// name is already installed there.
fn place(
    theme: &Path,
    dest: &Path,
    transfer: impl FnOnce(&Path, &Path) -> io::Result<()>,
) -> InstallReport {
    let mut report = InstallReport::default();
    let Some(id) = theme
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
    else {
        return report;
    };

    let target = dest.join(&id);
    if target.exists() {
        report.failed.push((id, "already installed".to_string()));
        return report;
    }
    match transfer(theme, &target) {
        Ok(()) => report.installed.push(id),
        Err(err) => report.failed.push((id, err.to_string())),
    }
    report
}

/// Returns `dir` if it's an icon theme, otherwise the icon themes inside it.
fn find_themes(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut themes = Vec::new();
    collect_themes(dir, SEARCH_DEPTH, &mut themes);
    if themes.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "no icon theme found",
        ));
    }
    themes.sort();
    Ok(themes)
}

fn collect_themes(dir: &Path, depth: usize, themes: &mut Vec<PathBuf>) {
    if is_icon_theme(dir) {
        themes.push(dir.to_path_buf());
        return;
    }
    if depth == 0 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            collect_themes(&entry.path(), depth - 1, themes);
        }
    }
}

fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let target = to.join(entry.file_name());
        if kind.is_symlink() {
            std::os::unix::fs::symlink(fs::read_link(entry.path())?, &target)?;
        } else if kind.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icons::tests::write_theme;

    #[test]
    fn installs_a_single_theme_folder_with_symlinks() {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("src/Neat");
        let dest = tmp.path().join("icons");
        write_theme(&source, "Neat", "hicolor", &["folder"]);
        std::os::unix::fs::symlink(
            "folder.svg",
            source.join("48x48/places/inode-directory.svg"),
        )
        .unwrap();

        let report = install_dir(&source, &dest).unwrap();

        assert_eq!(report.installed, ["Neat"]);
        let link = dest.join("Neat/48x48/places/inode-directory.svg");
        assert_eq!(fs::read_link(&link).unwrap(), Path::new("folder.svg"));
        assert!(dest.join("Neat/index.theme").is_file());
    }

    #[test]
    fn installs_every_theme_in_a_pack_and_skips_existing_ones() {
        let tmp = tempfile::tempdir().unwrap();
        let pack = tmp.path().join("pack-master");
        let dest = tmp.path().join("icons");
        write_theme(&pack.join("Pack-Dark"), "Pack Dark", "", &[]);
        write_theme(&pack.join("Pack-Light"), "Pack Light", "", &[]);
        fs::create_dir_all(dest.join("Pack-Light")).unwrap();

        let report = install_dir(&pack, &dest).unwrap();

        assert_eq!(report.installed, ["Pack-Dark"]);
        assert_eq!(
            report.failed,
            [("Pack-Light".to_string(), "already installed".to_string())]
        );
    }

    #[test]
    fn folder_without_a_theme_is_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        let err = install_dir(tmp.path(), &tmp.path().join("icons")).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn installs_themes_from_a_tar_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let pack = tmp.path().join("build/Archive-Pack");
        write_theme(&pack.join("Archived"), "Archived", "", &["folder"]);
        let archive = tmp.path().join("pack.tar.gz");
        let status = Command::new("tar")
            .arg("-czf")
            .arg(&archive)
            .arg("-C")
            .arg(tmp.path().join("build"))
            .arg("Archive-Pack")
            .status()
            .unwrap();
        assert!(status.success());

        let dest = tmp.path().join("icons");
        let report = install_archive(&archive, &dest).unwrap();

        assert_eq!(report.installed, ["Archived"]);
        assert!(dest.join("Archived/48x48/places/folder.svg").is_file());
        // The staging folder is cleaned up.
        assert_eq!(fs::read_dir(&dest).unwrap().count(), 1);
    }

    #[test]
    fn broken_archive_is_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = tmp.path().join("broken.tar.gz");
        fs::write(&archive, "not an archive").unwrap();

        let dest = tmp.path().join("icons");
        assert!(install_archive(&archive, &dest).is_err());
        assert_eq!(fs::read_dir(&dest).unwrap().count(), 0);
    }
}
