use hyprKCS::parser::input::*;
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
            "hyprkcs_test_input_{}_{}.conf",
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
fn test_load_input_simple() {
    let _guard = lock_env();
    let content = r#"
        input {
            kb_layout = us
            kb_options = grp:alt_shift_toggle
            follow_mouse = 1
            sensitivity = 0.5
        }
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let (input, _) = load_input_config().expect("Failed to load input config");

    assert_eq!(input.kb_layout, "us");
    assert_eq!(input.kb_options, "grp:alt_shift_toggle");
    assert_eq!(input.follow_mouse, 1);
    assert!((input.sensitivity - 0.5).abs() < f64::EPSILON);
}

#[test]
fn test_load_gestures_new_syntax() {
    let _guard = lock_env();
    let content = r#"
        gesture = 4, horizontal, workspace
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let (_, gestures) = load_input_config().expect("Failed to load gestures");

    assert!(gestures.workspace_swipe);
    assert_eq!(gestures.workspace_swipe_fingers, 4);
}

#[test]
fn test_save_input_round_trip() {
    let _guard = lock_env();
    let content = r#"
        input {
            kb_layout = us
        }
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    // Load
    let (mut input, mut gestures) = load_input_config().expect("Failed initial load");
    assert_eq!(input.kb_layout, "us");

    // Modify
    input.kb_layout = "br".to_string();
    input.sensitivity = -0.2;
    gestures.workspace_swipe = true;
    gestures.workspace_swipe_fingers = 3;

    // Save
    save_input_config(&input, &gestures).expect("Failed to save");

    // Reload
    let (input2, gestures2) = load_input_config().expect("Failed reload");

    assert_eq!(input2.kb_layout, "br");
    assert!((input2.sensitivity - -0.2).abs() < f64::EPSILON);
    assert!(gestures2.workspace_swipe);
    assert_eq!(gestures2.workspace_swipe_fingers, 3);
}

#[test]
fn test_legacy_cleanup() {
    let _guard = lock_env();
    // Input config with legacy fields and a legacy gestures block
    let content = r#"
        input {
            kb_layout = us
            workspace_swipe = true
            workspace_swipe_fingers = 3
        }

        gestures {
            workspace_swipe = true
            workspace_swipe_fingers = 3
        }
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let (input, gestures) = load_input_config().expect("Load legacy");

    save_input_config(&input, &gestures).expect("Save");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();

    assert!(!new_content.contains("gestures {"));
    assert!(!new_content.contains("workspace_swipe ="));
}
