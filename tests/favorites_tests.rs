use hyprKCS::config::favorites::*;
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
    path: PathBuf,
}

impl TempConfig {
    fn new() -> Self {
        let mut path = std::env::temp_dir();
        let dirname = format!("hyprkcs_fav_test_{}", std::process::id());
        path.push(dirname);
        fs::create_dir_all(&path).unwrap();
        std::env::set_var("XDG_CONFIG_HOME", &path);
        Self { path }
    }
}

impl Drop for TempConfig {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_favorites_round_trip() {
    let _guard = lock_env();
    let _temp = TempConfig::new();

    let favs = vec![
        FavoriteKeybind {
            mods: "SUPER".to_string(),
            key: "Q".to_string(),
            submap: "".to_string(),
            dispatcher: "exec".to_string(),
            args: "kitty".to_string(),
        },
        FavoriteKeybind {
            mods: "CTRL ALT".to_string(),
            key: "T".to_string(),
            submap: "resize".to_string(),
            dispatcher: "exec".to_string(),
            args: "notify-send test".to_string(),
        },
    ];

    save_favorites(&favs).expect("Save failed");

    let loaded = load_favorites();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].mods, "SUPER");
    assert_eq!(loaded[1].submap, "resize");
    assert_eq!(loaded[1].args, "notify-send test");
}

#[test]
fn test_toggle_favorite() {
    let mut favs = Vec::new();
    let item = FavoriteKeybind {
        mods: "SUPER".to_string(),
        key: "A".to_string(),
        submap: "".to_string(),
        dispatcher: "exec".to_string(),
        args: "cmd".to_string(),
    };

    let result = toggle_favorite(&mut favs, item.clone());
    assert!(result);
    assert_eq!(favs.len(), 1);

    let result = toggle_favorite(&mut favs, item);
    assert!(!result);
    assert_eq!(favs.len(), 0);
}

#[test]
fn test_is_favorite() {
    let favs = vec![FavoriteKeybind {
        mods: "SUPER".to_string(),
        key: "Q".to_string(),
        submap: "".to_string(),
        dispatcher: "exec".to_string(),
        args: "kitty".to_string(),
    }];

    assert!(is_favorite(&favs, "SUPER", "Q", "", "exec", "kitty"));
    assert!(!is_favorite(&favs, "SUPER", "W", "", "exec", "kitty"));
}
