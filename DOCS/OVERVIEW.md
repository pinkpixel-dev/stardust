# Starcoat Overview

Starcoat is a native COSMIC desktop app for saving, previewing, creating, and switching themes. The goal is to replace the "export a `.ron` file, then pick it again in Settings every time" routine with a grid of theme previews you can apply with one click.

This document describes how the project works right now. It is present tense. Decisions and their reasoning live in `MEMORY.md`. Release history lives in `CHANGELOG.md`.

## Stack

- Rust (edition 2024)
- [libcosmic](https://github.com/pop-os/libcosmic), pinned to rev `d77e99fb1555e2728297677de61e23ada4d9d09b` (the revision COSMIC epoch 1.9.0 ships)
- `i18n-embed` + Fluent for UI strings (`i18n/en/starcoat.ftl`)

## Current State

Right now the app is a scaffold. It opens a window with a header bar, a View > About menu, and a Themes page that shows an empty state. The theme library, previews, apply logic, and editor are not built yet. See `ROADMAP.md`.

## How COSMIC Themes Work

This is the part the app is built around, so it's worth writing down.

COSMIC stores the active theme as config files under `~/.config/cosmic/`, one file per key:

- `com.system76.CosmicTheme.Dark.Builder/v2/` and `.Light.Builder/v2/`: the editable "recipe" (`accent`, `bg_color`, `primary_container_bg`, `neutral_tint`, `text_tint`, `corner_radii`, `is_frosted`, and so on). An exported `.ron` theme file is a serialized `ThemeBuilder`.
- `com.system76.CosmicTheme.Dark/v2/` and `.Light/v2/`: the full theme generated from the builder.
- `com.system76.CosmicTheme.Mode/`: whether dark or light mode is active.

Applying a theme means parsing the `.ron` into a `ThemeBuilder`, switching dark/light mode if `palette.is_dark()` differs, then writing both the builder and `builder.build()` to config with `write_entry`. COSMIC apps and the compositor watch these files, so changes apply live.

The `v2` folder name is the config format version (`#[version = 2]` on `ThemeBuilder` in `cosmic-theme`). COSMIC 1.7.0 and 1.9.0 both use `v2`. Older `v1` folders can still be sitting next to it.

## Project Structure

```text
src/
  main.rs   app entry, window settings
  app.rs    app model, header menu, About drawer, Themes view
  i18n.rs   Fluent localization loader and fl! macro
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
cargo build --release
```

## Current Limits

- No theme library, previews, apply, or editor yet.
- `cargo install` only installs the binary. The desktop entry and icon in `resources/` are not installed anywhere yet.
- Only tested on CachyOS with COSMIC 1.7.0. Not tested on Pop!_OS yet.
