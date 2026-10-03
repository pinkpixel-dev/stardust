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

### Decision: Compare themes by their saved RON, not with `==`

What was decided: `library::same_theme` serializes both `ThemeBuilder`s with `ron::to_string` and compares the strings. `AppModel::refresh_active` uses it to find the active theme.

Why: Old `v1` config files store colors as raw floats (`0.99203914`), and saved theme files store hex. After the hex round trip the floats differ slightly, so `ThemeBuilder` `==` said "different" for a theme that was identical on screen. A check against the live config on CachyOS returned `exact_eq=false same_theme=true`.

Rejected: Plain `PartialEq`, which leaves the checkmark missing on any desktop with `v1` leftovers. Comparing with a float tolerance, which needs a hand-written field-by-field compare that would break whenever `ThemeBuilder` gains fields.

### Decision: Applying a theme doesn't change automatic day/night switching

What was decided: `desktop::apply` writes the theme to its dark or light slot and sets `is_dark`, but never touches `auto_switch` in `com.system76.CosmicTheme.Mode`.

Why: That's a desktop-wide preference the user set in COSMIC Settings. Turning it off silently would be surprising.

Rejected: Disabling `auto_switch` on apply so the chosen theme always stays visible.
