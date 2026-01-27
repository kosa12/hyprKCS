use hyprKCS::ui::utils::backup::*;
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

struct TempConfigDir {
    path: PathBuf,
}

impl TempConfigDir {
    fn new() -> Self {
        let mut path = std::env::temp_dir();
        let dirname = format!(
            "hyprkcs_test_config_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        path.push(dirname);
        fs::create_dir_all(&path).expect("Failed to create temp config dir");
        std::env::set_var("XDG_CONFIG_HOME", &path);
        Self { path }
    }

    fn hypr_dir(&self) -> PathBuf {
        self.path.join("hypr")
    }

    fn backup_dir(&self) -> PathBuf {
        self.hypr_dir().join("backups")
    }
}

impl Drop for TempConfigDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_backup_and_restore() {
    let _guard = lock_env();
    let temp = TempConfigDir::new();

    let hypr_dir = temp.hypr_dir();
    fs::create_dir_all(&hypr_dir).unwrap();

    let conf_path = hypr_dir.join("hyprland.conf");
    fs::write(&conf_path, "bind = SUPER, Q, exec, kitty").unwrap();

    let sub_dir = hypr_dir.join("configs");
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(sub_dir.join("other.conf"), "source = something").unwrap();

    let result = perform_backup(true).expect("Backup failed");
    assert!(result.contains("Backed up 2 files"));

    let backups = list_backups().expect("Failed to list backups");
    assert_eq!(backups.len(), 1);
    let backup_path = &backups[0];

    fs::write(&conf_path, "modified").unwrap();

    let restore_result = restore_backup(backup_path).expect("Restore failed");
    assert!(restore_result.contains("Restored 2 files successfully"));

    let restored_content = fs::read_to_string(&conf_path).unwrap();
    assert_eq!(restored_content, "bind = SUPER, Q, exec, kitty");
}

#[test]
fn test_prune_backups() {
    let _guard = lock_env();
    let temp = TempConfigDir::new();
    let backup_root = temp.backup_dir();
    fs::create_dir_all(&backup_root).unwrap();

    for i in 1..=5 {
        let dir = backup_root.join(format!("2026-01-26_12-00-0{}", i));
        fs::create_dir_all(dir).unwrap();
        let timestamp_dir = backup_root.join(format!("2026-01-26_12-00-0{}", i));
        fs::write(timestamp_dir.join("hyprland.conf"), "test").unwrap();
    }

    let hyprkcs_dir = temp.path.join("hyprkcs");
    fs::create_dir_all(&hyprkcs_dir).unwrap();
    fs::write(
        hyprkcs_dir.join("hyprkcs.conf"),
        "maxBackupsEnabled = true\nmaxBackupsCount = 3\nautoBackup = true",
    )
    .unwrap();

    fs::create_dir_all(temp.hypr_dir()).unwrap();
    fs::write(temp.hypr_dir().join("hyprland.conf"), "test").unwrap();

    perform_backup(false).expect("Backup failed");

    let backups = list_backups().expect("Failed to list backups");
    assert_eq!(backups.len(), 3);
}

#[test]
fn test_generate_diff() {
    let _guard = lock_env();
    let temp = TempConfigDir::new();
    let hypr_dir = temp.hypr_dir();
    fs::create_dir_all(&hypr_dir).unwrap();

    let conf_path = hypr_dir.join("hyprland.conf");
    fs::write(&conf_path, "line1\nline2\n").unwrap();

    perform_backup(true).unwrap();
    let backups = list_backups().unwrap();
    let backup_path = &backups[0];

    fs::write(&conf_path, "line1\nline2 modified\nline3\n").unwrap();

    let diff = generate_diff(backup_path).expect("Failed to generate diff");

    assert!(diff.contains("-line2 modified"));
    assert!(diff.contains("+line2"));
    assert!(diff.contains("-line3"));
}
