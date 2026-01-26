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

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(content: &str) -> Self {
        let mut path = std::env::temp_dir();
        let filename = format!(
            "hyprkcs_test_bulk_{}_{}.conf",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        path.push(filename);
        let mut file = std::fs::File::create(&path).expect("Failed to create temp file");
        file.write_all(content.as_bytes())
            .expect("Failed to write temp content");
        Self { path }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[test]
fn test_bulk_replace_logic() {
    let _guard = lock_env();
    let content = r#"
        bind = SUPER, Q, exec, kitty
        bind = SUPER, W, exec, firefox
        bind = SUPER, E, exec, nautilus
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse config");
    assert_eq!(binds.len(), 3);

    let find_text = "SUPER";
    let replace_text = "ALT";

    for bind in binds.iter() {
        let current_mods = bind.mods.to_string();
        if current_mods.contains(find_text) {
            let new_mods = current_mods.replace(find_text, replace_text);

            update_line(
                bind.file_path.clone(),
                bind.line_number,
                &new_mods,
                &bind.key,
                &bind.dispatcher,
                &bind.args,
                None,
                None,
            )
            .expect("Failed to update line");
        }
    }

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("bind = ALT, Q, exec, kitty"));
    assert!(new_content.contains("bind = ALT, W, exec, firefox"));
    assert!(new_content.contains("bind = ALT, E, exec, nautilus"));
}

#[test]
fn test_bulk_replace_case_insensitive() {
    let _guard = lock_env();
    let content = r#"
        bind = Super, Q, exec, kitty
        bind = super, W, exec, firefox
        bind = SUPER, E, exec, nautilus
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse config");

    let find_text = "super";
    let replace_text = "ALT";
    let find_text_lower = find_text.to_ascii_lowercase();

    for bind in binds.iter() {
        let current_mods = bind.mods.to_string();
        let current_mods_lower = current_mods.to_ascii_lowercase();

        if current_mods_lower.contains(&find_text_lower) {
            let mut new_mods = String::new();
            let mut last_end = 0;
            for (start, _) in current_mods_lower.match_indices(&find_text_lower) {
                new_mods.push_str(&current_mods[last_end..start]);
                new_mods.push_str(replace_text);
                last_end = start + find_text_lower.len();
            }
            new_mods.push_str(&current_mods[last_end..]);

            update_line(
                bind.file_path.clone(),
                bind.line_number,
                &new_mods,
                &bind.key,
                &bind.dispatcher,
                &bind.args,
                None,
                None,
            )
            .expect("Failed to update line");
        }
    }

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("bind = ALT, Q, exec, kitty"));
    assert!(new_content.contains("bind = ALT, W, exec, firefox"));
    assert!(new_content.contains("bind = ALT, E, exec, nautilus"));
}

#[test]
fn test_bulk_replace_dispatcher() {
    let _guard = lock_env();
    let content = r#"
        bind = SUPER, 1, workspace, 1
        bind = SUPER, 2, workspace, 2
        bind = SUPER, Q, killactive,
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Failed to parse config");

    let find_text = "workspace";
    let replace_text = "movetoworkspace";

    for bind in binds.iter() {
        let current_disp = bind.dispatcher.to_string();
        if current_disp == find_text {
            let new_disp = replace_text;

            update_line(
                bind.file_path.clone(),
                bind.line_number,
                &bind.mods,
                &bind.key,
                &new_disp,
                &bind.args,
                None,
                None,
            )
            .expect("Failed to update line");
        }
    }

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("bind = SUPER, 1, movetoworkspace, 1"));
    assert!(new_content.contains("bind = SUPER, 2, movetoworkspace, 2"));
    assert!(new_content.contains("bind = SUPER, Q, killactive"));
}
