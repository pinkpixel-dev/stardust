# Changelog

## 0.1.0 - October 3, 2026

### 🎨 App

- Initial libcosmic app shell with a header bar, View > About menu, and an empty Themes page
- App icon and desktop entry (`dev.pinkpixel.Starcoat`)

### 📚 Theme library

- Saved themes live in `~/.local/share/starcoat/themes/` as `.ron` files
- First launch saves your current COSMIC theme as "My Theme"
- Import theme files (Ctrl+O) or a whole folder (Ctrl+Shift+O), with a toast saying how many worked
- Theme cards show the background and accent colors plus a dark/light label
- Warning banner if the installed COSMIC uses a different theme config version than Starcoat

### 🧹 Maintenance

- libcosmic pinned to the revision COSMIC 1.9.0 ships
- Apache-2.0 license
