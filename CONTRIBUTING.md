# Contributing to hyprKCS

First off, thank you for considering contributing to hyprKCS! It's people like you that make hyprKCS such a great tool.

## Ways to Contribute

- **Reporting Bugs:** If you find a bug, please [open an issue](https://github.com/kosa12/hyprKCS/issues). Include details about your environment (Hyprland version, distro) and steps to reproduce.
- **Suggesting Features:** Have an idea? [open an issue](https://github.com/kosa12/hyprKCS/issues) to discuss it.
- **Code Contributions:** Fix bugs, add features, or improve documentation via Pull Requests.

## Development Environment

To build and test hyprKCS, you will need:

- **Rust:** Version 1.75 or newer (Edition 2021).
- **GTK4 & Libadwaita:** Development headers (e.g., `libadwaita-devel` or `libadwaita-1-dev`).

### Manual Setup
Ensure you have the following installed on your system:
- `rustc` / `cargo`
- `gtk4`
- `libadwaita`
- `pkg-config`

## Contribution Workflow

1.  **Fork the repository** and clone it locally.
2.  **Create a branch** for your changes:
    ```bash
    git checkout -b feature/my-new-feature
    ```
3.  **Implement your changes.** Ensure your code follows the existing style and architectural patterns (GTK4/Libadwaita with Rust).
4.  **Verify your changes:**
    - Build: `cargo build`
    - Run: `cargo run`
    - Check formatting: `cargo fmt --all -- --check`
5.  **Commit your changes.** Keep commit messages clear and concise.
6.  **Push to your fork** and [submit a Pull Request](https://github.com/kosa12/hyprKCS/pulls).

## Coding Guidelines

- **Style:** Use `cargo fmt` to ensure consistent formatting.
- **Error Handling:** Use `anyhow` for application-level error handling.
- **UI:** Leverage Libadwaita widgets to maintain a native look and feel.
- **Safety:** Avoid `unsafe` blocks unless absolutely necessary for FFI.

## Project Structure

- `src/main.rs`: Entry point and CLI logic.
- `src/parser.rs`: Logic for parsing Hyprland configuration files.
- `src/keybind_object.rs`: Data structures for keybinds.
- `src/ui/`: All GTK/Libadwaita UI components.
    - `window.rs`: Main application window.
    - `views.rs`: List views and row definitions.
    - `style.rs`: Custom CSS/styling.

## License

By contributing to hyprKCS, you agree that your contributions will be licensed under the [MIT License](LICENSE).
