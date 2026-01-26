use hyprKCS::ui::utils::execution::command_exists;
use hyprKCS::ui::utils::keybinds::normalize;
use hyprKCS::ui::utils::macro_builder::parse_macro;

#[test]
fn test_command_exists() {
    // ls is guaranteed to exist on unix
    assert!(command_exists("ls"));
    assert!(command_exists("ls -la"));
    assert!(command_exists("[float] ls"));

    // non-existent
    assert!(!command_exists(
        "this-command-definitely-does-not-exist-12345"
    ));
}

#[test]
fn test_normalize_keybinds() {
    let (mods, key) = normalize("SUPER SHIFT", "Q");
    assert_eq!(mods, "SHIFT SUPER");
    assert_eq!(key, "q");

    let (mods1, _) = normalize("SUPER CTRL", "A");
    let (mods2, _) = normalize("CTRL SUPER", "A");
    assert_eq!(mods1, mods2);
    assert_eq!(mods1, "CTRL SUPER");

    let (mods, _) = normalize("SUPER SUPER SHIFT", "A");
    assert_eq!(mods, "SHIFT SUPER");

    let (mods, _) = normalize("SUPER+ALT", "Return");
    assert_eq!(mods, "ALT SUPER");

    let (mods, key) = normalize("super shift", "RETURN");
    assert_eq!(mods, "SHIFT SUPER");
    assert_eq!(key, "return");

    let (mods, _) = normalize("SUPER + CTRL + ALT", "T");
    assert_eq!(mods, "ALT CTRL SUPER");

    let (mods, _) = normalize("   SUPER   SHIFT   ", "Q");
    assert_eq!(mods, "SHIFT SUPER");
}

#[test]
fn test_parse_macro_simple() {
    let dispatcher = "exec";
    let args = "bash -c \"hyprctl dispatch workspace 1; hyprctl dispatch fullscreen 1\"";

    let result = parse_macro(dispatcher, args).expect("Failed to parse valid macro");

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "workspace");
    assert_eq!(result[0].1, "1");
    assert_eq!(result[1].0, "fullscreen");
    assert_eq!(result[1].1, "1");
}

#[test]
fn test_parse_macro_invalid() {
    assert!(parse_macro("workspace", "1").is_none());
    assert!(parse_macro("exec", "kitty").is_none());
    assert!(parse_macro("exec", "bash -c \"echo hello\"").is_none());
}

#[test]
fn test_parse_macro_quoted_args() {
    let dispatcher = "exec";
    let args = "bash -c \"hyprctl dispatch notify-send \\\"Hello World\\\"\"";

    let result = parse_macro(dispatcher, args).expect("Failed to parse quoted macro");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, "notify-send");
    assert_eq!(result[0].1, "\"Hello World\"");
}

#[test]
fn test_parse_macro_multiple_commands_whitespace() {
    let args =
        "bash -c \"  hyprctl dispatch   workspace   2  ;    hyprctl dispatch   killactive   \"";
    let result = parse_macro("exec", args).expect("Failed to parse messy whitespace");

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "workspace");
    assert_eq!(result[0].1, "2");
    assert_eq!(result[1].0, "killactive");
    assert_eq!(result[1].1, "");
}