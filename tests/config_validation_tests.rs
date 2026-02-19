use hyprKCS::config::StyleConfig;
use std::fs;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    match ENV_LOCK.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

struct TempConfig {
    root: PathBuf,
}

impl TempConfig {
    fn new() -> Self {
        let mut path = std::env::temp_dir();
        let dirname = format!("hyprkcs_config_val_{}", std::process::id());
        path.push(dirname);
        let _ = fs::remove_dir_all(&path); // Clean start
        fs::create_dir_all(&path).expect("Failed to create temp config dir");

        std::env::set_var("XDG_CONFIG_HOME", &path);
        StyleConfig::invalidate_cache();

        Self { root: path }
    }

    fn write_conf(&self, content: &str) {
        let dir = self.root.join("hyprkcs");
        let _ = fs::create_dir_all(&dir);
        fs::write(dir.join("hyprkcs.conf"), content).unwrap();
        StyleConfig::invalidate_cache();
    }

    fn write_xkb(&self, name: &str, content: &str) -> PathBuf {
        let path = self.root.join(name);
        fs::write(&path, content).unwrap();
        path
    }
}

impl Drop for TempConfig {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn test_custom_xkb_file_validation() {
    let _guard = lock_env();
    let temp = TempConfig::new();

    // 1. Test invalid file
    temp.write_conf(
        r#"
keyboardLayout = AUTO
customXkbFile = /non/existent/path.xkb
"#,
    );
    let config = StyleConfig::load();
    assert!(config.custom_xkb_file.is_none());
    assert!(config
        .errors
        .iter()
        .any(|e| e.contains("not found or is not a valid XKB keymap")));

    // 2. Test valid file
    let xkb_content = r#"xkb_keymap {
    xkb_keycodes  { include "evdev+aliases(qwerty)" };
    xkb_types     { include "complete" };
    xkb_compat    { include "complete" };
    xkb_symbols   { include "pc+us+inet(evdev)" };
    xkb_geometry  { include "pc(pc105)" };
};"#;
    let xkb_path = temp.write_xkb("valid.xkb", xkb_content);
    temp.write_conf(&format!(
        r#"
keyboardLayout = AUTO
customXkbFile = {}
"#,
        xkb_path.to_string_lossy()
    ));

    let config = StyleConfig::load();
    assert_eq!(
        config.custom_xkb_file,
        Some(xkb_path.to_string_lossy().to_string())
    );
}

#[test]
fn test_alternative_paths_validation() {
    let _guard = lock_env();
    let temp = TempConfig::new();

    // 1. Test invalid paths (non-existent)
    temp.write_conf(
        r#"
alternativeConfigPath = /non/existent/config
alternativeBackupPath = /non/existent/backup
"#,
    );
    let config = StyleConfig::load();
    assert!(config.alternative_config_path.is_none());
    assert!(config.alternative_backup_path.is_none());
    assert!(config
        .errors
        .iter()
        .any(|e| e.contains("Alternative config path")));
    assert!(config
        .errors
        .iter()
        .any(|e| e.contains("Alternative backup path")));

    // 2. Test paths that are files instead of directories
    let dummy_file = temp.root.join("not_a_dir");
    fs::write(&dummy_file, "").unwrap();
    temp.write_conf(&format!(
        r#"
alternativeConfigPath = {}
"#,
        dummy_file.to_string_lossy()
    ));

    let config = StyleConfig::load();
    assert!(config.alternative_config_path.is_none());
    assert!(config
        .errors
        .iter()
        .any(|e| e.contains("is not a directory")));

    // 3. Test valid directories
    let valid_cfg_dir = temp.root.join("valid_cfg");
    let valid_bak_dir = temp.root.join("valid_bak");
    fs::create_dir(&valid_cfg_dir).unwrap();
    fs::create_dir(&valid_bak_dir).unwrap();

    temp.write_conf(&format!(
        r#"
alternativeConfigPath = {}
alternativeBackupPath = {}
"#,
        valid_cfg_dir.to_string_lossy(),
        valid_bak_dir.to_string_lossy()
    ));

    let config = StyleConfig::load();
    assert_eq!(
        config.alternative_config_path,
        Some(valid_cfg_dir.to_string_lossy().to_string())
    );
    assert_eq!(
        config.alternative_backup_path,
        Some(valid_bak_dir.to_string_lossy().to_string())
    );
}
