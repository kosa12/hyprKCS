use super::*;
use once_cell::sync::Lazy;
use std::io::Write;
use std::sync::Mutex;

static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

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
fn test_resolve_variables() {
    let mut vars = HashMap::new();
    vars.insert("$mainMod".to_string(), "SUPER".to_string());
    vars.insert("$browser".to_string(), "firefox".to_string());

    let sorted_keys = vec!["$mainMod".to_string(), "$browser".to_string()];

    assert_eq!(
        resolve_variables("$mainMod SHIFT", &vars, &sorted_keys),
        "SUPER SHIFT"
    );
    assert_eq!(
        resolve_variables("exec, $browser", &vars, &sorted_keys),
        "exec, firefox"
    );
    assert_eq!(
        resolve_variables("no vars here", &vars, &sorted_keys),
        "no vars here"
    );
}

#[test]
fn test_variable_precedence() {
    let mut vars = HashMap::new();
    vars.insert("$term".to_string(), "kitty".to_string());
    vars.insert("$terminal".to_string(), "alacritty".to_string());

    let sorted_keys = vec!["$terminal".to_string(), "$term".to_string()];

    assert_eq!(
        resolve_variables("$terminal", &vars, &sorted_keys),
        "alacritty"
    );
}

#[test]
fn test_parse_config_simple() {
    let _guard = ENV_LOCK.lock().unwrap();

    let content = r#"$
$mainMod = SUPER
bind = $mainMod, Q, exec, kitty
bind = CTRL, C, killactive,
"#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse config");
    assert_eq!(binds.len(), 2);

    let b1 = &binds[0];
    assert_eq!(b1.mods, "SUPER");
    assert_eq!(b1.key, "Q");
    assert_eq!(b1.dispatcher, "exec");
    assert_eq!(b1.args, "kitty");

    let b2 = &binds[1];
    assert_eq!(b2.mods, "CTRL");
    assert_eq!(b2.key, "C");
    assert_eq!(b2.dispatcher, "killactive");
    assert!(b2.args.is_empty());
}

#[test]
fn test_add_keybind() {
    let _guard = ENV_LOCK.lock().unwrap();

    let content = r#"$
$mainMod = SUPER
bind = $mainMod, Q, exec, kitty
"#;
    let temp = TempFile::new(content);

    add_keybind(
        temp.path.clone(),
        "SUPER SHIFT",
        "F",
        "fullscreen",
        "0",
        None,
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
    let _guard = ENV_LOCK.lock().unwrap();

    let content = r#"$
bind = SUPER, 1, workspace, 1
bind = SUPER, 2, workspace, 2
bind = SUPER, 3, workspace, 3
"#;
    let temp = TempFile::new(content);

    delete_keybind(temp.path.clone(), 2).expect("Failed to delete");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(!new_content.contains("workspace, 2"));
    assert!(new_content.contains("workspace, 1"));
    assert!(new_content.contains("workspace, 3"));
}

#[test]
fn test_source_inclusion() {
    let _guard = ENV_LOCK.lock().unwrap();

    let sourced_content = "bind = ALT, F4, killactive,";
    let sourced_file = TempFile::new(sourced_content);

    let main_content = format!("source = {}\n", sourced_file.path.to_str().unwrap());
    let main_file = TempFile::new(&main_content);

    std::env::set_var("HYPRKCS_CONFIG", &main_file.path);

    let binds = parse_config().expect("Failed to parse");
    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0].mods, "ALT");
}

#[test]
fn test_submaps_parsing() {
    let _guard = ENV_LOCK.lock().unwrap();
    let content = r#"
bind = SUPER, R, submap, resize
submap = resize
binde = , l, resizeactive, 10 0
bind = , escape, submap, reset
submap = reset
bind = SUPER, Return, exec, alacritty
"#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse submaps");

    // Should have:
    // 1. SUPER, R -> submap resize (Global)
    // 2. , l -> resizeactive (resize submap)
    // 3. , escape -> submap reset (resize submap)
    // 4. SUPER, Return -> exec alacritty (Global)

    assert_eq!(binds.len(), 4);

    assert_eq!(binds[0].submap, None);
    assert_eq!(binds[0].dispatcher, "submap");
    assert_eq!(binds[0].args, "resize");

    assert_eq!(binds[1].submap, Some("resize".to_string()));
    assert_eq!(binds[1].flags, "e"); // check 'binde' flag
    assert_eq!(binds[1].dispatcher, "resizeactive");

    assert_eq!(binds[2].submap, Some("resize".to_string()));

    assert_eq!(binds[3].submap, None);
    assert_eq!(binds[3].key, "Return");
}

#[test]
fn test_add_keybind_to_submap() {
    let _guard = ENV_LOCK.lock().unwrap();
    let content = r#"
submap = existing
bind = , k, killactive,
submap = reset
"#;
    let temp = TempFile::new(content);

    // Add to existing submap
    add_keybind(
        temp.path.clone(),
        "",
        "m",
        "movefocus",
        "l",
        Some("existing".to_string()),
    )
    .expect("Failed to add to existing submap");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("bind = , m, movefocus, l"));

    // Add to NEW submap
    add_keybind(
        temp.path.clone(),
        "",
        "q",
        "quit",
        "",
        Some("newmap".to_string()),
    )
    .expect("Failed to add to new submap");

    let new_content_2 = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content_2.contains("submap = newmap"));
    assert!(new_content_2.contains("bind = , q, quit"));
    assert!(new_content_2.contains("submap = reset"));
}

#[test]
fn test_comments_and_whitespace() {
    let _guard = ENV_LOCK.lock().unwrap();
    let content = r#"
# This is a comment
bind = SUPER, C, exec, code # Launch VS Code
    bind   =   ALT ,  Tab  ,   cyclenext,    # messy spacing
"#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse messy config");
    assert_eq!(binds.len(), 2);

    assert_eq!(binds[0].key, "C");
    assert_eq!(binds[0].args, "code");

    assert_eq!(binds[1].mods, "ALT");
    assert_eq!(binds[1].key, "Tab");
    assert_eq!(binds[1].dispatcher, "cyclenext");
}

#[test]
fn test_variable_chains() {
    let _guard = ENV_LOCK.lock().unwrap();
    let content = r#"
$term = alacritty
$myExec = exec, $term
bind = SUPER, Return, $myExec
"#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse variable chain");

    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0].dispatcher, "exec");
    assert_eq!(binds[0].args, "alacritty");
}

#[test]
fn test_malformed_lines() {
    let _guard = ENV_LOCK.lock().unwrap();
    let content = r#"
bind = SUPER, Q
bind =
random junk text
bind = SUPER, W, exec
"#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Should not crash on malformed lines");

    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0].key, "W");
    assert_eq!(binds[0].dispatcher, "exec");
}
