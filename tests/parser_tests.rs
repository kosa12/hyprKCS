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
            "hyprkcs_test_{}_{}.conf",
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
fn test_parse_config_simple() {
    let _guard = lock_env();

    let content = "$mainMod = SUPER\nbind = $mainMod, Q, exec, kitty\nbind = CTRL, C, killactive,";
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse config");
    assert_eq!(binds.len(), 2);

    let b1 = &binds[0];
    assert_eq!(b1.mods.as_ref(), "SUPER");
    assert_eq!(b1.key.as_ref(), "Q");
    assert_eq!(b1.dispatcher.as_ref(), "exec");
    assert_eq!(b1.args.as_ref(), "kitty");

    let b2 = &binds[1];
    assert_eq!(b2.mods.as_ref(), "CTRL");
    assert_eq!(b2.key.as_ref(), "C");
    assert_eq!(b2.dispatcher.as_ref(), "killactive");
}

#[test]
fn test_add_keybind() {
    let _guard = lock_env();

    let content = "$mainMod = SUPER\nbind = $mainMod, Q, exec, kitty";
    let temp = TempFile::new(content);

    add_keybind(
        temp.path.clone(),
        "SUPER SHIFT",
        "F",
        "fullscreen",
        "0",
        None,
        None,
        "",
    )
    .expect("Failed to add keybind");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("bind = SUPER SHIFT, F, fullscreen, 0"));

    std::env::set_var("HYPRKCS_CONFIG", &temp.path);
    let binds = parse_config().unwrap();
    assert_eq!(binds.len(), 2);
}

#[test]
fn test_delete_keybind() {
    let _guard = lock_env();

    let content = "bind = SUPER, 1, workspace, 1\nbind = SUPER, 2, workspace, 2\nbind = SUPER, 3, workspace, 3";
    let temp = TempFile::new(content);

    // Indices are 0-based. workspace, 2 is on line index 1.
    delete_keybind(temp.path.clone(), 1).expect("Failed to delete");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(!new_content.contains("workspace, 2"));
    assert!(new_content.contains("workspace, 1"));
    assert!(new_content.contains("workspace, 3"));
}

#[test]
fn test_submaps_parsing() {
    let _guard = lock_env();
    let content = "bind = SUPER, R, submap, resize\nsubmap = resize\nbinde = , l, resizeactive, 10 0\nbind = , escape, submap, reset\nsubmap = reset\nbind = SUPER, Return, exec, alacritty";
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse submaps");

    assert_eq!(binds.len(), 4);
    assert_eq!(binds[0].dispatcher.as_ref(), "submap");
    assert_eq!(binds[1].submap.as_deref(), Some("resize"));
}

#[test]
fn test_update_keybind_description() {
    let _guard = lock_env();
    let content = "bind = SUPER, Q, exec, kitty # Old Description\n";
    let temp = TempFile::new(content);

    update_line(
        temp.path.clone(),
        0,
        "SUPER",
        "Q",
        "exec",
        "kitty",
        Some("New Description".to_string()),
        None,
    )
    .expect("Failed to update description");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("bind = SUPER, Q, exec, kitty # New Description"));
}

#[test]
fn test_parse_macro_keybind() {
    let _guard = lock_env();
    let macro_cmd = "bash -c \"hyprctl dispatch workspace 1; hyprctl dispatch fullscreen 1\"";
    let content = format!("bind = SUPER, M, exec, {}", macro_cmd);
    let temp = TempFile::new(&content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse macro config");
    assert_eq!(binds.len(), 1);

    let b = &binds[0];
    assert_eq!(b.mods.as_ref(), "SUPER");
    assert_eq!(b.key.as_ref(), "M");
    assert_eq!(b.dispatcher.as_ref(), "exec");
    assert_eq!(b.args.as_ref(), macro_cmd);
}

#[test]
fn test_parser_corner_cases() {
    let _guard = lock_env();
    let content = r#" 
        # Case 1: Extra whitespace
        bind   =   SUPER  ,  Q  ,  exec  ,  kitty
        
        # Case 2: No args
        bind = CTRL, C, killactive

        # Case 3: Args with multiple commas
        bind = SUPER, N, exec, notify-send "Hello, World"

        # Case 4: Flags and no space around equal
        bindl=,Switch,exec,swaylock

        # Case 5: Variable inside string? (Just checking basic parsing)
        bind = $mainMod, E, exec, echo "$mainMod"
    "#;

    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse corner cases");
    assert_eq!(binds.len(), 5);

    // Case 1
    assert_eq!(binds[0].mods.as_ref(), "SUPER");
    assert_eq!(binds[0].key.as_ref(), "Q");
    assert_eq!(binds[0].dispatcher.as_ref(), "exec");
    assert_eq!(binds[0].args.as_ref(), "kitty");

    // Case 2
    assert_eq!(binds[1].dispatcher.as_ref(), "killactive");
    assert_eq!(binds[1].args.as_ref(), "");

    // Case 3
    assert_eq!(binds[2].args.as_ref(), "notify-send \"Hello, World\"");

    // Case 4
    assert_eq!(binds[3].flags.as_ref(), "l");
    assert_eq!(binds[3].mods.as_ref(), ""); // Empty mods before first comma
    assert_eq!(binds[3].key.as_ref(), "Switch");

    // Case 5
    // Note: $mainMod is not defined in this file, so it won't be substituted.
    assert_eq!(binds[4].mods.as_ref(), "$mainMod");
}

#[test]
fn test_add_keybind_with_flags() {
    let _guard = lock_env();

    let content = "$mainMod = SUPER\n";
    let temp = TempFile::new(content);

    add_keybind(
        temp.path.clone(),
        "",
        "XF86AudioRaiseVolume",
        "exec",
        "wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+",
        None,
        None,
        "el",
    )
    .expect("Failed to add keybind");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    // Should be bindel = ...
    assert!(new_content.contains(
        "bindel = , XF86AudioRaiseVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+"
    ));

    std::env::set_var("HYPRKCS_CONFIG", &temp.path);
    let binds = parse_config().unwrap();
    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0].flags.as_ref(), "el");
}

#[test]
fn test_update_keybind_with_flags() {
    let _guard = lock_env();

    let content = "bind = SUPER, Q, exec, kitty";
    let temp = TempFile::new(content);

    // Update to bindl
    update_line(
        temp.path.clone(),
        0,
        "SUPER",
        "Q",
        "exec",
        "kitty",
        None,
        Some("l"),
    )
    .expect("Failed to update flags");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("bindl = SUPER, Q, exec, kitty"));
}

#[test]
fn test_mouse_bind_parsing() {
    let _guard = lock_env();
    let content = r#"
        bindm = SUPER, mouse:272, movewindow
        bind = , mouse:273, exec, rofi -show drun
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse mouse binds");
    assert_eq!(binds.len(), 2);

    let b1 = &binds[0];
    assert_eq!(b1.flags.as_ref(), "m");
    assert_eq!(b1.mods.as_ref(), "SUPER");
    assert_eq!(b1.key.as_ref(), "mouse:272");
    assert_eq!(b1.dispatcher.as_ref(), "movewindow");

    let b2 = &binds[1];
    assert_eq!(b2.flags.as_ref(), "");
    assert_eq!(b2.key.as_ref(), "mouse:273");
}

#[test]
fn test_mouse_scroll_parsing() {
    let _guard = lock_env();
    let content = r#"
        bind = SUPER, mouse_up, workspace, e+1
        bind = SUPER, mouse_down, workspace, e-1
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse scroll binds");
    assert_eq!(binds.len(), 2);

    assert_eq!(binds[0].key.as_ref(), "mouse_up");
    assert_eq!(binds[0].dispatcher.as_ref(), "workspace");
    assert_eq!(binds[0].args.as_ref(), "e+1");

    assert_eq!(binds[1].key.as_ref(), "mouse_down");
}

#[test]
fn test_variable_resolution_conflict_simulation() {
    let _guard = lock_env();
    let content = "$mainMod = SUPER\nbind = $mainMod, Q, exec, kitty";
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    // 1. Verify parse_config resolves variable
    let binds = parse_config().expect("Failed to parse");
    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0].mods.as_ref(), "SUPER");

    // 2. Verify get_variables returns the variable
    let variables = get_variables().expect("Failed to get variables");
    assert_eq!(variables.get("$mainMod"), Some(&"SUPER".to_string()));

    // 3. Simulate check_conflict logic
    // Input: mods="$mainMod", key="Q"
    let input_mods = "$mainMod";
    let input_key = "Q";

    // Resolve
    let resolved_mods = if input_mods.contains('$') {
        let mut result = input_mods.to_string();
        // Simple resolve simulation matching conflicts.rs logic
        for (key, val) in &variables {
            if result.contains(key) {
                result = result.replace(key, val);
            }
        }
        result
    } else {
        input_mods.to_string()
    };
    assert_eq!(resolved_mods, "SUPER");

    // Normalize
    // (Simulate normalize: SUPER -> SUPER, Q -> q)
    let norm_mods = resolved_mods.to_uppercase();
    let norm_key = input_key.to_lowercase();

    // Check against existing
    let kb = &binds[0];
    let kb_mods = kb.mods.to_uppercase();
    let kb_key = kb.key.to_lowercase();

    assert_eq!(norm_mods, kb_mods);
    assert_eq!(norm_key, kb_key);
}
