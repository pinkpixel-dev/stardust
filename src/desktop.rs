// SPDX-License-Identifier: Apache-2.0

//! Reads the theme the COSMIC desktop is using right now.

use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::cosmic_theme::{ThemeBuilder, ThemeMode};

/// Returns the active theme's builder for the current dark/light mode.
///
/// Missing keys fall back the same way the desktop does (older config
/// version, then system defaults), so this matches what's on screen.
pub fn current_theme() -> Result<ThemeBuilder, cosmic::cosmic_config::Error> {
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
