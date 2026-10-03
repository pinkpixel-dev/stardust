# Changelog

## 0.1.0 - October 3, 2026

### 🎨 App

- Initial libcosmic app shell with a header bar, View > About menu, and an empty Themes page
- App icon and desktop entry (`dev.pinkpixel.Starcoat`)

### 📚 Theme library

- Saved themes live in `~/.local/share/starcoat/themes/` as `.ron` files
- First launch saves your current COSMIC theme as "My Theme"
- Import theme files (Ctrl+O) or a whole folder (Ctrl+Shift+O), with a toast saying how many worked
- Theme cards show a small preview window drawn from each theme's colors, corner style, and window hint
- Pick Small, Medium, or Large previews from the View menu, or step through them with Ctrl+= and Ctrl+-. Medium is the default, and Starcoat remembers your choice
- Click a card (or press Enter on it) to apply the theme. Starcoat switches between dark and light mode when needed
- A checkmark marks the saved theme that matches your desktop, and it stays in sync when you change themes in COSMIC Settings
- Warning banner if the installed COSMIC uses a different theme config version than Starcoat

### 🧹 Maintenance

- libcosmic pinned to the revision COSMIC 1.9.0 ships
- Apache-2.0 license
