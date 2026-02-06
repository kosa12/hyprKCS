use hyprKCS::config::{constants, StyleConfig};
use hyprKCS::parser::get_config_path;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

// Global lock to serialize tests that modify environment variables
static TEST_LOCK: Mutex<()> = Mutex::new(());

// Helper to run code with a temporary XDG_CONFIG_HOME
fn with_temp_config<F>(test_name: &str, callback: F)
where
    F: FnOnce(&PathBuf),
{
    // Acquire lock to ensure serial execution
    let _guard = TEST_LOCK.lock().unwrap();

    // Invalidate cache since we are switching XDG_CONFIG_HOME
    StyleConfig::invalidate_cache();

    // Use a unique path for each test to avoid collisions
    let mut temp_config_dir = env::temp_dir();
    temp_config_dir.push("hyprkcs_test_config");
    temp_config_dir.push(test_name);

    if temp_config_dir.exists() {
        let _ = fs::remove_dir_all(&temp_config_dir);
    }
    fs::create_dir_all(&temp_config_dir).unwrap();

    let original_xdg = env::var_os("XDG_CONFIG_HOME");
    env::set_var("XDG_CONFIG_HOME", &temp_config_dir);

    unsafe {
        env::remove_var("HYPRKCS_CONFIG");
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        callback(&temp_config_dir);
    }));

    if let Some(val) = original_xdg {
        env::set_var("XDG_CONFIG_HOME", val);
    } else {
        unsafe {
            env::remove_var("XDG_CONFIG_HOME");
        }
    }
    unsafe {
        env::remove_var("HYPRKCS_CONFIG");
    }

    let _ = fs::remove_dir_all(&temp_config_dir);

    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}

#[test]
fn test_default_config_path() {
    with_temp_config("default", |temp_dir| {
        let expected = temp_dir.join("hypr").join("hyprland.conf");
        let result = get_config_path().unwrap();

        assert_eq!(result, expected, "Should default to standard config path");
    });
}

#[test]
fn test_cli_override() {
    with_temp_config("cli_override", |_| {
        let override_path = PathBuf::from("/tmp/custom/hyprland.conf");
        env::set_var("HYPRKCS_CONFIG", &override_path);

        let result = get_config_path().unwrap();
        assert_eq!(result, override_path, "CLI env var should take precedence");
    });
}

#[test]
fn test_cli_directory_override() {
    with_temp_config("cli_dir_override", |temp_dir| {
        let custom_dir = temp_dir.join("custom_conf");
        fs::create_dir_all(&custom_dir).unwrap();

        env::set_var("HYPRKCS_CONFIG", &custom_dir);

        let result = get_config_path().unwrap();
        let expected = custom_dir.join("hyprland.conf");

        assert_eq!(
            result, expected,
            "Directory path should assume hyprland.conf"
        );
    });
}

#[test]
fn test_alternative_config_path_setting() {
    with_temp_config("alt_setting", |temp_dir| {
        // 1. Create a "fake" alternative config directory
        let alt_dir = temp_dir.join("my_dotfiles");
        fs::create_dir_all(&alt_dir).unwrap();

        // 2. Create hyprkcs.conf specifying this path
        let hyprkcs_conf_dir = temp_dir.join(constants::HYPRKCS_DIR);
        fs::create_dir_all(&hyprkcs_conf_dir).unwrap();

        let hyprkcs_conf_path = hyprkcs_conf_dir.join(constants::HYPRKCS_CONF);
        let conf_content = format!("alternativeConfigPath = {}", alt_dir.to_string_lossy());
        fs::write(&hyprkcs_conf_path, conf_content).unwrap();

        // 3. Run get_config_path
        let result = get_config_path().unwrap();
        let expected = alt_dir.join("hyprland.conf");

        assert_eq!(
            result, expected,
            "Should use alternative config path from settings"
        );
    });
}

#[test]
fn test_priority_order() {
    with_temp_config("priority", |temp_dir| {
        // 1. Set up alternative path in settings
        let alt_dir = temp_dir.join("alt");
        fs::create_dir_all(&alt_dir).unwrap();

        let hyprkcs_conf_dir = temp_dir.join(constants::HYPRKCS_DIR);
        fs::create_dir_all(&hyprkcs_conf_dir).unwrap();
        let hyprkcs_conf_path = hyprkcs_conf_dir.join(constants::HYPRKCS_CONF);
        fs::write(
            &hyprkcs_conf_path,
            format!("alternativeConfigPath = {}", alt_dir.to_string_lossy()),
        )
        .unwrap();

        // 2. Set up CLI override
        let cli_path = temp_dir.join("cli").join("hyprland.conf");
        env::set_var("HYPRKCS_CONFIG", &cli_path);

        // 3. Expect CLI to win
        let result = get_config_path().unwrap();
        assert_eq!(result, cli_path, "CLI should override settings");
    });
}

#[test]
fn test_style_config_serialization() {
    with_temp_config("style_serialization", |_| {
        let mut config = StyleConfig::default();
        config.alternative_config_path = Some("/test/path".to_string());

        // Save
        config.save().unwrap();

        // Load
        let loaded = StyleConfig::load();
        assert_eq!(
            loaded.alternative_config_path,
            Some("/test/path".to_string())
        );

        // Clear
        let mut config2 = loaded;
        config2.alternative_config_path = None;
        config2.save().unwrap();

        let loaded2 = StyleConfig::load();
        assert!(loaded2.alternative_config_path.is_none());
    });
}
