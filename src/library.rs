// SPDX-License-Identifier: Apache-2.0

//! The saved theme library: one `.ron` file per theme, named after the theme.

use cosmic::cosmic_theme::{Theme, ThemeBuilder};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const EXTENSION: &str = "ron";

#[derive(Debug, Clone)]
pub struct SavedTheme {
    pub name: String,
    pub builder: ThemeBuilder,
    /// The full theme generated from `builder`, used for previews.
    pub theme: Theme,
}

impl SavedTheme {
    fn new(name: String, builder: ThemeBuilder) -> Self {
        let theme = builder.clone().build();
        Self {
            name,
            builder,
            theme,
        }
    }

    pub fn is_dark(&self) -> bool {
        self.builder.palette.is_dark()
    }
}

#[derive(Debug)]
pub enum LibraryError {
    Io(io::Error),
    Parse(ron::error::SpannedError),
    Serialize(ron::Error),
}

impl fmt::Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Parse(err) => write!(f, "not a COSMIC theme file ({err})"),
            Self::Serialize(err) => write!(f, "could not write theme ({err})"),
        }
    }
}

impl From<io::Error> for LibraryError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

/// Result of importing several files at once.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ImportReport {
    pub imported: usize,
    pub failed: usize,
}

#[derive(Debug)]
pub struct Library {
    dir: PathBuf,
    themes: Vec<SavedTheme>,
}

impl Library {
    /// `~/.local/share/stardust/themes`
    pub fn default_dir() -> Option<PathBuf> {
        dirs::data_dir().map(|dir| dir.join("stardust").join("themes"))
    }

    /// Opens the library folder, creating it if needed. Returns the library
    /// and whether the folder was just created (first run).
    pub fn open(dir: PathBuf) -> io::Result<(Self, bool)> {
        let created = !dir.exists();
        fs::create_dir_all(&dir)?;

        let mut themes = Vec::new();
        for path in ron_files(&dir)? {
            // Files that don't parse are left alone on disk and skipped.
            if let Ok(builder) = read_theme(&path) {
                themes.push(SavedTheme::new(theme_name(&path), builder));
            }
        }

        let mut library = Self { dir, themes };
        library.sort();
        Ok((library, created))
    }

    pub fn themes(&self) -> &[SavedTheme] {
        &self.themes
    }

    /// Saves a theme under a name, adding " 2", " 3"... if the name is taken.
    pub fn save(&mut self, name: &str, builder: ThemeBuilder) -> Result<&SavedTheme, LibraryError> {
        let name = self.unique_name(&clean_name(name));
        let path = self.dir.join(format!("{name}.{EXTENSION}"));

        let contents = ron::ser::to_string_pretty(&builder, ron::ser::PrettyConfig::default())
            .map_err(LibraryError::Serialize)?;
        fs::write(&path, contents)?;

        self.themes.push(SavedTheme::new(name.clone(), builder));
        self.sort();

        Ok(self
            .themes
            .iter()
            .find(|theme| theme.name == name)
            .expect("theme was just added"))
    }

    /// Imports a single exported theme file, named after the file.
    pub fn import_file(&mut self, path: &Path) -> Result<&SavedTheme, LibraryError> {
        let builder = read_theme(path)?;
        self.save(&theme_name(path), builder)
    }

    /// Imports every `.ron` theme in a folder. Not recursive.
    pub fn import_dir(&mut self, dir: &Path) -> io::Result<ImportReport> {
        let mut report = ImportReport::default();
        for path in ron_files(dir)? {
            match self.import_file(&path) {
                Ok(_) => report.imported += 1,
                Err(_) => report.failed += 1,
            }
        }
        Ok(report)
    }

    fn unique_name(&self, base: &str) -> String {
        let taken = |name: &str| {
            self.themes
                .iter()
                .any(|theme| theme.name.eq_ignore_ascii_case(name))
                || self.dir.join(format!("{name}.{EXTENSION}")).exists()
        };

        if !taken(base) {
            return base.to_string();
        }

        (2..)
            .map(|n| format!("{base} {n}"))
            .find(|name| !taken(name))
            .expect("unbounded range always finds a free name")
    }

    fn sort(&mut self) {
        self.themes
            .sort_by_key(|theme| theme.name.to_lowercase());
    }
}

/// Parses an exported COSMIC theme. Accepts both the older RGB format and the
/// newer hex color format.
pub fn read_theme(path: &Path) -> Result<ThemeBuilder, LibraryError> {
    let contents = fs::read_to_string(path)?;
    ron::from_str(&contents).map_err(LibraryError::Parse)
}

/// True when two themes would save to the same file. Comparing the saved
/// form ignores tiny float differences, like an old RGB value such as
/// `0.99203914` versus the same color after a round trip through hex.
pub fn same_theme(a: &ThemeBuilder, b: &ThemeBuilder) -> bool {
    match (ron::to_string(a), ron::to_string(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

fn ron_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == EXTENSION))
        .collect();
    files.sort();
    Ok(files)
}

fn theme_name(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Makes a name safe to use as a file name.
fn clean_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c == '/' || c.is_control() { ' ' } else { c })
        .collect();
    let cleaned = cleaned.trim().trim_start_matches('.').trim();

    if cleaned.is_empty() {
        "Untitled".to_string()
    } else {
        cleaned.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_library() -> (tempfile::TempDir, Library) {
        let tmp = tempfile::tempdir().unwrap();
        let (library, created) = Library::open(tmp.path().join("themes")).unwrap();
        assert!(created);
        (tmp, library)
    }

    #[test]
    fn save_and_reload() {
        let (tmp, mut library) = temp_library();
        library.save("Nord Dark", ThemeBuilder::dark()).unwrap();
        library.save("Paper", ThemeBuilder::light()).unwrap();

        let (reloaded, created) = Library::open(tmp.path().join("themes")).unwrap();
        assert!(!created);
        let names: Vec<_> = reloaded.themes().iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, ["Nord Dark", "Paper"]);
        assert!(reloaded.themes()[0].is_dark());
        assert!(!reloaded.themes()[1].is_dark());
        assert_eq!(reloaded.themes()[0].builder, ThemeBuilder::dark());
    }

    #[test]
    fn duplicate_names_get_a_number() {
        let (_tmp, mut library) = temp_library();
        library.save("Nord", ThemeBuilder::dark()).unwrap();
        library.save("nord", ThemeBuilder::dark()).unwrap();
        library.save("Nord", ThemeBuilder::dark()).unwrap();

        let names: Vec<_> = library.themes().iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, ["Nord", "nord 2", "Nord 3"]);
    }

    #[test]
    fn unsafe_names_are_cleaned() {
        let (_tmp, mut library) = temp_library();
        let saved = library.save("../evil/name", ThemeBuilder::dark()).unwrap();
        assert_eq!(saved.name, "evil name");
        let saved = library.save("   ", ThemeBuilder::dark()).unwrap();
        assert_eq!(saved.name, "Untitled");
    }

    #[test]
    fn import_dir_counts_good_and_bad_files() {
        let (tmp, mut library) = temp_library();
        let source = tmp.path().join("downloads");
        fs::create_dir_all(&source).unwrap();

        let good = ron::ser::to_string_pretty(&ThemeBuilder::dark(), Default::default()).unwrap();
        fs::write(source.join("Midnight.ron"), &good).unwrap();
        fs::write(source.join("Broken.ron"), "not a theme").unwrap();
        fs::write(source.join("notes.txt"), "ignored").unwrap();

        let report = library.import_dir(&source).unwrap();
        assert_eq!(report, ImportReport { imported: 1, failed: 1 });
        assert_eq!(library.themes()[0].name, "Midnight");
    }

    #[test]
    fn same_theme_ignores_hex_rounding() {
        let mut exact = ThemeBuilder::dark();
        exact.bg_color = Some(cosmic::cosmic_theme::palette::Srgba::new(0.99203914, 0.42984885, 0.7296743, 1.0));
        let ron = ron::to_string(&exact).unwrap();
        let rounded: ThemeBuilder = ron::from_str(&ron).unwrap();

        assert_ne!(exact, rounded, "hex really does round");
        assert!(same_theme(&exact, &rounded));

        let mut other = rounded.clone();
        other.accent = Some(cosmic::cosmic_theme::palette::Srgb::new(0.1, 0.2, 0.3));
        assert!(!same_theme(&exact, &other));
    }

    #[test]
    fn reads_older_rgb_color_format() {
        let tmp = tempfile::tempdir().unwrap();
        let mut ron = ron::ser::to_string_pretty(&ThemeBuilder::dark(), Default::default()).unwrap();
        // COSMIC exports from before the hex format wrote colors as RGB structs.
        ron = ron.replacen(
            "accent: None",
            "accent: Some((red: 0.99, green: 0.43, blue: 0.73))",
            1,
        );
        let path = tmp.path().join("Old.ron");
        fs::write(&path, ron).unwrap();

        let builder = read_theme(&path).unwrap();
        let accent = builder.accent.expect("accent parsed");
        assert!((accent.red - 0.99).abs() < 0.01);
    }
}
