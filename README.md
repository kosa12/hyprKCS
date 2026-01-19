# hyprKCS

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![AUR version](https://img.shields.io/aur/version/hyprkcs-git)](https://aur.archlinux.org/packages/hyprkcs-git)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Binary Size](https://img.shields.io/badge/binary_size-~3MB-blue)](https://github.com/kosa12/hyprKCS)

A fast, lightweight, and graphical keybind manager for Hyprland, built with Rust and GTK4.

<p align="center">
  <img src="./assets/image_1.png" width="32%" />
  <img src="./assets/image_2.png" width="32%" />
  <img src="./assets/image_3.png" width="32%" />
</p>

<details>
  <summary align="center">View a Live Demo</summary>
  <p align="center">
    <img src="./assets/livedemo_2.gif" width="100%" />
  </p>
</details>

## Overview

hyprKCS provides a simple and intuitive interface to view, edit, and manage your Hyprland keybinds. It automatically parses your `hyprland.conf` (and any sourced files), detects conflicts, and allows you to make changes safely.

## Features

- **Native GTK4 Interface**: Integrates seamlessly with your system theme, supporting both light and dark modes via Libadwaita.
- **Real-time Fuzzy Search**: Instantly find keybinds as you type.
- **Advanced Search Syntax**: Use tags like `mod:`, `key:`, `action:`, or `desc:` to filter keybinds with precision.
- **Visual Keyboard Map**: Interactive layout to visualize used and available keys for any modifier combination. Supports multiple physical layouts including ANSI, ISO, JIS, ABNT2, or Hungarian.
- **Category Filtering**: Filter binds by common categories like Workspace, Window, Media, or Custom scripts.
- **Conflict Detection**: Automatically identifies and highlights duplicate keybinds, resolving Hyprland variables (e.g., `$mainMod`) for accuracy.
- **Full Keybind Management**: Add, edit, and delete keybinds directly from the UI. Changes are written back to the correct configuration files.
- **Configuration Backup**: Create a timestamped backup of your configuration files with a single click or set the automatic backup behavior in the settings (it's set to true by default).
- **Conflict Resolution Wizard**: A guided tool to help resolve duplicate keybinds one by one.
- **Smart Autocomplete**: Suggests valid Hyprland dispatchers as you type.
- **Favorites**: Pin frequently used keybinds for quick access.
- **Settings Editor**: Configure UI, backup behavior, and appearance directly within the app.
- **Automatic Backups**: Automatically backup your configuration on every change, with optional retention limits.
- **Command-Line Interface**: Quickly search and print keybinds from the terminal.
- **Keybind Exporting**: Export your keybinds to a simple markdown file for easy sharing or documentation.

## Installation

### From AUR (Arch Linux)
```bash
yay -S hyprkcs-git
```

### From Nix
```bash
nix run github:kosa12/hyprKCS
```

### From Source
Ensure you have `rust`, `cargo`, and `gtk4` development headers installed.

**Using Make (Recommended):**
```bash
git clone --depth=1 https://github.com/kosa12/hyprKCS.git
cd hyprKCS
make
sudo make install
```

**Using Cargo directly:**
```bash
git clone --depth=1 https://github.com/kosa12/hyprKCS.git
cd hyprKCS
cargo build --release
# The binary will be at ./target/release/hyprKCS
```

## Configuration

You can customize the appearance and behavior of hyprKCS by creating a configuration file at `~/.config/hyprkcs/hyprkcs.conf`. If a value is invalid or omitted, a default will be used.

| Option | Description | Default |
| --- | --- | --- |
| `width` | Window width (in pixels) | `700` |
| `height` | Window height (in pixels) | `500` |
| `opacity` | Window background opacity (0.0 to 1.0) | `1.0` |
| `fontSize` | Global font size (e.g., `10pt`, `1rem`) | `0.9rem` |
| `borderSize` | Global border thickness | `1px` |
| `borderRadius` | Main window corner radius | `12px` |
| `showSubmaps` | Toggles visibility of the "Submap" column | `false` |
| `showArgs` | Toggles visibility of the "Arguments" column | `true` |
| `showFavorites` | Toggles visibility of the "Favorites" column and category | `true` |
| `alternatingRowColors` | Toggles striped rows for the list view | `true` |
| `defaultSort` | Initial sort column (`key`, `dispatcher`, `mods`, etc.) | `key` |
| `keyboardLayout` | Physical keyboard layout for the visualizer (`ANSI`, `ISO`, `JIS`, `ABNT2`, `HU`) | `ANSI` |
| `shadowSize` | CSS box-shadow property for the window (`none` to disable) | `0 4px 24px rgba(0,0,0,0.4)` |
| `monitorMargin` | Margin around the window (in pixels) | `12` |
| `rowPadding` | Vertical padding between list rows (in pixels) | `2` |
| `autoBackup` | Automatically backup config on save | `true` |
| `maxBackupsEnabled` | Enable limiting the number of backups | `false` |
| `maxBackupsCount` | Maximum number of backups to keep | `10` |
| `showDescription` | Toggles visibility of the "Description" column (parsed comments from config files) | `false` |
<details>
<summary>Example Configuration</summary>

```ini
# Window dimensions
width = 1000px
height = 800px

# Appearance
opacity = 0.95
fontSize = 10pt
borderSize = 2px
borderRadius = 10px
alternatingRowColors = true
shadowSize = 0 4px 24px rgba(0,0,0,0.4)

# UI Elements
showSubmaps = false
showArgs = true
showFavorites = true
defaultSort = mods
keyboardLayout = ANSI
showDescription = true

# Behavior
autoBackup = true
maxBackupsEnabled = true
maxBackupsCount = 20

# Spacing
monitorMargin = 20px
rowPadding = 5px
```
</details>

## Usage

### Graphical Interface

Launch `hyprKCS` from your application menu or terminal to open the main window.

**Keyboard Shortcuts**
| Key | Action |
| --- | --- |
| `/` | Focus the search bar |
| `Enter` | Edit the selected keybind |
| `Ctrl` + `f` | Focus the search bar |
| `Esc` | Clear search or close the window |

**Advanced Search Syntax**
The search bar supports specific tags to filter results:
- `mod:<value>` / `mods:<value>`: Filter by modifiers (e.g., `mod:super`).
- `key:<value>`: Filter by key (e.g., `key:return`).
- `action:<value>` / `disp:<value>`: Filter by dispatcher/action (e.g., `action:exec`).
- `arg:<value>`: Filter by arguments (e.g., `arg:volume`).
- `desc:<value>`: Filter by description (e.g., `desc:screenshot`).

*Example:* `mod:super action:exec firefox` finds all Super-bound execution commands for Firefox.

**Visual Keyboard Map**
Click the keyboard icon in the top toolbar to open an interactive keyboard layout.
- **Select Modifiers**: Toggle SUPER, SHIFT, CTRL, or ALT to see which keys are bound to those modifiers.
- **Color Coding**: Keys bound to actions are highlighted. Hover over them to see the exact dispatcher and arguments.
- **Find Free Keys**: Easily spot unhighlighted keys to find available shortcuts for your configuration.
- **Multiple Layouts**: Switch between ANSI, ISO, JIS, ABNT2, or Hungarian layouts in the Settings to match your physical hardware.

<p align="center">
    <img src="./assets/image_4.png" width="80%" />
</p>

### Command-Line Interface

hyprKCS also includes a CLI for quick lookups and scripting.

- **Print all keybinds:**
  ```bash
  hyprkcs --print
  # Short: hyprkcs -p
  ```
- **Search for a keybind:**
  ```bash
  hyprkcs --search "firefox"
  # Short: hyprkcs -s "firefox"
  ```
- **Advanced search via CLI:**
  ```bash
  hyprkcs --search "mod:super action:exec"
  ```
- **Use a custom config file:**
  ```bash
  hyprkcs --config ~/.config/hypr/custom.conf
  # Short: hyprkcs -c ~/.config/hypr/custom.conf
  ```

## Troubleshooting

### GPG Key Import Issues
If you encounter errors like `gpg: keyserver receive failed` when installing from the AUR, you may need to import the required PGP key manually.

Try importing from the Ubuntu keyserver:
```bash
gpg --keyserver keyserver.ubuntu.com --recv-keys D2059131FDE2EECC7C90A549F2CB939C8AA67892
```

Or from OpenPGP:
```bash
gpg --keyserver keys.openpgp.org --recv-keys D2059131FDE2EECC7C90A549F2CB939C8AA67892
```

## Contributing

Contributions are welcome. Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
