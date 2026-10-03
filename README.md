# Stardust

Stardust is a COSMIC desktop app for saving, previewing, creating, and switching themes.

I made it because switching themes in COSMIC means exporting a `.ron` file and picking it again in Settings every single time. I wanted a grid of my saved themes where I can see what each one looks like and apply it with one click.

> **Status:** early. Right now Stardust is just the app shell. The theme library, previews, and editor are being built next. See [DOCS/ROADMAP.md](DOCS/ROADMAP.md).

## Planned features

- A library of your saved themes, with previews drawn from each theme's real colors
- Apply a theme with one click (Stardust switches dark/light mode for you)
- Import existing `.ron` theme files
- Create a new theme in the app and save it

## Requirements

- The COSMIC desktop (built against COSMIC 1.9, works on 1.7)
- A recent stable Rust toolchain (I build with 1.96)

I use it on CachyOS. It should work on Pop!_OS too, since it only touches your user config in `~/.config/cosmic/`, but I haven't tested it there yet. If you try it on Pop, I'd love to hear how it goes.

## Install

```bash
cargo install --git https://github.com/pinkpixel-dev/stardust
```

Stardust isn't on crates.io because libcosmic isn't published there, and crates.io doesn't allow git dependencies. Installing from git works the same way.

## Build from source

```bash
git clone https://github.com/pinkpixel-dev/stardust
cd stardust
cargo run --release
```

## License

Apache-2.0. See [LICENSE](LICENSE).

Made with 💖 by [Pink Pixel](https://pinkpixel.dev)
