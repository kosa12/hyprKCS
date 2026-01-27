use hyprKCS::parser::*;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

// Reusing lock and tempfile helpers
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
            "hyprkcs_test_bindd_{}_{}.conf",
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
fn test_parse_bindd() {
    let _guard = lock_env();

    // bindd = MODS, KEY, DESCRIPTION, DISPATCHER, ARGS
    let content = r#"
        bindd = SUPER, Q, Launch Terminal, exec, kitty
        bindd = , XF86AudioMute, Mute Audio, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse bindd");
    assert_eq!(binds.len(), 2);

    let b1 = &binds[0];
    assert_eq!(b1.flags.as_ref(), "d");
    assert_eq!(b1.mods.as_ref(), "SUPER");
    assert_eq!(b1.key.as_ref(), "Q");
    assert_eq!(b1.description.as_deref(), Some("Launch Terminal"));
    assert_eq!(b1.dispatcher.as_ref(), "exec");
    assert_eq!(b1.args.as_ref(), "kitty");

    let b2 = &binds[1];
    assert_eq!(b2.flags.as_ref(), "d");
    assert_eq!(b2.description.as_deref(), Some("Mute Audio"));
    assert_eq!(b2.args.as_ref(), "wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle");
}

#[test]
fn test_add_bindd() {
    let _guard = lock_env();

    let content = "";
    let temp = TempFile::new(content);

    add_keybind(
        temp.path.clone(),
        "SUPER",
        "E",
        "exec",
        "dolphin",
        None,
        Some("Open File Manager".to_string()),
        "d",
    )
    .expect("Failed to add bindd");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("bindd = SUPER, E, Open File Manager, exec, dolphin"));
    // Ensure no redundant comment
    assert!(!new_content.contains("# Open File Manager"));
}

#[test]
fn test_update_to_bindd() {
    let _guard = lock_env();

    let content = "bind = SUPER, Q, exec, kitty";
    let temp = TempFile::new(content);

    update_line(
        temp.path.clone(),
        0,
        "SUPER",
        "Q",
        "exec",
        "kitty",
        Some("Launch Kitty".to_string()),
        Some("d"),
    )
    .expect("Failed to update to bindd");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("bindd = SUPER, Q, Launch Kitty, exec, kitty"));
}

#[test]
fn test_bindd_empty_args() {
    let _guard = lock_env();
    let content = "bindd = SUPER, K, MyDesc, killactive";
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse");
    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0].description.as_deref(), Some("MyDesc"));
    assert_eq!(binds[0].dispatcher.as_ref(), "killactive");
    assert_eq!(binds[0].args.as_ref(), "");
}
