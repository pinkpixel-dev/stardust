// SPDX-License-Identifier: Apache-2.0

//! Finds installed icon themes and reads and sets the one COSMIC uses.
//!
//! libcosmic's own icon lookup builds its theme list once per process, so a
//! theme imported while Stardust is open wouldn't show up there. Scanning the
//! folders ourselves keeps the Icons page current.

mod index;
mod install;

pub use install::{InstallReport, install_archive, install_dir, user_dir};

use cosmic::config::CosmicTk;
use cosmic::cosmic_config::{ConfigSet, CosmicConfigEntry, Error};
use index::Index;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Icons shown on each card, in order. Themes don't agree on names, so each
/// slot lists a few to try.
const PREVIEW_ICONS: &[&[&str]] = &[
    &["folder", "inode-directory"],
    &["user-home", "folder-home"],
    &["folder-download", "folder-downloads"],
    &["user-trash", "user-trash-empty"],
    &["text-x-generic", "text-plain"],
    &["image-x-generic", "image"],
    &["audio-x-generic", "audio"],
    &["video-x-generic", "video"],
    &[
        "utilities-terminal",
        "org.gnome.Terminal",
        "com.system76.CosmicTerm",
        "terminal",
    ],
    &[
        "system-file-manager",
        "org.gnome.Nautilus",
        "com.system76.CosmicFiles",
    ],
    &["web-browser", "internet-web-browser", "firefox"],
    &[
        "preferences-system",
        "com.system76.CosmicSettings",
        "preferences-desktop",
    ],
];

/// What COSMIC uses when nothing is set.
const DEFAULT_THEME: &str = "Cosmic";

#[derive(Clone, Debug)]
pub struct IconTheme {
    /// Folder name. This is the value COSMIC stores.
    pub id: String,
    /// Display name from `index.theme`.
    pub name: String,
    /// Icon files for the card, at most one per preview slot.
    pub previews: Vec<PathBuf>,
}

/// The folders icon themes are installed in, highest priority first.
pub fn base_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".icons"));
    }
    if let Some(data) = dirs::data_dir() {
        dirs.push(data.join("icons"));
    }
    let data_dirs = std::env::var_os("XDG_DATA_DIRS")
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    dirs.extend(std::env::split_paths(&data_dirs).map(|dir| dir.join("icons")));

    let mut seen = HashSet::new();
    dirs.retain(|dir| seen.insert(dir.clone()));
    dirs
}

/// Lists every visible icon theme in `bases`, sorted by name.
pub fn scan(bases: &[PathBuf]) -> Vec<IconTheme> {
    let installed = Installed::read(bases);

    let mut themes: Vec<IconTheme> = installed
        .themes
        .iter()
        .filter(|(id, theme)| id.as_str() != "hicolor" && !theme.index.hidden)
        .map(|(id, theme)| IconTheme {
            id: id.clone(),
            name: if theme.index.name.is_empty() {
                id.clone()
            } else {
                theme.index.name.clone()
            },
            previews: PREVIEW_ICONS
                .iter()
                .filter_map(|names| installed.find(id, names))
                .collect(),
        })
        .collect();

    themes.sort_by_cached_key(|theme| (theme.name.to_lowercase(), theme.id.clone()));
    themes
}

/// The icon theme COSMIC is set to right now.
pub fn current() -> Result<String, Error> {
    let config = CosmicTk::config()?;
    let tk = match CosmicTk::get_entry(&config) {
        Ok(tk) => tk,
        Err((_errors, tk)) => tk,
    };
    Ok(if tk.icon_theme.is_empty() {
        DEFAULT_THEME.to_string()
    } else {
        tk.icon_theme
    })
}

/// Sets COSMIC's icon theme. COSMIC apps watch this key and switch live.
pub fn apply(id: &str) -> Result<(), Error> {
    CosmicTk::config()?.set("icon_theme", id.to_string())
}

struct Installed {
    themes: HashMap<String, Found>,
}

struct Found {
    /// Every folder with this theme's name. A theme can be split across
    /// several base folders, and they're searched in priority order.
    roots: Vec<PathBuf>,
    index: Index,
}

impl Installed {
    fn read(bases: &[PathBuf]) -> Self {
        let mut roots: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for base in bases {
            let Ok(entries) = std::fs::read_dir(base) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                // is_dir follows symlinks, which some distros use for themes.
                if path.is_dir() {
                    let id = entry.file_name().to_string_lossy().into_owned();
                    roots.entry(id).or_default().push(path);
                }
            }
        }

        let themes = roots
            .into_iter()
            .filter_map(|(id, roots)| {
                // The highest priority folder with an index.theme describes the theme.
                let index = roots.iter().find_map(|root| read_index(root))?;
                Some((id, Found { roots, index }))
            })
            .collect();

        Installed { themes }
    }

    /// Looks for one of `names` in the theme, then in the themes it
    /// inherits from, then in hicolor, which every theme falls back to.
    fn find(&self, id: &str, names: &[&str]) -> Option<PathBuf> {
        let mut seen = HashSet::new();
        self.find_in(id, names, &mut seen)
            .or_else(|| self.find_in("hicolor", names, &mut seen))
    }

    fn find_in(&self, id: &str, names: &[&str], seen: &mut HashSet<String>) -> Option<PathBuf> {
        if !seen.insert(id.to_string()) {
            return None;
        }
        let theme = self.themes.get(id)?;

        // A theme's own icon under a fallback name looks more like the theme
        // than its parent's icon under the first name.
        for name in names {
            for dir in &theme.index.dirs {
                for root in &theme.roots {
                    if let Some(path) = icon_file(&root.join(dir), name) {
                        return Some(path);
                    }
                }
            }
        }

        theme
            .index
            .inherits
            .iter()
            .find_map(|parent| self.find_in(parent, names, seen))
    }
}

fn read_index(root: &Path) -> Option<Index> {
    index::parse(&std::fs::read_to_string(root.join("index.theme")).ok()?)
}

fn icon_file(dir: &Path, name: &str) -> Option<PathBuf> {
    ["svg", "png"]
        .iter()
        .map(|ext| dir.join(format!("{name}.{ext}")))
        .find(|path| path.is_file())
}

/// Reads `index.theme` in `dir`, for checking whether a folder is an icon theme.
fn is_icon_theme(dir: &Path) -> bool {
    read_index(dir).is_some()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::fs;

    /// Writes a small icon theme with the given icons in `48x48/places`.
    pub(crate) fn write_theme(dir: &Path, name: &str, inherits: &str, icons: &[&str]) {
        let places = dir.join("48x48/places");
        fs::create_dir_all(&places).unwrap();
        fs::write(
            dir.join("index.theme"),
            format!(
                "[Icon Theme]\nName={name}\nInherits={inherits}\nDirectories=48x48/places\n\
                 [48x48/places]\nSize=48\nContext=Places\nType=Fixed\n"
            ),
        )
        .unwrap();
        for icon in icons {
            fs::write(places.join(format!("{icon}.svg")), "<svg/>").unwrap();
        }
    }

    #[test]
    fn scan_lists_icon_themes_and_skips_cursors_and_hicolor() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().to_path_buf();
        write_theme(&base.join("zeta"), "Zeta Icons", "hicolor", &["folder"]);
        write_theme(&base.join("alpha"), "Alpha Icons", "hicolor", &["folder"]);
        write_theme(&base.join("hicolor"), "Hicolor", "", &[]);
        fs::create_dir_all(base.join("cursors")).unwrap();
        fs::write(
            base.join("cursors/index.theme"),
            "[Icon Theme]\nName=Cursors\n",
        )
        .unwrap();

        let themes = scan(&[base]);
        let ids: Vec<_> = themes.iter().map(|theme| theme.id.as_str()).collect();
        assert_eq!(ids, ["alpha", "zeta"]);
        assert_eq!(themes[0].name, "Alpha Icons");
    }

    #[test]
    fn previews_prefer_own_icons_then_parents_then_hicolor() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().to_path_buf();
        // Own icon under a fallback name beats the parent's first-choice name.
        write_theme(&base.join("child"), "Child", "parent", &["folder-home"]);
        write_theme(
            &base.join("parent"),
            "Parent",
            "hicolor",
            &["user-home", "folder"],
        );
        write_theme(&base.join("hicolor"), "Hicolor", "", &["user-trash"]);

        let installed = Installed::read(std::slice::from_ref(&base));
        let found = |names: &[&str]| installed.find("child", names).unwrap();

        assert_eq!(
            found(&["user-home", "folder-home"]),
            base.join("child/48x48/places/folder-home.svg")
        );
        assert_eq!(
            found(&["folder"]),
            base.join("parent/48x48/places/folder.svg")
        );
        assert_eq!(
            found(&["user-trash"]),
            base.join("hicolor/48x48/places/user-trash.svg")
        );
        assert!(installed.find("child", &["missing"]).is_none());
    }

    #[test]
    fn inheritance_loops_end() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().to_path_buf();
        write_theme(&base.join("a"), "A", "b", &[]);
        write_theme(&base.join("b"), "B", "a", &[]);

        assert!(Installed::read(&[base]).find("a", &["folder"]).is_none());
    }

    #[test]
    fn user_folder_wins_over_system_folder() {
        let tmp = tempfile::tempdir().unwrap();
        let (user, system) = (tmp.path().join("user"), tmp.path().join("system"));
        write_theme(&user.join("same"), "User Copy", "", &[]);
        write_theme(&system.join("same"), "System Copy", "", &["folder"]);

        let themes = scan(&[user, system.clone()]);
        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].name, "User Copy");
        // Icons are still found in the lower priority folder.
        assert_eq!(
            themes[0].previews,
            [system.join("same/48x48/places/folder.svg")]
        );
    }
}
