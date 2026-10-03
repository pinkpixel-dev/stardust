// SPDX-License-Identifier: Apache-2.0

//! Parses an icon theme's `index.theme` file.

use std::collections::HashMap;

/// The parts of `index.theme` Stardust needs.
#[derive(Debug, Default)]
pub struct Index {
    pub name: String,
    pub hidden: bool,
    pub inherits: Vec<String>,
    /// Subfolders that can hold the preview icons, largest icons first.
    pub dirs: Vec<String>,
}

#[derive(Default)]
struct Section {
    size: Option<u32>,
    max_size: Option<u32>,
    scalable: bool,
    context: String,
}

/// Contexts the preview icons come from. Skipping the rest (actions, status,
/// emblems, and so on) keeps lookups fast.
const PREVIEW_CONTEXTS: &[&str] = &["places", "mimetypes", "applications", "apps"];

/// Returns `None` for files that aren't icon themes, including cursor-only
/// themes, which have an `[Icon Theme]` section but no icon folders.
pub fn parse(text: &str) -> Option<Index> {
    let mut index = Index::default();
    let mut is_theme = false;
    let mut directories = Vec::new();
    let mut sections: HashMap<&str, Section> = HashMap::new();
    let mut section = "";

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = name;
            is_theme |= name == "Icon Theme";
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());

        if section == "Icon Theme" {
            match key {
                "Name" => index.name = value.to_string(),
                "Hidden" => index.hidden = value.eq_ignore_ascii_case("true"),
                "Inherits" => index.inherits = list(value),
                "Directories" | "ScaledDirectories" => directories.extend(list(value)),
                _ => {}
            }
        } else {
            let entry = sections.entry(section).or_default();
            match key {
                "Size" => entry.size = value.parse().ok(),
                "MaxSize" => entry.max_size = value.parse().ok(),
                "Type" => entry.scalable = value == "Scalable",
                "Context" => entry.context = value.to_ascii_lowercase(),
                _ => {}
            }
        }
    }

    if !is_theme || directories.is_empty() {
        return None;
    }

    let mut ranked: Vec<(u32, String)> = directories
        .into_iter()
        .filter_map(|dir| {
            let section = sections.get(dir.as_str())?;
            let wanted =
                section.context.is_empty() || PREVIEW_CONTEXTS.contains(&section.context.as_str());
            wanted.then(|| (rank(section), dir))
        })
        .collect();
    // Stable, so folders of the same size keep the theme's own order.
    ranked.sort_by_key(|(rank, _)| std::cmp::Reverse(*rank));
    index.dirs = ranked.into_iter().map(|(_, dir)| dir).collect();

    Some(index)
}

/// Bigger is better for previews. Scalable folders are usually SVGs that
/// look sharp at any size, unless they say they stop at a small size.
fn rank(section: &Section) -> u32 {
    if section.scalable {
        section.max_size.unwrap_or(256)
    } else {
        section.size.unwrap_or(0)
    }
}

fn list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_folders_largest_first_and_skips_other_contexts() {
        let index = parse(
            "[Icon Theme]\nName=Sample\nInherits=Adwaita, hicolor\n\
             Directories=16x16/places,48x48/places,scalable/places,scalable-up-to-32/places,48x48/actions\n\
             [16x16/places]\nSize=16\nContext=Places\nType=Fixed\n\
             [48x48/places]\nSize=48\nContext=Places\nType=Fixed\n\
             [scalable/places]\nSize=16\nContext=Places\nType=Scalable\n\
             [scalable-up-to-32/places]\nSize=16\nMaxSize=32\nContext=Places\nType=Scalable\n\
             [48x48/actions]\nSize=48\nContext=Actions\nType=Fixed\n",
        )
        .unwrap();

        assert_eq!(index.name, "Sample");
        assert_eq!(index.inherits, ["Adwaita", "hicolor"]);
        assert_eq!(
            index.dirs,
            [
                "scalable/places",
                "48x48/places",
                "scalable-up-to-32/places",
                "16x16/places"
            ]
        );
    }

    #[test]
    fn cursor_themes_are_not_icon_themes() {
        assert!(parse("[Icon Theme]\nName=Cursors\nInherits=Adwaita\n").is_none());
        assert!(parse("[Desktop Entry]\nName=Nope\n").is_none());
    }

    #[test]
    fn reads_hidden_flag() {
        let index = parse("[Icon Theme]\nName=X\nHidden=true\nDirectories=a\n[a]\nSize=48\n");
        assert!(index.unwrap().hidden);
    }
}
