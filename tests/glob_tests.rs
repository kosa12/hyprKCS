use hyprKCS::parser::*;
use std::fs;
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

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let mut path = std::env::temp_dir();
        let dirname = format!(
            "hyprkcs_test_glob_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        path.push(dirname);
        fs::create_dir(&path).expect("Failed to create temp dir");
        Self { path }
    }

    fn create_file(&self, relative_path: &str, content: &str) -> PathBuf {
        let full_path = self.path.join(relative_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dirs");
        }
        let mut file = fs::File::create(&full_path).expect("Failed to create file");
        file.write_all(content.as_bytes())
            .expect("Failed to write content");
        full_path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_glob_source_parsing() {
    let _guard = lock_env();
    let temp = TempDir::new();

    // Main config
    let main_conf = temp.create_file("hyprland.conf", "source = ./conf.d/*.conf");

    // Sub configs
    temp.create_file("conf.d/a.conf", "bind = SUPER, A, exec, echo A");
    temp.create_file("conf.d/b.conf", "bind = SUPER, B, exec, echo B");
    temp.create_file("conf.d/ignored.txt", "bind = SUPER, I, exec, echo I"); // Should be ignored by *.conf

    std::env::set_var("HYPRKCS_CONFIG", &main_conf);

    let binds = parse_config().expect("Failed to parse config with globs");

    // Check results
    assert_eq!(binds.len(), 2, "Expected 2 binds, found {}", binds.len());

    let has_a = binds.iter().any(|b| b.key.as_ref() == "A");
    let has_b = binds.iter().any(|b| b.key.as_ref() == "B");
    let has_i = binds.iter().any(|b| b.key.as_ref() == "I");

    assert!(has_a, "Missing bind from a.conf");
    assert!(has_b, "Missing bind from b.conf");
    assert!(!has_i, "Included file that should have been ignored");
}

#[test]
fn test_glob_recursive_variables() {
    let _guard = lock_env();
    let temp = TempDir::new();

    // Test that variables are passed down to globbed files
    temp.create_file("hyprland.conf", "$mainMod = SUPER\nsource = ./subs/*.conf");
    temp.create_file("subs/test.conf", "bind = $mainMod, T, exec, term");

    std::env::set_var("HYPRKCS_CONFIG", temp.path.join("hyprland.conf"));

    let binds = parse_config().expect("Failed to parse");
    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0].mods.as_ref(), "SUPER");
}

#[test]
fn test_glob_no_matches() {
    let _guard = lock_env();
    let temp = TempDir::new();

    // source pattern that matches nothing should not fail
    temp.create_file(
        "hyprland.conf",
        "source = ./nowhere/*.conf\nbind = SUPER, K, exec, ok",
    );

    std::env::set_var("HYPRKCS_CONFIG", temp.path.join("hyprland.conf"));

    let binds = parse_config().expect("Failed to parse");
    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0].key.as_ref(), "K");
}

#[test]
fn test_glob_dot_segment() {
    let _guard = lock_env();
    let temp = TempDir::new();

    // Test path with dot segment: ./subdir/*.conf
    // This often happens when joining paths: /abs/path/./subdir/*.conf
    temp.create_file("hyprland.conf", "source = ./subdir/*.conf");
    temp.create_file("subdir/test.conf", "bind = SUPER, D, exec, dot");

    std::env::set_var("HYPRKCS_CONFIG", temp.path.join("hyprland.conf"));

    let binds = parse_config().expect("Failed to parse");
    assert_eq!(binds.len(), 1, "Failed to find bind via dot-segment path");
    assert_eq!(binds[0].key.as_ref(), "D");
}

#[test]
fn test_glob_typo_fallback() {
    let _guard = lock_env();
    let temp = TempDir::new();
    
    // Test fallback: source = ./typo/.conf (file .conf doesn't exist) -> treats as *.conf
    temp.create_file("hyprland.conf", "source = ./typo/.conf");
    temp.create_file("typo/real.conf", "bind = SUPER, F, exec, fallback");
    
    std::env::set_var("HYPRKCS_CONFIG", temp.path.join("hyprland.conf"));
    
    let binds = parse_config().expect("Failed to parse");
    assert_eq!(binds.len(), 1, "Failed to use fallback for .conf");
    assert_eq!(binds[0].key.as_ref(), "F");
}

#[test]
fn test_glob_dot_d_directory() {
    let _guard = lock_env();
    let temp = TempDir::new();
    
    // Test directory with .d extension: ./conf.d/*.conf
    temp.create_file("hyprland.conf", "source = ./conf.d/*.conf");
    temp.create_file("conf.d/test.conf", "bind = SUPER, D, exec, dot_d");
    
    std::env::set_var("HYPRKCS_CONFIG", temp.path.join("hyprland.conf"));
    
    let binds = parse_config().expect("Failed to parse");
    assert_eq!(binds.len(), 1, "Failed to parse from .d directory");
    assert_eq!(binds[0].args.as_ref(), "dot_d");
}

#[test]
fn test_directory_source() {
    let _guard = lock_env();
    let temp = TempDir::new();
    
    // Test direct directory sourcing: source = ./confdir (now recursive)
    temp.create_file("hyprland.conf", "source = ./confdir");
    temp.create_file("confdir/a.conf", "bind = SUPER, A, exec, echo A");
    temp.create_file("confdir/b.conf", "bind = SUPER, B, exec, echo B");
    // Nested file should now be picked up
    temp.create_file("confdir/nested/nested.conf", "bind = SUPER, N, exec, nested");
    
    std::env::set_var("HYPRKCS_CONFIG", temp.path.join("hyprland.conf"));
    
    let binds = parse_config().expect("Failed to parse directory source");
    assert_eq!(binds.len(), 3, "Expected 3 binds from recursive directory");
    
    let has_a = binds.iter().any(|b| b.key.as_ref() == "A");
    let has_b = binds.iter().any(|b| b.key.as_ref() == "B");
    let has_n = binds.iter().any(|b| b.key.as_ref() == "N");
    
    assert!(has_a);
    assert!(has_b);
    assert!(has_n, "Failed to recursively source nested subdirectory");
}

#[test]
fn test_hyprland_d_directory_structure() {
    let _guard = lock_env();
    let temp = TempDir::new();

    // Structure: hyprland.d/hyprland.conf
    //            hyprland.d/custom.d/regular/keybinds.conf
    let main_conf = temp.create_file("hyprland.d/hyprland.conf", "source = ./custom.d/regular/.conf");
    temp.create_file("hyprland.d/custom.d/regular/keybinds.conf", "bind = SUPER, K, exec, loaded");

    std::env::set_var("HYPRKCS_CONFIG", &main_conf);

    let binds = parse_config().expect("Failed to parse hyprland.d structure");
    assert_eq!(binds.len(), 1, "Failed to load keybinds from nested .d structure");
    assert_eq!(binds[0].key.as_ref(), "K");
}

#[test]
fn test_exact_user_reproduction() {
    let _guard = lock_env();
    let temp = TempDir::new();

    // Recreating user's structure
    // ~/.config/hypr (root of temp)
    //   custom.d/regular/keybinds.conf
    //   hyprland.conf sourcing ./custom.d/regular/.conf
    
    temp.create_file("hyprland.conf", "source = ./custom.d/regular/.conf");
    
    // The content from user's keybinds.conf (simplified)
    temp.create_file(
        "custom.d/regular/keybinds.conf", 
        "bind = SUPER, Q, exec, kitty\nbind = SUPER, W, killactive"
    );

    std::env::set_var("HYPRKCS_CONFIG", temp.path.join("hyprland.conf"));

    let binds = parse_config().expect("Failed to parse user reproduction config");
    
    // User expects these binds to be found
    let has_q = binds.iter().any(|b| b.key.as_ref() == "Q" && b.dispatcher.as_ref() == "exec" && b.args.as_ref() == "kitty");
    let has_w = binds.iter().any(|b| b.key.as_ref() == "W" && b.dispatcher.as_ref() == "killactive");

    assert!(has_q, "Failed to find SUPER+Q bind");
    assert!(has_w, "Failed to find SUPER+W bind");
}
