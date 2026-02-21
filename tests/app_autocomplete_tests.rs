use hyprKCS::ui::utils::apps::get_installed_apps;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

static TEST_LOCK: Mutex<()> = Mutex::new(());

fn with_temp_xdg_data<F>(callback: F)
where
    F: FnOnce(&PathBuf, &PathBuf) + std::panic::UnwindSafe,
{
    let _guard = TEST_LOCK.lock().unwrap();

    let mut temp_dir = env::temp_dir();
    temp_dir.push(format!("hyprkcs_test_apps_{}", std::process::id()));
    let apps_dir = temp_dir.join("applications");

    if temp_dir.exists() {
        let _ = fs::remove_dir_all(&temp_dir);
    }
    fs::create_dir_all(&apps_dir).expect("Failed to create temp dirs");

    let original_xdg = env::var_os("XDG_DATA_DIRS");

    // We set XDG_DATA_DIRS to our temp_dir.
    // get_installed_apps looks for {dir}/applications, so pointing to temp_dir is correct.
    unsafe {
        env::set_var("XDG_DATA_DIRS", &temp_dir);
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        callback(&temp_dir, &apps_dir);
    }));

    // Restore environment
    if let Some(val) = original_xdg {
        unsafe {
            env::set_var("XDG_DATA_DIRS", val);
        }
    } else {
        unsafe {
            env::remove_var("XDG_DATA_DIRS");
        }
    }

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);

    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}

#[test]
fn test_autocomplete_exec_cleaning() {
    with_temp_xdg_data(|_base_dir, apps_dir| {
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
        let quoted_desktop = r#"
[Desktop Entry]
Name=Fake Quoted
Exec="quoted-app" %F
Type=Application
"#;
        fs::write(apps_dir.join("fake-quoted.desktop"), quoted_desktop).unwrap();

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
            println!("Quoted exec resulted in: {}", q.exec);
        }
    });
}
