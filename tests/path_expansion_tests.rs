use hyprKCS::parser::*;
use std::fs;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

// We need a lock because we are modifying environment variables
static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    match ENV_LOCK.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let mut path = std::env::temp_dir();
        let dirname = format!(
            "hyprkcs_test_path_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        path.push(dirname);
        fs::create_dir_all(&path).expect("Failed to create temp dir");
        invalidate_parser_cache();
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_get_config_path_tilde_expansion() {
    let _guard = lock_env();

    std::env::set_var("HYPRKCS_CONFIG", "~/test_config.conf");
    let path = get_config_path().expect("get_config_path failed");

    let path_str = path.to_string_lossy();
    assert!(
        !path_str.starts_with('~'),
        "Path should not start with ~: {}",
        path_str
    );
    assert!(path.is_absolute(), "Path should be absolute: {}", path_str);
}

#[test]
fn test_source_tilde_expansion() {
    let _guard = lock_env();
    let temp_dir = TempDir::new();

    let main_conf = temp_dir.path.join("hyprland.conf");
    let nested_dir = temp_dir.path.join("nested");
    fs::create_dir(&nested_dir).expect("Failed to create nested dir");
    let nested_conf = nested_dir.join("included.conf");

    fs::write(&main_conf, "source = ./nested/included.conf").expect("Failed to write main");
    fs::write(&nested_conf, "bind = SUPER, S, exec, source_works").expect("Failed to write nested");

    std::env::set_var("HYPRKCS_CONFIG", &temp_dir.path);

    let binds = parse_config().expect("Failed to parse config");
    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0].key.as_ref(), "S");
}

#[test]
fn test_variable_expansion_in_source() {
    let _guard = lock_env();
    let temp_dir = TempDir::new();

    let main_conf = temp_dir.path.join("hyprland.conf");
    let other_conf = temp_dir.path.join("other.conf");

    // Using $hypr var which is injected by hyprKCS to point to the config root
    fs::write(&main_conf, "source = $hypr/other.conf").expect("Failed to write main");
    fs::write(&other_conf, "bind = SUPER, V, exec, var_works").expect("Failed to write other");

    std::env::set_var("HYPRKCS_CONFIG", &temp_dir.path);

    let binds = parse_config().expect("Failed to parse config");
    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0].key.as_ref(), "V");
}
