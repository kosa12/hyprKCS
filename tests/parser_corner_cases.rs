use hyprKCS::parser::*;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    match ENV_LOCK.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(content: &str) -> Self {
        let mut path = std::env::temp_dir();
        let filename = format!(
            "hyprkcs_test_corner_{}_{}.conf",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        path.push(filename);
        let mut file = std::fs::File::create(&path).expect("Failed to create temp file");
        file.write_all(content.as_bytes())
            .expect("Failed to write temp content");
        Self { path }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[test]
fn test_quoted_args_complex() {
    let _guard = lock_env();
    let content = r###"#;
        bind = SUPER, M, exec, notify-send "Hello, World"

        # Case: Quoted args with escaped quotes
        bind = SUPER, N, exec, bash -c "echo \"This is complex\""

        # Case: Unbalanced quotes (should handle gracefully or parse weirdly but not crash)
        bind = SUPER, B, exec, echo "Unbalanced
    "###;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse quoted args");
    assert_eq!(binds.len(), 3);

    assert_eq!(binds[0].args.as_ref(), "notify-send \"Hello, World\"");
    assert_eq!(
        binds[1].args.as_ref(),
        "bash -c \"echo \\\"This is complex\\\"\""
    );
}

#[test]
fn test_args_with_commas_no_quotes() {
    let _guard = lock_env();
    let content = "bind = SUPER, P, exec, grim -g \"$(slurp)\" - | wl-copy";
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse comma args");
    assert_eq!(binds.len(), 1);

    let content_simple = "bind = SUPER, P, exec, notify-send \"Hello, World\", extra args";
    let mut file = std::fs::File::create(&temp.path).expect("Recreate");
    file.write_all(content_simple.as_bytes()).expect("Write");

    let binds = parse_config().expect("Parse simple");
    assert_eq!(
        binds[0].args.as_ref(),
        "notify-send \"Hello, World\", extra args"
    );
}

#[test]
fn test_variable_substitution_in_binds() {
    let _guard = lock_env();
    let content = r###"#;
        $mainMod = SUPER
        $term = kitty
        $browser = firefox

        bind = $mainMod, Return, exec, $term
        bind = $mainMod SHIFT, B, exec, $browser
    "###;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse vars");
    assert_eq!(binds.len(), 2);

    assert_eq!(binds[0].mods.as_ref(), "SUPER");
    assert_eq!(binds[0].args.as_ref(), "kitty");

    assert_eq!(binds[1].mods.as_ref(), "SUPER SHIFT");
    assert_eq!(binds[1].args.as_ref(), "firefox");
}

#[test]
fn test_variable_recursive() {
    let _guard = lock_env();
    let content = r###"#;
        $color = red
        $cmd = notify-send "Color is $color"
        bind = SUPER, C, exec, $cmd
    "###;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse recursive vars");
    assert_eq!(binds.len(), 1);

    assert_eq!(binds[0].args.as_ref(), "notify-send \"Color is red\"");
}

#[test]
fn test_flags_parsing() {
    let _guard = lock_env();
    let content = r###"#;
        bindl = , Switch, exec, swaylock
        bindr = SUPER, Super_L, exec, pkill -SIGUSR1 waybar
        binde = , XF86AudioRaiseVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+ 
    "###;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse flags");
    assert_eq!(binds.len(), 3);

    assert_eq!(binds[0].flags.as_ref(), "l");
    assert_eq!(binds[1].flags.as_ref(), "r");
    assert_eq!(binds[2].flags.as_ref(), "e");
}

#[test]
fn test_inline_comments() {
    let _guard = lock_env();
    let content = r###"#;
        bind = SUPER, Q, killactive # Close window
        bind = SUPER, E, exec, dolphin #Open File Manager
    "###;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse comments");

    assert_eq!(binds[0].description.as_deref(), Some("Close window"));
    assert_eq!(binds[1].description.as_deref(), Some("Open File Manager"));
    assert_eq!(binds[1].args.as_ref(), "dolphin");
}

#[test]
fn test_preceding_comments() {
    let _guard = lock_env();
    let content = r###"#;
        # Terminal
        bind = SUPER, T, exec, kitty

        #   Launch Browser
        bind = SUPER, B, exec, firefox
    "###;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse comments");

    assert_eq!(binds[0].description.as_deref(), Some("Terminal"));
    assert_eq!(binds[1].description.as_deref(), Some("Launch Browser"));
}

#[test]
fn test_circular_source() {
    let _guard = lock_env();

    let temp1_path =
        std::env::temp_dir().join(format!("hyprkcs_circ1_{}.conf", std::process::id()));
    let temp2_path =
        std::env::temp_dir().join(format!("hyprkcs_circ2_{}.conf", std::process::id()));

    // temp1 sources temp2, temp2 sources temp1
    std::fs::write(
        &temp1_path,
        format!(
            "source = {}\nbind = SUPER, 1, exec, a",
            temp2_path.to_string_lossy()
        ),
    )
    .unwrap();
    std::fs::write(
        &temp2_path,
        format!(
            "source = {}\nbind = SUPER, 2, exec, b",
            temp1_path.to_string_lossy()
        ),
    )
    .unwrap();

    std::env::set_var("HYPRKCS_CONFIG", &temp1_path);

    // This should not hang/stack overflow due to HashSet<PathBuf> guard
    let binds = parse_config().expect("Failed to handle circular source");

    assert_eq!(binds.len(), 2);

    let _ = std::fs::remove_file(temp1_path);
    let _ = std::fs::remove_file(temp2_path);
}

#[test]
fn test_source_with_variable() {
    let _guard = lock_env();

    let sourced_temp = TempFile::new("bind = SUPER, T, exec, terminal");
    let content = format!(
        "$configDir = {}\nsource = $configDir/{}",
        sourced_temp.path.parent().unwrap().to_string_lossy(),
        sourced_temp.path.file_name().unwrap().to_string_lossy()
    );
    let main_temp = TempFile::new(&content);

    std::env::set_var("HYPRKCS_CONFIG", &main_temp.path);
    let binds = parse_config().expect("Failed to parse source with variable");

    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0].args.as_ref(), "terminal");
}

#[test]
fn test_bind_no_modifiers() {
    let _guard = lock_env();
    let content = "bind = , XF86AudioMute, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle";
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse bind with no mods");
    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0].mods.as_ref(), "");
    assert_eq!(binds[0].key.as_ref(), "XF86AudioMute");
}
