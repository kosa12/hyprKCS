use hyprKCS::config::StyleConfig;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

// Ensure tests don't run in parallel because they modify environment variables and shared caches
static TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[test]
fn test_style_config_mtime_invalidation() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    // 1. Setup
    let mut config_dir = std::env::temp_dir();
    config_dir.push(format!("xdg_cache_test_{}", std::process::id()));
    let hyprkcs_dir = config_dir.join("hyprkcs");
    fs::create_dir_all(&hyprkcs_dir).unwrap();
    let conf_path = hyprkcs_dir.join("hyprkcs.conf");

    std::env::set_var("XDG_CONFIG_HOME", &config_dir);
    StyleConfig::invalidate_cache();

    // 2. Write initial config
    fs::write(&conf_path, "width = 100px\n").unwrap();
    let cfg1 = StyleConfig::load();
    assert_eq!(cfg1.width, 100);

    // 3. Wait a bit to ensure mtime changes (filesystems often have 1s resolution)
    sleep(Duration::from_millis(1100));

    // 4. Modify file externally
    fs::write(&conf_path, "width = 200px\n").unwrap();

    // 5. Load again - should reflect changes due to mtime check
    let cfg2 = StyleConfig::load();
    assert_eq!(
        cfg2.width, 200,
        "Cache should have invalidated because mtime changed"
    );

    // 6. Cleanup
    let _ = fs::remove_dir_all(&config_dir);
}

#[test]
fn test_parser_cache_mtime_invalidation() {
    let _lock = TEST_LOCK.lock().unwrap();
    use hyprKCS::parser::{invalidate_parser_cache, parse_config};

    // 1. Setup
    let mut config_dir = std::env::temp_dir();
    config_dir.push(format!("parser_cache_test_{}", std::process::id()));
    let hypr_dir = config_dir.join("hypr");
    fs::create_dir_all(&hypr_dir).unwrap();
    let conf_path = hypr_dir.join("hyprland.conf");

    // FORCE the parser to use our temp file
    std::env::set_var("HYPRKCS_CONFIG", &conf_path);
    invalidate_parser_cache();

    // 2. Initial parse
    fs::write(&conf_path, "bind = SUPER, A, exec, cmd1").unwrap();
    let binds1 = parse_config().unwrap();
    assert_eq!(binds1.len(), 1);
    assert_eq!(binds1[0].key.as_ref(), "A");

    // 3. Wait for mtime resolution
    sleep(Duration::from_millis(1100));

    // 4. Modify externally
    fs::write(&conf_path, "bind = SUPER, B, exec, cmd2").unwrap();

    // 5. Parse again
    let binds2 = parse_config().unwrap();
    assert_eq!(binds2.len(), 1);
    assert_eq!(
        binds2[0].key.as_ref(),
        "B",
        "Parser cache should have invalidated because mtime changed"
    );

    // 6. Cleanup
    let _ = fs::remove_dir_all(&config_dir);
}

#[test]
fn test_reload_keybinds_generation_ordering() {
    let _lock = TEST_LOCK.lock().unwrap();
    use gtk4::gio;
    use gtk4::glib;
    use gtk4::prelude::*;
    use hyprKCS::keybind_object::KeybindObject;
    use hyprKCS::ui::utils::reload_keybinds;

    // 1. Setup GTK and model
    if let Err(e) = gtk4::init() {
        eprintln!("GTK init failed: {}. Skipping in headless env.", e);
        return;
    }
    let model = gio::ListStore::new::<KeybindObject>();

    // 2. Trigger multiple reloads
    let mut config_dir = std::env::temp_dir();
    config_dir.push(format!("reload_order_test_{}", std::process::id()));
    let hypr_dir = config_dir.join("hypr");
    fs::create_dir_all(&hypr_dir).unwrap();
    let conf_path = hypr_dir.join("hyprland.conf");
    std::env::set_var("HYPRKCS_CONFIG", conf_path.to_string_lossy().to_string());
    hyprKCS::parser::invalidate_parser_cache();

    // Initial state
    fs::write(&conf_path, "bind = SUPER, 1, exec, cmd1").unwrap();
    hyprKCS::parser::invalidate_parser_cache();
    reload_keybinds(&model);

    // Rapidly change and reload
    // We vary the size (trailing spaces) to ensure the cache-validation
    // doesn't think the file is unchanged if mtime doesn't tick.
    fs::write(&conf_path, "bind = SUPER, 2, exec, cmd2  ").unwrap();
    hyprKCS::parser::invalidate_parser_cache();
    reload_keybinds(&model);

    fs::write(&conf_path, "bind = SUPER, 3, exec, cmd3    ").unwrap();
    hyprKCS::parser::invalidate_parser_cache();
    reload_keybinds(&model);
    // 3. Wait for reloads to finish (with timeout)
    let mut success = false;
    for _ in 0..50 {
        // Run main loop iterations
        let context = glib::MainContext::default();
        while context.pending() {
            context.iteration(false);
        }

        if model.n_items() == 1 {
            if let Some(obj) = model.item(0).and_downcast::<KeybindObject>() {
                let key: String = obj.property("key");
                if key == "3" {
                    success = true;
                    break;
                }
            }
        }
        sleep(Duration::from_millis(100));
    }

    assert!(
        success,
        "Model should eventually reflect the LATEST reload (key '3')"
    );

    // Cleanup
    let _ = fs::remove_dir_all(&config_dir);
}

#[test]
fn test_parser_directory_sourcing_invalidation() {
    let _lock = TEST_LOCK.lock().unwrap();
    use hyprKCS::parser::{invalidate_parser_cache, parse_config};

    // 1. Setup
    let mut config_dir = std::env::temp_dir();
    config_dir.push(format!("parser_dir_test_{}", std::process::id()));
    let hypr_dir = config_dir.join("hypr");
    let sourced_dir = hypr_dir.join("sourced");
    fs::create_dir_all(&sourced_dir).unwrap();
    let main_conf = hypr_dir.join("hyprland.conf");

    // Main config sources the directory
    fs::write(
        &main_conf,
        format!("source = {}/", sourced_dir.to_string_lossy()),
    )
    .unwrap();

    // sourced/a.conf has one bind
    fs::write(sourced_dir.join("a.conf"), "bind = SUPER, A, exec, cmd1").unwrap();

    std::env::set_var("HYPRKCS_CONFIG", &main_conf);
    invalidate_parser_cache();

    // 2. Initial parse
    let binds1 = parse_config().unwrap();
    assert_eq!(binds1.len(), 1);
    assert_eq!(binds1[0].key.as_ref(), "A");

    // 3. Wait for mtime resolution
    sleep(Duration::from_millis(1100));

    // 4. Add a NEW file to the sourced directory
    fs::write(sourced_dir.join("b.conf"), "bind = SUPER, B, exec, cmd2").unwrap();

    // 5. Parse again - should pick up the new file because we track directory mtime
    let binds2 = parse_config().unwrap();
    assert_eq!(
        binds2.len(),
        2,
        "Parser should have detected the new file in sourced directory"
    );

    let mut keys: Vec<String> = binds2.iter().map(|b| b.key.to_string()).collect();
    keys.sort();
    assert_eq!(keys, vec!["A", "B"]);

    // 6. Cleanup
    let _ = fs::remove_dir_all(&config_dir);
}

#[test]
#[cfg(unix)]
fn test_parser_symlink_invalidation() {
    let _lock = TEST_LOCK.lock().unwrap();
    use hyprKCS::parser::{invalidate_parser_cache, parse_config};

    let mut config_dir = std::env::temp_dir();
    config_dir.push(format!("parser_symlink_test_{}", std::process::id()));
    let hypr_dir = config_dir.join("hypr");
    fs::create_dir_all(&hypr_dir).unwrap();

    let target_a = hypr_dir.join("target_a.conf");
    let target_b = hypr_dir.join("target_b.conf");
    let link = hypr_dir.join("link.conf");
    let main_conf = hypr_dir.join("hyprland.conf");

    fs::write(&target_a, "bind = SUPER, A, exec, cmdA").unwrap();
    // Ensure target_b is different in size to guarantee cache invalidation
    fs::write(
        &target_b,
        "bind = SUPER, B, exec, cmdB_extra_content_to_change_size",
    )
    .unwrap();
    symlink(&target_a, &link).unwrap();

    fs::write(&main_conf, format!("source = {}", link.to_string_lossy())).unwrap();

    std::env::set_var("HYPRKCS_CONFIG", &main_conf);
    invalidate_parser_cache();

    // Initial parse -> A
    let binds1 = parse_config().unwrap();
    assert_eq!(binds1.len(), 1);
    assert_eq!(binds1[0].key.as_ref(), "A");

    sleep(Duration::from_millis(1100));

    // Change symlink to point to B
    fs::remove_file(&link).unwrap();
    symlink(&target_b, &link).unwrap();

    // Parse -> B
    let binds2 = parse_config().unwrap();
    assert_eq!(binds2.len(), 1);
    assert_eq!(
        binds2[0].key.as_ref(),
        "B",
        "Parser should detect symlink target change"
    );

    let _ = fs::remove_dir_all(&config_dir);
}

#[test]
fn test_parser_deep_nesting_invalidation() {
    let _lock = TEST_LOCK.lock().unwrap();
    use hyprKCS::parser::{invalidate_parser_cache, parse_config};

    let mut config_dir = std::env::temp_dir();
    config_dir.push(format!("parser_deep_nest_test_{}", std::process::id()));
    let hypr_dir = config_dir.join("hypr");
    let deep_dir = hypr_dir.join("level1").join("level2");
    fs::create_dir_all(&deep_dir).unwrap();
    let main_conf = hypr_dir.join("hyprland.conf");
    let deep_file = deep_dir.join("deep.conf");

    fs::write(
        &main_conf,
        format!("source = {}/", hypr_dir.join("level1").to_string_lossy()),
    )
    .unwrap();
    fs::write(&deep_file, "bind = SUPER, A, exec, cmdA").unwrap();

    std::env::set_var("HYPRKCS_CONFIG", &main_conf);
    invalidate_parser_cache();

    let binds1 = parse_config().unwrap();
    assert_eq!(binds1.len(), 1);
    assert_eq!(binds1[0].key.as_ref(), "A");

    sleep(Duration::from_millis(1100));

    // Modify deeply nested file
    fs::write(&deep_file, "bind = SUPER, B, exec, cmdB").unwrap();

    let binds2 = parse_config().unwrap();
    assert_eq!(binds2.len(), 1);
    assert_eq!(
        binds2[0].key.as_ref(),
        "B",
        "Parser should detect changes in deeply nested sourced files"
    );

    let _ = fs::remove_dir_all(&config_dir);
}

#[test]
fn test_parser_delete_recreate_invalidation() {
    let _lock = TEST_LOCK.lock().unwrap();
    use hyprKCS::parser::{invalidate_parser_cache, parse_config};

    let mut config_dir = std::env::temp_dir();
    config_dir.push(format!("parser_del_recreate_test_{}", std::process::id()));
    let hypr_dir = config_dir.join("hypr");
    fs::create_dir_all(&hypr_dir).unwrap();
    let main_conf = hypr_dir.join("hyprland.conf");
    let sub_conf = hypr_dir.join("sub.conf");

    fs::write(
        &main_conf,
        format!("source = {}", sub_conf.to_string_lossy()),
    )
    .unwrap();
    fs::write(&sub_conf, "bind = SUPER, A, exec, cmdA").unwrap();

    std::env::set_var("HYPRKCS_CONFIG", &main_conf);
    invalidate_parser_cache();

    let binds1 = parse_config().unwrap();
    assert_eq!(binds1.len(), 1);

    sleep(Duration::from_millis(1100));

    // Delete and recreate file with different content
    fs::remove_file(&sub_conf).unwrap();
    fs::write(&sub_conf, "bind = SUPER, B, exec, cmdB").unwrap();

    let binds2 = parse_config().unwrap();
    assert_eq!(binds2.len(), 1);
    assert_eq!(
        binds2[0].key.as_ref(),
        "B",
        "Parser should handle delete-recreate cycle correctly"
    );

    let _ = fs::remove_dir_all(&config_dir);
}

#[test]
fn test_parser_broken_config_recovery() {
    let _lock = TEST_LOCK.lock().unwrap();
    use hyprKCS::parser::{invalidate_parser_cache, parse_config};

    let mut config_dir = std::env::temp_dir();
    config_dir.push(format!("parser_broken_test_{}", std::process::id()));
    let hypr_dir = config_dir.join("hypr");
    fs::create_dir_all(&hypr_dir).unwrap();
    let main_conf = hypr_dir.join("hyprland.conf");

    fs::write(&main_conf, "bind = SUPER, A, exec, cmdA").unwrap();
    std::env::set_var("HYPRKCS_CONFIG", &main_conf);
    invalidate_parser_cache();

    let binds1 = parse_config().unwrap();
    assert_eq!(binds1.len(), 1);

    sleep(Duration::from_millis(1100));

    // Write broken content (parser is resilient, but let's try something that might produce 0 binds or partial)
    // Writing strict garbage that isn't a bind
    fs::write(&main_conf, "THIS IS NOT A VALID CONFIG LINE").unwrap();

    let binds2 = parse_config().unwrap();
    assert_eq!(
        binds2.len(),
        0,
        "Parser should return empty/partial result for broken config"
    );

    sleep(Duration::from_millis(1100));

    // Fix it
    fs::write(&main_conf, "bind = SUPER, B, exec, cmdB").unwrap();

    let binds3 = parse_config().unwrap();
    assert_eq!(binds3.len(), 1);
    assert_eq!(
        binds3[0].key.as_ref(),
        "B",
        "Parser should recover after broken config is fixed"
    );

    let _ = fs::remove_dir_all(&config_dir);
}
