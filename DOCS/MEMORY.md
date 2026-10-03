# Memory

Significant project decisions, the reasoning behind them, and the alternatives that were turned down.

Log a decision when a real fork in the road existed, the reason is not visible in the code, and reversing it later without knowing that reason would cost something. Skip routine implementation choices, feature announcements, and anything that belongs in `CHANGELOG.md` or `ROADMAP.md`.

## 2026-10-03

### Decision: Pin libcosmic to the revision COSMIC 1.9.0 ships

What was decided: `Cargo.toml` pins `libcosmic` to rev `d77e99fb1555e2728297677de61e23ada4d9d09b`, taken from `Cargo.lock` in `cosmic-settings` at tag `epoch-1.9.0`.

Why: The theme config format is versioned (`v1`, `v2` folders). If Starcoat writes a different version than the running desktop reads, applying a theme silently does nothing. Pop!_OS usually gets COSMIC updates before CachyOS, so tracking the newest stable epoch keeps Starcoat matched to Pop while still working on CachyOS 1.7.0 (both use `v2`). When a new COSMIC epoch ships, bump the rev to match it.

Rejected: Tracking libcosmic `master`, which can change the config format before any distro ships it. Using libcosmic's `v0.x` git tags, which are old and don't line up with COSMIC epoch releases.

### Decision: Install from git, not crates.io

What was decided: The install path is `cargo install --git https://github.com/pinkpixel-dev/starcoat`.

Why: crates.io doesn't allow git dependencies, and libcosmic, `cosmic-theme`, and `cosmic-config` aren't published there. `cargo publish` can't work until that changes.

Rejected: Publishing to crates.io (blocked by the above). Vendoring libcosmic into the crate (huge, and it would drift from the desktop's version).

### Decision: Write our own apply logic instead of copying COSMIC Settings

What was decided: Starcoat applies themes using public `cosmic-theme` APIs (`ThemeBuilder`, `build()`, `write_entry`) with its own code.

Why: Starcoat is Apache-2.0. libcosmic is MPL-2.0, which is fine to depend on. `cosmic-settings` is GPL-3.0, so copying its `theme_manager.rs` code would force the whole project to GPL.

Rejected: Copying the `Manager` code from `cosmic-settings/src/pages/desktop/appearance/theme_manager.rs`.

### Decision: Leave gaps and spacing out of the v1 editor

What was decided: The editor covers colors, frosted glass, corner style presets, and window hint. `gaps` and `spacing` are not editable in v1.

Why: Those two change the layout of every COSMIC app and are easy to break. Colors are what people actually want to swap.

Rejected: Exposing every `ThemeBuilder` field from the start.
