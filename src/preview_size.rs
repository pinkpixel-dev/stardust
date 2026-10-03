// SPDX-License-Identifier: Apache-2.0

//! How big theme cards are drawn, remembered between launches.

use cosmic::cosmic_config::{Config, ConfigGet, ConfigSet};

const CONFIG_VERSION: u64 = 1;
const KEY: &str = "preview_size";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PreviewSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl PreviewSize {
    /// Multiplier applied to the card and every measurement in the preview.
    pub fn scale(self) -> f32 {
        match self {
            PreviewSize::Small => 1.0,
            PreviewSize::Medium => 1.3,
            PreviewSize::Large => 1.6,
        }
    }

    pub fn larger(self) -> Self {
        match self {
            PreviewSize::Small => PreviewSize::Medium,
            _ => PreviewSize::Large,
        }
    }

    pub fn smaller(self) -> Self {
        match self {
            PreviewSize::Large => PreviewSize::Medium,
            _ => PreviewSize::Small,
        }
    }

    fn key(self) -> &'static str {
        match self {
            PreviewSize::Small => "small",
            PreviewSize::Medium => "medium",
            PreviewSize::Large => "large",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        match key {
            "small" => Some(PreviewSize::Small),
            "medium" => Some(PreviewSize::Medium),
            "large" => Some(PreviewSize::Large),
            _ => None,
        }
    }

    /// The saved size, or the default when nothing was saved or it can't be read.
    pub fn load(app_id: &str) -> Self {
        Config::new(app_id, CONFIG_VERSION)
            .and_then(|config| config.get::<String>(KEY))
            .ok()
            .and_then(|key| Self::from_key(&key))
            .unwrap_or_default()
    }

    pub fn save(self, app_id: &str) {
        let result = Config::new(app_id, CONFIG_VERSION)
            .and_then(|config| config.set(KEY, self.key().to_string()));
        if let Err(err) = result {
            eprintln!("failed to save preview size: {err}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PreviewSize::{self, *};

    #[test]
    fn steps_stop_at_the_ends() {
        assert_eq!(Small.smaller(), Small);
        assert_eq!(Small.larger(), Medium);
        assert_eq!(Medium.larger(), Large);
        assert_eq!(Large.larger(), Large);
        assert_eq!(Large.smaller(), Medium);
    }

    #[test]
    fn keys_round_trip() {
        for size in [Small, Medium, Large] {
            assert_eq!(PreviewSize::from_key(size.key()), Some(size));
        }
        assert_eq!(PreviewSize::from_key("huge"), None);
    }
}
