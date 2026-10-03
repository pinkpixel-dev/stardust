# Starcoat Overview

Starcoat is a native COSMIC desktop app for saving, previewing, creating, and switching themes. The goal is to replace the "export a `.ron` file, then pick it again in Settings every time" routine with a grid of theme previews you can apply with one click.

This document describes how the project works right now. It is present tense. Decisions and their reasoning live in `MEMORY.md`. Release history lives in `CHANGELOG.md`.

## Stack

- Rust (edition 2024)
- [libcosmic](https://github.com/pop-os/libcosmic), pinned to rev `d77e99fb1555e2728297677de61e23ada4d9d09b` (the revision COSMIC epoch 1.9.0 ships)
- `i18n-embed` + Fluent for UI strings (`i18n/en/starcoat.ftl`)
- `ron` for reading and writing theme files, `dirs` for the data folder
- `tempfile` for tests

## Current State

Starcoat has a working theme library with previews. Each saved theme shows as a card with a tiny COSMIC window drawn from its colors, and clicking a card applies it to the desktop. Theme files and whole folders can be imported. The editor isn't built yet. See `ROADMAP.md`.

## Core Flow

1. On startup, `compat::check()` compares the highest `v<N>` folder in `<XDG data dir>/cosmic/com.system76.CosmicTheme.Dark.Builder/` (installed by the COSMIC packages) with `ThemeBuilder::VERSION`. If they differ, a dismissible warning banner explains which side needs updating.
2. `Library::open` reads every `.ron` file in `~/.local/share/starcoat/themes/`. Files that don't parse as a `ThemeBuilder` are skipped and left on disk.
3. On first run (the folder didn't exist yet), `desktop::current_theme()` reads the live theme for the current dark/light mode and saves it as "My Theme".
4. File > Import Themes… (Ctrl+O) and File > Import Folder… (Ctrl+Shift+O) open the XDG portal file picker. Imports are copied into the library, and a toast reports how many worked.
5. Clicking a card (or focusing it and pressing Enter) calls `desktop::apply`. If the version check found a mismatch, it shows the warning as a toast instead of writing anything.
6. Starcoat watches the dark builder, light builder, and mode configs. Any change, from Starcoat or from COSMIC Settings, re-runs `refresh_active`, which puts the checkmark on the saved theme that matches the desktop.

## Theme Library

- One file per theme: `~/.local/share/starcoat/themes/<name>.ron`. The file name is the theme name. There's no separate metadata file.
- Names that are already taken (case-insensitive) get " 2", " 3", and so on. `/`, control characters, and leading dots are stripped.
- Files are written with `ron::ser::to_string_pretty`, the same way COSMIC Settings exports them, so a library file can be imported back into Settings.
- Each `SavedTheme` keeps the parsed `ThemeBuilder` plus the full `Theme` from `builder.build()`, which the views use for colors.
- `library::same_theme` compares two builders by their serialized RON, not with `==`. Old `v1` config values are raw floats and saved files are hex, so `==` fails on themes that look identical.

## Applying A Theme

`desktop::apply(builder)`:

1. Picks the dark or light slot from `builder.palette.is_dark()`.
2. Writes the builder to `com.system76.CosmicTheme.<Mode>.Builder` and `builder.build()` to `com.system76.CosmicTheme.<Mode>` with `write_entry`. Every key is written, so `v1` leftovers stop showing through.
3. Sets `is_dark` on `com.system76.CosmicTheme.Mode` last, only if it needs to change.

It doesn't touch `auto_switch`. If automatic day/night switching is on, COSMIC can still flip modes later and show whatever is in the other slot.

## Previews

`views/preview.rs` draws a small window: a title bar, a sidebar using the secondary container color with one accent-tinted selected item, and a content area using the primary container color with text bars and an accent button. The border is the window hint color (or the accent if none is set). Corner radii come from the theme's `radius_m`, `radius_s`, and `radius_xl`, scaled by 0.4.

## How COSMIC Themes Work

This is the part the app is built around, so it's worth writing down.

COSMIC stores the active theme as config files under `~/.config/cosmic/`, one file per key:

- `com.system76.CosmicTheme.Dark.Builder/v2/` and `.Light.Builder/v2/`: the editable "recipe" (`accent`, `bg_color`, `primary_container_bg`, `neutral_tint`, `text_tint`, `corner_radii`, `frosted` (a blur strength level, not on/off), and so on). An exported `.ron` theme file is a serialized `ThemeBuilder`.
- `com.system76.CosmicTheme.Dark/v2/` and `.Light/v2/`: the full theme generated from the builder.
- `com.system76.CosmicTheme.Mode/`: whether dark or light mode is active.

Applying a theme means parsing the `.ron` into a `ThemeBuilder`, switching dark/light mode if `palette.is_dark()` differs, then writing both the builder and `builder.build()` to config with `write_entry`. COSMIC apps and the compositor watch these files, so changes apply live.

Reads fall back per key. If a key file is missing from `v2`, cosmic-config reads it from `v1` (one version back only), then from the system defaults in `/usr/share/cosmic/`. A desktop can end up showing a mix of `v2` and `v1` values, and `desktop::current_theme()` sees the same mix because it reads through cosmic-config.

Old exports store colors as RGB structs and newer ones use hex strings. `cosmic-theme` reads both.

The `v2` folder name is the config format version (`#[version = 2]` on `ThemeBuilder` in `cosmic-theme`). COSMIC 1.7.0 and 1.9.0 both use `v2`, and `cosmic-theme` is identical between their libcosmic revisions. Older `v1` folders can still be sitting next to it.

## Project Structure

```text
src/
  main.rs          app entry, window settings
  app.rs           app model, menus, key binds, import dialogs, toasts
  library.rs       theme library: load, save, import, naming (with tests)
  compat.rs        desktop theme config version check (with tests)
  desktop.rs       reads and applies the desktop theme (with tests)
  i18n.rs          Fluent localization loader and fl! macro
  views/themes.rs  theme card grid and empty state
  views/preview.rs mini window preview drawn from a Theme
i18n/en/starcoat.ftl          UI strings
resources/
  dev.pinkpixel.Starcoat.desktop
  icons/hicolor/scalable/apps/icon.svg
```

App ID: `dev.pinkpixel.Starcoat`

Project documentation lives in `/DOCS`.

- `/DOCS/OVERVIEW.md`: current architecture and behavior
- `/DOCS/MEMORY.md`: significant decisions and rejected alternatives
- `/DOCS/ERRORS.md`: difficult bugs, failed approaches, and reusable fixes
- `/DOCS/ROADMAP.md`: planned phases
- `README.md`: user-facing project documentation
- `CHANGELOG.md`: release history

## Important Commands

```bash
cargo build
cargo run
cargo test
cargo build --release
```

## Current Limits

- Themes can't be renamed or deleted from the app yet.
- No editor yet.
- `cargo install` only installs the binary. The desktop entry and icon in `resources/` are not installed anywhere yet.
- Only tested on CachyOS with COSMIC 1.7.0. Not tested on Pop!_OS yet.
