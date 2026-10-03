// SPDX-License-Identifier: Apache-2.0

//! Reads and writes the theme the COSMIC desktop is using.

use cosmic::cosmic_config::{ConfigSet, CosmicConfigEntry, Error};
use cosmic::cosmic_theme::{Theme, ThemeBuilder, ThemeMode};

/// Returns the active theme's builder for the current dark/light mode.
///
/// Missing keys fall back the same way the desktop does (older config
/// version, then system defaults), so this matches what's on screen.
pub fn current_theme() -> Result<ThemeBuilder, Error> {
    let is_dark = ThemeMode::is_dark(&ThemeMode::config()?)?;
    let config = if is_dark {
        ThemeBuilder::dark_config()?
    } else {
        ThemeBuilder::light_config()?
    };

    Ok(match ThemeBuilder::get_entry(&config) {
        Ok(builder) => builder,
        // Some keys failed to load and were filled with defaults.
        Err((_errors, builder)) => builder,
    })
}

/// Applies a theme to the desktop.
///
/// Dark themes go in the dark slot and light themes in the light slot, then
/// the desktop switches to that mode if it isn't already in it. Every key is
/// written, so leftovers from older config versions stop showing through.
pub fn apply(builder: &ThemeBuilder) -> Result<(), Error> {
    let is_dark = builder.palette.is_dark();
    let (builder_config, theme_config) = if is_dark {
        (ThemeBuilder::dark_config()?, Theme::dark_config()?)
    } else {
        (ThemeBuilder::light_config()?, Theme::light_config()?)
    };

    builder.write_entry(&builder_config)?;
    builder.clone().build().write_entry(&theme_config)?;

    // Switch mode last, so the desktop flips straight to the new theme.
    let mode_config = ThemeMode::config()?;
    if ThemeMode::is_dark(&mode_config).ok() != Some(is_dark) {
        mode_config.set("is_dark", is_dark)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmic::cosmic_config::Config;
    use cosmic::cosmic_theme::DARK_THEME_BUILDER_ID;

    /// The active-theme checkmark compares a saved file against what's read
    /// back from config, so a round trip through both has to be lossless.
    #[test]
    fn theme_round_trips_through_file_and_config() {
        let tmp = tempfile::tempdir().unwrap();

        let mut original = ThemeBuilder::dark();
        original.accent = Some(cosmic::cosmic_theme::palette::Srgb::new(0.76, 0.44, 1.0));
        let ron = ron::ser::to_string_pretty(&original, Default::default()).unwrap();
        let from_file: ThemeBuilder = ron::from_str(&ron).unwrap();

        let config = Config::with_custom_path(
            DARK_THEME_BUILDER_ID,
            ThemeBuilder::VERSION,
            tmp.path().to_path_buf(),
        )
        .unwrap();
        from_file.write_entry(&config).unwrap();
        let from_config = ThemeBuilder::get_entry(&config).unwrap();

        assert_eq!(from_config, from_file);
    }
}
