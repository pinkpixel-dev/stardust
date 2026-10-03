# Changelog

## 0.2.0 - October 3, 2026

### 🖼️ Icon themes

- New Icons page, reached from the nav sidebar next to Themes
- Each installed icon theme gets a card with 12 of its real icons (folders, files, and a few apps), sized by the same Small, Medium, and Large setting as theme cards
- Click a card to set it as COSMIC's icon theme. A checkmark marks the current one and stays in sync with COSMIC Settings
- File > Import on the Icons page installs `.tar` archives (gz, xz, bz2, zst) or a theme folder into `~/.local/share/icons/`. Packs with several themes inside get every theme installed
- Cursor-only themes and hidden themes are left out of the grid

### 🧹 Maintenance

- File picker helpers moved out of the app module so both pages can share them

## 0.1.0 - October 3, 2026

### 🎨 App

- Initial libcosmic app shell with a header bar, View > About menu, and an empty Themes page
- App icon and desktop entry (`dev.pinkpixel.Stardust`)

### 📚 Theme library

- Saved themes live in `~/.local/share/stardust/themes/` as `.ron` files
- First launch saves your current COSMIC theme as "My Theme"
- Import theme files (Ctrl+O) or a whole folder (Ctrl+Shift+O), with a toast saying how many worked
- Theme cards show a small preview window drawn from each theme's colors, corner style, and window hint
- Pick Small, Medium, or Large previews from the View menu, or step through them with Ctrl+= and Ctrl+-. Medium is the default, and Stardust remembers your choice
- Click a card (or press Enter on it) to apply the theme. Stardust switches between dark and light mode when needed
- A checkmark marks the saved theme that matches your desktop, and it stays in sync when you change themes in COSMIC Settings
- Warning banner if the installed COSMIC uses a different theme config version than Stardust

### 🧹 Maintenance

- libcosmic pinned to the revision COSMIC 1.9.0 ships
- Apache-2.0 license
