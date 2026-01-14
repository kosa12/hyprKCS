# hyprKCS

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![AUR version](https://img.shields.io/aur/version/hyprkcs-git)](https://aur.archlinux.org/packages/hyprkcs-git)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Binary Size](https://img.shields.io/badge/binary_size-~3MB-blue)](https://github.com/kosa12/hyprKCS)


A fast, minimal Hyprland keybind manager written in Rust with GTK4 and Libadwaita.

<p align="center">
  <img src="./assets/image_1.png" width="32%" />
  <img src="./assets/image_2.png" width="32%" />
  <img src="./assets/image_3.png" width="32%" />
</p>

<details>
  <summary align="center"><b>🎥 Click to see Live Demo</b></summary>
  <p align="center">
    <img src="./assets/livedemo_2.gif" width="100%" />
  </p>
</details>

## Features

- **Blazing Fast (lol):** Written in Rust.
- **Native Look:** Uses GTK4 and Libadwaita to match your system theme (supports Dark/Light mode and Matugen).
- **Category Filtering:** Quickly filter keybinds by type (Workspace, Window, Media, etc) using the dropdown.
- **Interactive Search:** Filter through your keybinds in real-time as you type (fuzzy).
- **Key Recorder:** Click "Record" and press a combination to automatically fill in modifiers and keys.
- **Conflict Detection:** Automatically highlights duplicate keybinds with a warning icon, resolving Hyprland variables (like `$mainMod`) for accuracy.
- **Sourced File Support:** Recursively parses files included via `source = ...`, allowing you to edit binds across your entire configuration.
- **Add, Edit & Delete:** Full CRUD support for your keybinds directly from the UI.
- **Toast Notifications:** Provides smooth, non-intrusive feedback for every action.
- **Safe Persistence:** Changes are written back to the correct configuration files automatically.
- **Backup Config:** Save your current config so you don't mess up something.
- **Conflict Wizard:** Step-by-step guide to resolve duplicate keybinds.
## Keyboard Shortcuts

| Key | Action |
| --- | --- |
| `/` | Focus search bar |
| `Enter` | Edit selected keybind |
| `Ctrl` + `f` | Focus search bar |
| `Esc` | Clear search / Close window |

## Installation

### AUR (Arch Linux)
```bash
yay -S hyprkcs-git
```

### Nix
Run directly without installing:
```bash
nix run github:kosa12/hyprKCS
```

### Manual Build
Ensure you have `rust`, `cargo`, and `gtk4` development headers installed.
```bash
git clone https://github.com/kosa12/hyprKCS.git
cd hyprKCS
cargo build --release
```
The binary will be available at `./target/release/hyprKCS`.

## Configuration

You can customize the appearance and behavior of hyprKCS by creating a configuration file at `~/.config/hyprkcs/hyprkcs.conf`.

```ini
# Window dimensions
width = 1000px
height = 800px

# Appearance
opacity = 0.95
fontSize = 10pt
borderSize = 2px
borderRadius = 10px

# UI Elements
showSubmaps = true
showArgs = true

# Spacing
monitorMargin = 20px
rowPadding = 5px
```

| Option | Description | Default |
| --- | --- | --- |
| `width` | Window width (px) | `700` |
| `height` | Window height (px) | `500` |
| `opacity` | Window background opacity (0.0 - 1.0) | `1.0` |
| `fontSize` | Font size for labels (supports px, pt, rem) | `0.9rem` |
| `borderSize` | Border thickness | `1px` |
| `borderRadius` | Main window border radius | `12px` |
| `showSubmaps` | Show/Hide the Submap column | `true` |
| `showArgs` | Show/Hide the Arguments column | `true` |
| `monitorMargin` | Margin around the window (px) | `12` |
| `rowPadding` | Vertical padding between rows (px) | `2` |

## CLI Usage

hyprKCS provides a robust command-line interface for quick lookups and configuration management.

```bash
hyprKCS --print #or hyprKCS -p
```
```bash
hyprKCS --search "firefox" #or hyprKCS -s "firefox"
```
```bash
hyprKCS --config ~/.config/hypr/custom_binds.conf #or hyprKCS -c ~/.config/hypr/custom_binds.conf
```
```bash
hyprKCS --help #or hyprKCS -h
```
```bash
hyprKCS --version #or hyprKCS -V
```

## Roadmap

The goal of hyprKCS is to become the ultimate keybind management tool for Hyprland.

- [x] **CLI Interface:** Search and print keybinds from terminal and other info.
- [x] **Fuzzy Search:** Quickly find keybinds as you type.
- [x] **Category Filter:** Filter keybinds by type (Workspace, Window, Media, etc).
- [x] **Vim-style Navigation:** Use `j`/`k` to navigate the list. etc.
- [x] **Quick Execute:** Test keybinds immediately via `hyprctl dispatch`.
- [x] **Autocomplete:** Smart suggestions for Hyprland dispatchers.
- [x] **Conflict Wizard:** Interactive resolution for duplicate binds.
- [x] **Auto-Reload:** Automatically trigger `hyprctl reload` after edits.
- [ ] **Pinned Binds:** Star your most-used binds to keep them at the top.
- [ ] **Visual Statistics:** Breakdown of your config by category and complexity.

## Troubleshooting

### GPG Key Import Issues
If you encounter errors like `gpg: keyserver receive failed` or `unknown public key` when installing from AUR, you need to import the PGP key manually.

Try importing from the Ubuntu keyserver:
```bash
gpg --keyserver keyserver.ubuntu.com --recv-keys D2059131FDE2EECC7C90A549F2CB939C8AA67892
```

Or from OpenPGP:
```bash
gpg --keyserver keys.openpgp.org --recv-keys D2059131FDE2EECC7C90A549F2CB939C8AA67892
```

## Maintainer:

**kosa12** (kosa03matyas@gmail.com)
- **PGP Fingerprint:** `D205 9131 FDE2 EECC 7C90 A549 F2CB 939C 8AA6 7892`

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT
