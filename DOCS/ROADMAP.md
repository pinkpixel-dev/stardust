# Roadmap

## Phase 1: Scaffold (done)

- libcosmic app with header, About drawer, and an empty Themes page
- Apache-2.0 license, docs, app icon, desktop entry

## Phase 2: Theme library (done)

- Store themes as `.ron` files in `~/.local/share/starcoat/themes/`
- Import the current theme on first run
- Import one file or a whole folder
- Check which theme config version the running desktop uses and show one clear message if it doesn't match

## Phase 3: Preview grid

- Each card draws a tiny mock COSMIC window from the theme's real colors and corner radii
- Click or Enter applies the theme, switching dark/light mode if needed
- Checkmark on the active theme, dark/light shown with an icon and label

## Phase 4: Editor

- Start from a dark or light base, or duplicate an existing theme
- Edit accent, background, containers, tints, status colors, window hint, frosted, and corner style
- Live preview while editing, optional "apply while editing"
- Name it and save it to the library

## Phase 5: Polish

- Rename, delete (with confirm), export
- `starcoat --install-desktop` to add the launcher entry and icon after `cargo install`
- Full keyboard navigation and tooltips
- Test on Pop!_OS

## Maybe later

- Editing gaps and spacing
- Panel applet for quick switching
- Flatpak
- Automatic light/dark theme pairs
