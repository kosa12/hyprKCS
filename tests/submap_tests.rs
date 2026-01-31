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
            "hyprkcs_submap_test_{}_{}.conf",
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
fn test_parse_submap_basic() {
    let _guard = lock_env();
    let content = r#"
        bind = SUPER, Q, exec, kitty
        submap = resize
        binde = , right, resizeactive, 10 0
        bind = , escape, submap, reset
        submap = reset
        bind = SUPER, E, exec, dolphin
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse config");
    assert_eq!(binds.len(), 4);

    // 1. Global
    assert_eq!(binds[0].submap, None);
    assert_eq!(binds[0].key.as_ref(), "Q");

    // 2. Resize Submap - Resize action
    assert_eq!(binds[1].submap.as_deref(), Some("resize"));
    assert_eq!(binds[1].key.as_ref(), "right");

    // 3. Resize Submap - Reset action
    assert_eq!(binds[2].submap.as_deref(), Some("resize"));
    assert_eq!(binds[2].key.as_ref(), "escape");

    // 4. Global (after reset)
    assert_eq!(binds[3].submap, None);
    assert_eq!(binds[3].key.as_ref(), "E");
}

#[test]
fn test_add_keybind_to_existing_submap() {
    let _guard = lock_env();
    let content = r#"
        submap = mysubmap
        bind = , a, exec, echo A
        submap = reset
    "#;
    let temp = TempFile::new(content);

    // Add bind to 'mysubmap'
    add_keybind(
        temp.path.clone(),
        "",
        "b",
        "exec",
        "echo B",
        Some("mysubmap".to_string()),
        None,
        "",
    )
    .expect("Failed to add keybind to submap");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    // Should be inside the submap block
    assert!(new_content.contains("bind = , b, exec, echo B"));

    // Verify structure by parsing
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);
    let binds = parse_config().unwrap();

    let added_bind = binds
        .iter()
        .find(|b| b.key.as_ref() == "b")
        .expect("Bind not found");
    assert_eq!(added_bind.submap.as_deref(), Some("mysubmap"));
}

#[test]
fn test_add_keybind_creates_new_submap() {
    let _guard = lock_env();
    let content = "bind = SUPER, Q, exec, kitty";
    let temp = TempFile::new(content);

    // Add bind to NEW submap 'brandnew'
    add_keybind(
        temp.path.clone(),
        "",
        "x",
        "exec",
        "echo X",
        Some("brandnew".to_string()),
        None,
        "",
    )
    .expect("Failed to add keybind to new submap");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("submap = brandnew"));
    assert!(new_content.contains("bind = , x, exec, echo X"));
    assert!(new_content.contains("submap = reset"));

    std::env::set_var("HYPRKCS_CONFIG", &temp.path);
    let binds = parse_config().unwrap();
    let added_bind = binds
        .iter()
        .find(|b| b.submap.as_deref() == Some("brandnew"))
        .unwrap();
    assert_eq!(added_bind.key.as_ref(), "x");
}

#[test]
fn test_submap_reset_logic() {
    let _guard = lock_env();
    // Test implicitly closed submap (end of file) vs explicitly closed
    let content = r#"
        submap = open_ended
        bind = , a, exec, echo A
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().unwrap();
    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0].submap.as_deref(), Some("open_ended"));
}
