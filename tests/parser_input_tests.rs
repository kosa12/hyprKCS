use hyprKCS::parser::input::*;
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
fn test_input_block_detection() {
    let _guard = lock_env();

    let content = r#"
        # Valid input block
        input {
            kb_layout = us
            follow_mouse = 1
        }

        # Misleading block (should be ignored)
        input_device {
            kb_layout = de
            follow_mouse = 0
        }

        # Another misleading block
        input-config {
            kb_layout = fr
        }
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let (input, _) = load_input_config().expect("Failed to load input config");

    // Should match the valid "input" block
    assert_eq!(input.kb_layout, "us");
    assert_eq!(input.follow_mouse, 1);
}

#[test]
fn test_input_save_preserves_misleading_blocks() {
    let _guard = lock_env();

    let content = r#"
input {
    kb_layout = us
}

input_device {
    kb_layout = de
}
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let mut input = InputConfig::default();
    input.kb_layout = "gb".to_string();
    let gestures = GesturesConfig::default();

    save_input_config(&input, &gestures).expect("Failed to save");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();

    // Valid block should be updated
    assert!(new_content.contains("kb_layout = gb"));
    // Misleading block should be preserved untouched
    assert!(new_content.contains("input_device {"));
    assert!(new_content.contains("kb_layout = de"));
}
