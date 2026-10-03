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

Starcoat has a working theme library. It shows saved themes as cards (background and accent swatches, name, dark/light label) and can import theme files or whole folders. Applying themes, full previews, and the editor are not built yet. See `ROADMAP.md`.

## Core Flow

1. On startup, `compat::check()` compares the highest `v<N>` folder in `<XDG data dir>/cosmic/com.system76.CosmicTheme.Dark.Builder/` (installed by the COSMIC packages) with `ThemeBuilder::VERSION`. If they differ, a dismissible warning banner explains which side needs updating.
2. `Library::open` reads every `.ron` file in `~/.local/share/starcoat/themes/`. Files that don't parse as a `ThemeBuilder` are skipped and left on disk.
3. On first run (the folder didn't exist yet), `desktop::current_theme()` reads the live theme for the current dark/light mode and saves it as "My Theme".
4. File > Import Themes… (Ctrl+O) and File > Import Folder… (Ctrl+Shift+O) open the XDG portal file picker. Imports are copied into the library, and a toast reports how many worked.

## Theme Library

- One file per theme: `~/.local/share/starcoat/themes/<name>.ron`. The file name is the theme name. There's no separate metadata file.
- Names that are already taken (case-insensitive) get " 2", " 3", and so on. `/`, control characters, and leading dots are stripped.
- Files are written with `ron::ser::to_string_pretty`, the same way COSMIC Settings exports them, so a library file can be imported back into Settings.
- Each `SavedTheme` keeps the parsed `ThemeBuilder` plus the full `Theme` from `builder.build()`, which the views use for colors.

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
  desktop.rs       reads the live desktop theme
  i18n.rs          Fluent localization loader and fl! macro
  views/themes.rs  theme cards and empty state
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

- Themes can't be applied, renamed, or deleted from the app yet. Cards only show swatches, not a full preview.
- No editor yet.
- `cargo install` only installs the binary. The desktop entry and icon in `resources/` are not installed anywhere yet.
- Only tested on CachyOS with COSMIC 1.7.0. Not tested on Pop!_OS yet.
