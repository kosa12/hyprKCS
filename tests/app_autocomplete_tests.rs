use hyprKCS::ui::utils::apps::get_installed_apps;
use std::fs;
use std::path::PathBuf;

fn setup_test_env() -> (PathBuf, PathBuf) {
    let mut temp_dir = std::env::temp_dir();
    temp_dir.push(format!("hyprkcs_test_{}", std::process::id()));

    let applications_dir = temp_dir.join("applications");
    fs::create_dir_all(&applications_dir).expect("Failed to create temp dirs");

    (temp_dir, applications_dir)
}

fn cleanup_test_env(path: PathBuf) {
    let _ = fs::remove_dir_all(path);
}

#[test]
fn test_autocomplete_exec_cleaning() {
    let (base_dir, apps_dir) = setup_test_env();

    // 1. Firefox Case: Absolute path with argument placeholder
    let firefox_desktop = r#"
[Desktop Entry]
Name=Fake Firefox
Exec=/usr/lib/firefox/firefox %u
Type=Application
"#;
    fs::write(apps_dir.join("fake-firefox.desktop"), firefox_desktop).unwrap();

    // 2. Chrome Case: Absolute path only
    let chrome_desktop = r#"
[Desktop Entry]
Name=Fake Chrome
Exec=/opt/google/chrome/google-chrome
Type=Application
"#;
    fs::write(apps_dir.join("fake-chrome.desktop"), chrome_desktop).unwrap();

    // 3. Simple Command: Just binary name with arguments
    let simple_desktop = r#"
[Desktop Entry]
Name=Fake Simple
Exec=simple-cmd --flag
Type=Application
"#;
    fs::write(apps_dir.join("fake-simple.desktop"), simple_desktop).unwrap();

    // 4. Quoted Command (Edge case handling check)
    // The current logic splits by whitespace, so "my-app" might be kept as ""my-app"" if quoted?
    // Let's see what happens.
    let quoted_desktop = r#"
[Desktop Entry]
Name=Fake Quoted
Exec="quoted-app" %F
Type=Application
"#;
    fs::write(apps_dir.join("fake-quoted.desktop"), quoted_desktop).unwrap();

    // Set env var to point to our temp dir
    // We append the existing path if we want, but here we just want to ensure our dir is scanned.
    // The code scans ~/.local/share AND XDG_DATA_DIRS.
    // We set XDG_DATA_DIRS to our base_dir.
    // Note: get_installed_apps looks for {dir}/applications, so we point XDG_DATA_DIRS to base_dir.
    unsafe {
        std::env::set_var("XDG_DATA_DIRS", base_dir.to_str().unwrap());
    }

    let apps = get_installed_apps();

    // Filter for our fake apps to ignore system noise
    let fake_firefox = apps
        .iter()
        .find(|a| a.name == "Fake Firefox")
        .expect("Firefox not found");
    let fake_chrome = apps
        .iter()
        .find(|a| a.name == "Fake Chrome")
        .expect("Chrome not found");
    let fake_simple = apps
        .iter()
        .find(|a| a.name == "Fake Simple")
        .expect("Simple not found");
    // Quoted handling might be tricky with current simple split logic, let's observe
    let fake_quoted = apps.iter().find(|a| a.name == "Fake Quoted");

    // Assertions
    // /usr/lib/firefox/firefox -> firefox
    assert_eq!(fake_firefox.exec, "firefox", "Failed to clean firefox path");

    // /opt/google/chrome/google-chrome -> google-chrome
    assert_eq!(
        fake_chrome.exec, "google-chrome",
        "Failed to clean chrome path"
    );

    // simple-cmd --flag -> simple-cmd
    assert_eq!(
        fake_simple.exec, "simple-cmd",
        "Failed to clean simple command"
    );

    if let Some(q) = fake_quoted {
        // "quoted-app" -> "quoted-app" (quotes likely preserved by simple split)
        // or quoted-app if logic strips them? Current logic doesn't strip quotes explicitly.
        // It splits by whitespace.
        println!("Quoted exec resulted in: {}", q.exec);
    }

    // Cleanup
    cleanup_test_env(base_dir);
}
