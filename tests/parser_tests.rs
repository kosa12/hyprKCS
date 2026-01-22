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
    )
    .expect("Failed to update description");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("bind = SUPER, Q, exec, kitty # New Description"));
}
