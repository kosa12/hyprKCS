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
            "hyprkcs_submap_wizard_test_{}_{}.conf",
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
fn test_wizard_smart_placement_custom_default() {
    let _guard = lock_env();

    // Initial content mimicking Caelestia: stuck in global submap
    let content = r#" 
submap = global
bind = SUPER, Q, exec, kitty
submap = reset
"#;
    let temp = TempFile::new(content);

    // Simulate Wizard Logic for creating submap 'resize'
    let submap_name = "resize";
    let default_submap = "global"; // User's custom default

    // 1. Create Submap Block with custom exit target
    create_submap_block(
        temp.path.clone(),
        submap_name,
        Some("escape"), // reset key
        default_submap, // exit target -> 'global'
    )
    .expect("Failed to create submap block");

    // 2. Add Entry Bind inside the 'global' submap
    add_keybind(
        temp.path.clone(),
        "SUPER",
        "R",
        "submap",
        submap_name,
        Some(default_submap.to_string()), // Parent: global
        None,
        "",
    )
    .expect("Failed to add entry bind");

    // Verify Content
    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    println!("Generated Config:\n{}", new_content);

    // Check Exit Bind: should return to 'global'
    assert!(new_content.contains("bind = , escape, submap, global"));

    // Check Entry Bind: should be INSIDE 'global' submap
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);
    let binds = parse_config().unwrap();

    // Verify Entry Bind
    let entry_bind = binds
        .iter()
        .find(|b| b.key.as_ref() == "R")
        .expect("Entry bind not found");
    assert_eq!(entry_bind.submap.as_deref(), Some("global"));
    assert_eq!(entry_bind.dispatcher.as_ref(), "submap");
    assert_eq!(entry_bind.args.as_ref(), "resize");

    // Verify Exit Bind (in 'resize' submap)
    let exit_bind = binds
        .iter()
        .find(|b| b.key.as_ref() == "escape")
        .expect("Exit bind not found");
    assert_eq!(exit_bind.submap.as_deref(), Some("resize"));
    assert_eq!(exit_bind.dispatcher.as_ref(), "submap");
    assert_eq!(exit_bind.args.as_ref(), "global"); // Crucial check!
}

#[test]
fn test_wizard_standard_behavior() {
    let _guard = lock_env();
    let content = "bind = SUPER, Q, exec, kitty";
    let temp = TempFile::new(content);

    // Simulate Wizard Logic for creating submap 'resize'
    let submap_name = "resize";
    let default_submap = "reset"; // Standard behavior

    create_submap_block(
        temp.path.clone(),
        submap_name,
        Some("escape"),
        default_submap,
    )
    .expect("Failed to create submap block");

    add_keybind(
        temp.path.clone(),
        "SUPER",
        "R",
        "submap",
        submap_name,
        None, // Parent: root (None)
        None,
        "",
    )
    .expect("Failed to add entry bind");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();

    // Check Exit Bind: should return to 'reset'
    assert!(new_content.contains("bind = , escape, submap, reset"));

    std::env::set_var("HYPRKCS_CONFIG", &temp.path);
    let binds = parse_config().unwrap();

    let entry_bind = binds.iter().find(|b| b.key.as_ref() == "R").unwrap();
    assert_eq!(entry_bind.submap, None); // Root scope

    let exit_bind = binds.iter().find(|b| b.key.as_ref() == "escape").unwrap();
    assert_eq!(exit_bind.submap.as_deref(), Some("resize"));
    assert_eq!(exit_bind.args.as_ref(), "reset");
}
