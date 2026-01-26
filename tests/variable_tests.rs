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
            "hyprkcs_test_var_{}_{}.conf",
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
fn test_rename_variable_references_word_boundaries() {
    let _guard = lock_env();
    let content = r#"
        $mod = SUPER
        $mod_alt = ALT
        bind = $mod, Q, exec, kitty
        bind = $mod_alt, W, exec, firefox
        # This is $mod in a comment
        bind = CTRL, E, exec, echo $modding
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    // Rename $mod -> $mainMod
    // Note: rename_variable_references expects the clean name (no $)
    let _count = rename_variable_references("mod", "mainMod").expect("Rename failed");

    // Should have renamed in definition, one bind, and the comment.
    // Wait, the comment match: "# This is $mod in a comment" -> "$mod" is a match.
    // "$modding" should NOT be matched because of word boundary.

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("$mainMod = SUPER"));
    assert!(new_content.contains("bind = $mainMod, Q, exec, kitty"));
    assert!(new_content.contains("# This is $mainMod in a comment"));

    assert!(new_content.contains("$mod_alt = ALT"));
    assert!(new_content.contains("bind = $mod_alt, W, exec, firefox"));
    assert!(new_content.contains("echo $modding")); // Should remain unchanged
}

#[test]
fn test_count_variable_references() {
    let _guard = lock_env();
    let content = r#"
        $term = kitty
        bind = SUPER, Return, exec, $term
        bind = SUPER SHIFT, Return, exec, [float] $term
        $other = $term
        # $term in comment
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    // count_variable_references should exclude the definition "$term ="
    let count = count_variable_references("term").expect("Count failed");

    // Expected: 2 binds, 1 other var usage, 1 comment = 4
    assert_eq!(count, 4);
}

#[test]
fn test_inline_variable_references() {
    let _guard = lock_env();
    let content = r#"
        $term = kitty
        bind = SUPER, Return, exec, $term
        $other = $term
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    // Inline $term -> kitty
    inline_variable_references("term", "kitty").expect("Inline failed");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();

    // Definition should REMAINE (inline function specifically avoids definition)
    // The user's request was "replace it with the value" on delete.
    // The UI then calls delete_variable separately.
    assert!(new_content.contains("$term = kitty"));

    assert!(new_content.contains("bind = SUPER, Return, exec, kitty"));
    assert!(new_content.contains("$other = kitty"));
}

#[test]
fn test_refactor_hardcoded_references() {
    let _guard = lock_env();
    let content = r#"
        $term = kitty
        bind = SUPER, Return, exec, kitty
        bind = SUPER SHIFT, Return, exec, kitty-stable
        # kitty in comment (should not be refactored by refactor_hardcoded_references as it only targets bind lines)
        exec-once = kitty
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    // Refactor "kitty" -> "$term"
    let _count = refactor_hardcoded_references("kitty", "term").expect("Refactor failed");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();

    // Should refactor the bind
    assert!(new_content.contains("bind = SUPER, Return, exec, $term"));

    // Should NOT refactor "kitty-stable" (boundary)
    assert!(new_content.contains("bind = SUPER SHIFT, Return, exec, kitty-stable"));

    // Should NOT refactor "exec-once" (only targets "bind" lines in current implementation)
    assert!(new_content.contains("exec-once = kitty"));

    // Should NOT refactor comment
    assert!(new_content.contains("# kitty in comment"));
}

#[test]
fn test_variable_add_update_delete() {
    let _guard = lock_env();
    let content = r#"
        # Config
        $oldVar = value
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    // 1. Add
    add_variable(temp.path.clone(), "newVar", "newValue").expect("Add failed");
    let c = std::fs::read_to_string(&temp.path).unwrap();
    assert!(c.contains("$newVar = newValue"));

    // 2. Update
    // get_defined_variables to find line number
    let vars = get_defined_variables().expect("Get defined failed");
    let v = vars.iter().find(|v| v.name.as_ref() == "$oldVar").unwrap();
    update_variable(
        temp.path.clone(),
        v.line_number,
        "oldVarRenamed",
        "updatedValue",
    )
    .expect("Update failed");

    let c = std::fs::read_to_string(&temp.path).unwrap();
    assert!(c.contains("$oldVarRenamed = updatedValue"));
    assert!(!c.contains("$oldVar = value"));

    // 3. Delete
    let vars = get_defined_variables().expect("Get defined failed 2");
    let v = vars.iter().find(|v| v.name.as_ref() == "$newVar").unwrap();

    // Verify line content before deletion to ensure we aren't deleting shifted/wrong line
    let content_lines: Vec<String> = std::fs::read_to_string(&temp.path)
        .unwrap()
        .lines()
        .map(|s| s.to_string())
        .collect();
    assert!(
        content_lines[v.line_number].contains("$newVar"),
        "Line {} does not contain expected variable $newVar",
        v.line_number
    );

    delete_variable(temp.path.clone(), v.line_number).expect("Delete failed");

    let c = std::fs::read_to_string(&temp.path).unwrap();
    assert!(!c.contains("$newVar = newValue"));
}

#[test]
fn test_variable_recursive_definition_handling() {
    let _guard = lock_env();
    let content = r#"
        $color = red
        $bg = $color
        bind = SUPER, X, exec, echo $bg
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    // Verify parser resolves it
    let binds = parse_config().expect("Parse failed");
    assert_eq!(binds[0].args.as_ref(), "echo red");

    // Rename $color -> $primary
    rename_variable_references("color", "primary").expect("Rename failed");

    let c = std::fs::read_to_string(&temp.path).unwrap();
    assert!(c.contains("$primary = red"));
    assert!(c.contains("$bg = $primary"));
}

#[test]
fn test_variable_rename_cross_file() {
    let _guard = lock_env();

    // Create a sourced file
    let sourced_content = r#"
        bind = $mainMod, T, exec, kitty
    "#;
    let sourced_temp = TempFile::new(sourced_content);

    // Create main config that sources the other file
    let main_content = format!(
        "$mainMod = SUPER\nsource = {}",
        sourced_temp.path.to_string_lossy()
    );
    let main_temp = TempFile::new(&main_content);

    std::env::set_var("HYPRKCS_CONFIG", &main_temp.path);

    // Verify initial state
    let binds = parse_config().expect("Initial parse failed");
    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0].mods.as_ref(), "SUPER");

    // Rename $mainMod -> $superKey
    rename_variable_references("mainMod", "superKey").expect("Cross-file rename failed");

    // Check main file
    let main_c = std::fs::read_to_string(&main_temp.path).unwrap();
    assert!(main_c.contains("$superKey = SUPER"));

    // Check sourced file
    let sourced_c = std::fs::read_to_string(&sourced_temp.path).unwrap();
    assert!(sourced_c.contains("bind = $superKey, T, exec, kitty"));
}
