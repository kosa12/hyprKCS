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

struct TempConfig {
    root: PathBuf,
}

impl TempConfig {
    fn new() -> Self {
        let mut path = std::env::temp_dir();
        let dirname = format!(
            "hyprkcs_complex_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        path.push(dirname);
        std::fs::create_dir_all(&path).expect("Failed to create temp dir");
        Self { root: path }
    }

    fn write_file(&self, relative_path: &str, content: &str) -> PathBuf {
        let full_path = self.root.join(relative_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create parent dir");
        }
        let mut file = std::fs::File::create(&full_path).expect("Failed to create file");
        file.write_all(content.as_bytes()).expect("Failed to write");
        full_path
    }
}

impl Drop for TempConfig {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn test_nested_sources_relative_paths() {
    let _guard = lock_env();
    let config = TempConfig::new();

    let main_conf = "
        source = ./subdir/second.conf
        bind = SUPER, A, exec, main
    ";
    let second_conf = "
        source = ./third.conf
        bind = SUPER, B, exec, second
    ";
    let third_conf = "
        bind = SUPER, C, exec, third
    ";

    let main_path = config.write_file("hyprland.conf", main_conf);
    config.write_file("subdir/second.conf", second_conf);
    config.write_file("subdir/third.conf", third_conf);

    std::env::set_var("HYPRKCS_CONFIG", &main_path);

    let binds = parse_config().expect("Failed to parse nested sources");
    assert_eq!(binds.len(), 3);

    // Sort binds by key to ensure stable order for assertions
    let mut binds_sorted = binds.clone();
    binds_sorted.sort_by_key(|b| b.key.clone());

    assert_eq!(binds_sorted[0].key.as_ref(), "A");
    assert_eq!(binds_sorted[1].key.as_ref(), "B");
    assert_eq!(binds_sorted[2].key.as_ref(), "C");
}

#[test]
fn test_deep_variable_expansion() {
    let _guard = lock_env();
    let config = TempConfig::new();

    let content = "
        $color1 = red
        $color2 = blue
        $combined = $color1 and $color2
        $final_cmd = echo \"Colors: $combined\" 
        
        bind = SUPER, X, exec, $final_cmd
    ";

    let main_path = config.write_file("hyprland.conf", content);
    std::env::set_var("HYPRKCS_CONFIG", &main_path);

    let binds = parse_config().expect("Failed to parse deep variables");
    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0].args.as_ref(), "echo \"Colors: red and blue\"");
}

#[test]
fn test_submap_nesting_and_reset() {
    let _guard = lock_env();
    let config = TempConfig::new();

    let content = "
        bind = SUPER, R, submap, resize
        
        submap = resize
        binde = , right, resizeactive, 10 0
        binde = , left, resizeactive, -10 0
        bind = , escape, submap, reset
        submap = reset

        bind = SUPER, F, exec, firefox
    ";

    let main_path = config.write_file("hyprland.conf", content);
    std::env::set_var("HYPRKCS_CONFIG", &main_path);

    let binds = parse_config().expect("Failed to parse submaps");

    // 1. bind = SUPER, R, submap, resize (None)
    // 2. binde = , right, resizeactive, 10 0 (Some("resize"))
    // 3. binde = , left, resizeactive, -10 0 (Some("resize"))
    // 4. bind = , escape, submap, reset (Some("resize"))
    // 5. bind = SUPER, F, exec, firefox (None)

    assert_eq!(binds.len(), 5);
    assert_eq!(binds[0].submap, None);
    assert_eq!(binds[1].submap.as_deref(), Some("resize"));
    assert_eq!(binds[2].submap.as_deref(), Some("resize"));
    assert_eq!(binds[3].submap.as_deref(), Some("resize"));
    assert_eq!(binds[4].submap, None);
}

#[test]
fn test_comma_in_quotes_exec() {
    let _guard = lock_env();
    let config = TempConfig::new();

    // The parser should not split on commas inside quotes
    let content = "
        bind = SUPER, P, exec, notify-send \"Title, with comma\", \"Message, with comma\"
        bind = SUPER, S, exec, bash -c \"echo '1,2,3' | cut -d, -f1\"
    ";

    let main_path = config.write_file("hyprland.conf", content);
    std::env::set_var("HYPRKCS_CONFIG", &main_path);

    let binds = parse_config().expect("Failed to parse commas in quotes");
    assert_eq!(binds.len(), 2);

    assert_eq!(
        binds[0].args.as_ref(),
        "notify-send \"Title, with comma\", \"Message, with comma\""
    );
    assert_eq!(
        binds[1].args.as_ref(),
        "bash -c \"echo '1,2,3' | cut -d, -f1\""
    );
}

#[test]
fn test_variable_in_source_path() {
    let _guard = lock_env();
    let config = TempConfig::new();

    let main_conf = "
        $my_config = ./extra.conf
        source = $my_config
    ";
    let extra_conf = "
        bind = SUPER, E, exec, extra
    ";

    let main_path = config.write_file("hyprland.conf", main_conf);
    config.write_file("extra.conf", extra_conf);

    std::env::set_var("HYPRKCS_CONFIG", &main_path);

    let binds = parse_config().expect("Failed to parse source with variable");
    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0].args.as_ref(), "extra");
}

#[test]
fn test_env_var_expansion_in_path() {
    let _guard = lock_env();
    let config = TempConfig::new();

    // Create a file in a custom location and use $HOME (via ~ expansion)
    // Note: expand_path handles ~

    let extra_path = config.write_file("my_extra.conf", "bind = SUPER, Z, exec, zebra");

    // We can't easily mock HOME for dirs::home_dir() without potential side effects on other tests
    // But we can test absolute paths and relative paths which are expanded.

    let main_conf = format!("source = {}", extra_path.to_string_lossy());
    let main_path = config.write_file("hyprland.conf", &main_conf);

    std::env::set_var("HYPRKCS_CONFIG", &main_path);

    let binds = parse_config().expect("Failed to parse absolute source path");
    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0].args.as_ref(), "zebra");
}
