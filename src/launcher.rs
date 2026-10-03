// SPDX-License-Identifier: Apache-2.0

//! Adds Stardust's desktop entry and icon to the user's data folder.
//!
//! `cargo install` only installs the binary, so without this the launcher,
//! dock, and window have no entry or icon to show.

use std::fs;
use std::io;
use std::path::Path;

const APP_ID: &str = "dev.pinkpixel.Stardust";
const DESKTOP_ENTRY: &str = include_str!("../resources/dev.pinkpixel.Stardust.desktop");
const ICON: &[u8] =
    include_bytes!("../resources/icons/hicolor/512x512/apps/dev.pinkpixel.Stardust.png");

/// Installs the desktop entry and icon for the running binary.
///
/// Skipped in debug builds so `cargo run` doesn't point the launcher at a
/// dev binary.
pub fn install() {
    if cfg!(debug_assertions) {
        return;
    }
    let (Some(data_dir), Ok(exe)) = (dirs::data_dir(), std::env::current_exe()) else {
        return;
    };
    if let Err(err) = install_into(&data_dir, &exe) {
        eprintln!("failed to install desktop entry: {err}");
    }
}

/// Writes both files under `data_dir`, skipping any that are already current.
fn install_into(data_dir: &Path, exe: &Path) -> io::Result<()> {
    let entry = desktop_entry(exe);
    write_if_changed(
        &data_dir.join(format!("applications/{APP_ID}.desktop")),
        entry.as_bytes(),
    )?;
    write_if_changed(
        &data_dir.join(format!("icons/hicolor/512x512/apps/{APP_ID}.png")),
        ICON,
    )
}

/// The bundled desktop entry with `Exec` pointing at `exe`, so it works
/// even when `~/.cargo/bin` isn't on the session's PATH.
fn desktop_entry(exe: &Path) -> String {
    let exec = format!("Exec={}", quote_exec(&exe.to_string_lossy()));
    DESKTOP_ENTRY
        .lines()
        .map(|line| if line.starts_with("Exec=") { exec.as_str() } else { line })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

/// Quotes a path for the `Exec` key as the desktop entry spec describes.
fn quote_exec(path: &str) -> String {
    let mut quoted = String::from("\"");
    for c in path.chars() {
        if matches!(c, '"' | '`' | '$' | '\\') {
            quoted.push('\\');
        }
        quoted.push(c);
    }
    quoted.push('"');
    quoted
}

fn write_if_changed(path: &Path, contents: &[u8]) -> io::Result<()> {
    if fs::read(path).is_ok_and(|current| current == contents) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_points_at_the_binary() {
        let entry = desktop_entry(Path::new("/home/me/.cargo/bin/stardust"));
        assert!(entry.contains("\nExec=\"/home/me/.cargo/bin/stardust\"\n"));
        assert!(entry.contains(&format!("Icon={APP_ID}")));
        assert_eq!(entry.matches("Exec=").count(), 1);
    }

    #[test]
    fn exec_escapes_reserved_characters() {
        assert_eq!(quote_exec("/a b/$x\"y"), "\"/a b/\\$x\\\"y\"");
    }

    #[test]
    fn installs_entry_and_icon_and_updates_stale_entry() {
        let dir = tempfile::tempdir().unwrap();
        let entry_path = dir.path().join(format!("applications/{APP_ID}.desktop"));
        let icon_path = dir
            .path()
            .join(format!("icons/hicolor/512x512/apps/{APP_ID}.png"));

        install_into(dir.path(), Path::new("/old/stardust")).unwrap();
        assert_eq!(fs::read(&icon_path).unwrap(), ICON);
        assert!(fs::read_to_string(&entry_path).unwrap().contains("/old/stardust"));

        install_into(dir.path(), Path::new("/new/stardust")).unwrap();
        assert!(fs::read_to_string(&entry_path).unwrap().contains("/new/stardust"));
    }
}
