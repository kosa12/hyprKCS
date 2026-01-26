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

    let _count = rename_variable_references("mod", "mainMod").expect("Rename failed");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("$mainMod = SUPER"));
    assert!(new_content.contains("bind = $mainMod, Q, exec, kitty"));
    assert!(new_content.contains("# This is $mainMod in a comment"));
    assert!(new_content.contains("$mod_alt = ALT"));
    assert!(new_content.contains("bind = $mod_alt, W, exec, firefox"));
    assert!(new_content.contains("echo $modding"));
}

#[test]
fn test_rename_numeric_suffix_boundaries() {
    let _guard = lock_env();
    let content = r#"
        $ws1 = 1
        $ws10 = 10
        $ws100 = 100
        bind = SUPER, 1, workspace, $ws1
        bind = SUPER, 0, workspace, $ws10
        bind = SUPER SHIFT, 0, workspace, $ws100
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    rename_variable_references("ws1", "workspace1").expect("Rename failed");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("$workspace1 = 1"));
    assert!(new_content.contains("workspace, $workspace1"));
    assert!(new_content.contains("$ws10 = 10"));
    assert!(new_content.contains("$ws100 = 100"));
}

#[test]
fn test_consecutive_variables() {
    let _guard = lock_env();
    let content = r#"
        $a = foo
        $b = bar
        bind = SUPER, X, exec, echo $a$b
        bind = SUPER, Y, exec, echo $a/$b
        bind = SUPER, Z, exec, echo $a-$b
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    rename_variable_references("a", "first").expect("Rename failed");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("$first = foo"));
    assert!(new_content.contains("echo $first$b"));
    assert!(new_content.contains("echo $first/$b"));
    assert!(new_content.contains("echo $first-$b"));
}

#[test]
fn test_variable_at_line_boundaries() {
    let _guard = lock_env();
    let content = "$term = kitty\nbind = SUPER, Return, exec, $term\n$term";
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let count = count_variable_references("term").expect("Count failed");
    assert_eq!(count, 2); // bind usage + standalone $term at end

    inline_variable_references("term", "kitty").expect("Inline failed");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("exec, kitty"));
    assert!(new_content.ends_with("kitty\n") || new_content.ends_with("kitty"));
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

    let count = count_variable_references("term").expect("Count failed");
    assert_eq!(count, 4); // 2 binds + 1 other var usage + 1 comment
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

    inline_variable_references("term", "kitty").expect("Inline failed");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("$term = kitty")); // Definition remains
    assert!(new_content.contains("bind = SUPER, Return, exec, kitty"));
    assert!(new_content.contains("$other = kitty"));
}

#[test]
fn test_inline_with_special_chars() {
    let _guard = lock_env();
    let content = r#"
        $browser = firefox --new-window
        bind = SUPER, B, exec, $browser
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    inline_variable_references("browser", "firefox --new-window").expect("Inline failed");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("bind = SUPER, B, exec, firefox --new-window"));
}

#[test]
fn test_refactor_hardcoded_references() {
    let _guard = lock_env();
    let content = r#"
        $term = kitty
        bind = SUPER, Return, exec, kitty
        bind = SUPER SHIFT, Return, exec, kitty-stable
        # kitty in comment
        exec-once = kitty
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let _count = refactor_hardcoded_references("kitty", "term").expect("Refactor failed");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("bind = SUPER, Return, exec, $term"));
    assert!(new_content.contains("bind = SUPER SHIFT, Return, exec, kitty-stable"));
    assert!(new_content.contains("exec-once = kitty")); // Only targets bind lines
    assert!(new_content.contains("# kitty in comment"));
}

#[test]
fn test_refactor_path_like_value() {
    let _guard = lock_env();
    let content = r#"
        $script = ~/.config/hypr/script.sh
        bind = SUPER, S, exec, ~/.config/hypr/script.sh
        bind = SUPER, D, exec, ~/.config/hypr/script.sh --daemon
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    refactor_hardcoded_references("~/.config/hypr/script.sh", "script").expect("Refactor failed");

    let new_content = std::fs::read_to_string(&temp.path).unwrap();
    assert!(new_content.contains("bind = SUPER, S, exec, $script"));
    assert!(new_content.contains("bind = SUPER, D, exec, $script --daemon"));
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

    add_variable(temp.path.clone(), "newVar", "newValue").expect("Add failed");
    let c = std::fs::read_to_string(&temp.path).unwrap();
    assert!(c.contains("$newVar = newValue"));

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

    let vars = get_defined_variables().expect("Get defined failed 2");
    let v = vars.iter().find(|v| v.name.as_ref() == "$newVar").unwrap();

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
fn test_variable_empty_value() {
    let _guard = lock_env();
    let content = "$empty =\n$spaced =   \n";
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let vars = get_defined_variables().expect("Parse failed");
    assert_eq!(vars.len(), 2);
    assert!(vars
        .iter()
        .any(|v| v.name.as_ref() == "$empty" && v.value.as_ref() == ""));
    assert!(vars
        .iter()
        .any(|v| v.name.as_ref() == "$spaced" && v.value.as_ref() == ""));
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

    let binds = parse_config().expect("Parse failed");
    assert_eq!(binds[0].args.as_ref(), "echo red");

    rename_variable_references("color", "primary").expect("Rename failed");

    let c = std::fs::read_to_string(&temp.path).unwrap();
    assert!(c.contains("$primary = red"));
    assert!(c.contains("$bg = $primary"));
}

#[test]
fn test_variable_triple_chain() {
    let _guard = lock_env();
    let content = r#"
        $base = value
        $mid = $base
        $top = $mid
        bind = SUPER, X, exec, echo $top
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Parse failed");
    assert_eq!(binds[0].args.as_ref(), "echo value");
}

#[test]
fn test_variable_rename_cross_file() {
    let _guard = lock_env();

    let sourced_content = r#"
        bind = $mainMod, T, exec, kitty
    "#;
    let sourced_temp = TempFile::new(sourced_content);

    let main_content = format!(
        "$mainMod = SUPER\nsource = {}",
        sourced_temp.path.to_string_lossy()
    );
    let main_temp = TempFile::new(&main_content);

    std::env::set_var("HYPRKCS_CONFIG", &main_temp.path);

    let binds = parse_config().expect("Initial parse failed");
    assert_eq!(binds.len(), 1);
    assert_eq!(binds[0].mods.as_ref(), "SUPER");

    rename_variable_references("mainMod", "superKey").expect("Cross-file rename failed");

    let main_c = std::fs::read_to_string(&main_temp.path).unwrap();
    assert!(main_c.contains("$superKey = SUPER"));

    let sourced_c = std::fs::read_to_string(&sourced_temp.path).unwrap();
    assert!(sourced_c.contains("bind = $superKey, T, exec, kitty"));
}

#[test]
fn test_variable_whitespace_variations() {
    let _guard = lock_env();
    let content = "$a=1\n$b = 2\n$c  =  3\n$d=  4\n";
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let vars = get_defined_variables().expect("Parse failed");
    assert_eq!(vars.len(), 4);
    assert!(vars
        .iter()
        .any(|v| v.name.as_ref() == "$a" && v.value.as_ref() == "1"));
    assert!(vars
        .iter()
        .any(|v| v.name.as_ref() == "$b" && v.value.as_ref() == "2"));
    assert!(vars
        .iter()
        .any(|v| v.name.as_ref() == "$c" && v.value.as_ref() == "3"));
    assert!(vars
        .iter()
        .any(|v| v.name.as_ref() == "$d" && v.value.as_ref() == "4"));
}

#[test]
fn test_variable_with_inline_comment() {
    let _guard = lock_env();
    let content = "$var = value # this is a comment\n$var2 = val#ue\n";
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let vars = get_defined_variables().expect("Parse failed");
    let v1 = vars.iter().find(|v| v.name.as_ref() == "$var").unwrap();
    let v2 = vars.iter().find(|v| v.name.as_ref() == "$var2").unwrap();

    assert_eq!(v1.value.as_ref(), "value");
    assert_eq!(v2.value.as_ref(), "val");
}

#[test]
fn test_variable_with_dollar_in_value() {
    let _guard = lock_env();
    let content = r#"
        $price = $100
        bind = SUPER, P, exec, echo $price
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let binds = parse_config().expect("Parse failed");
    // It should resolve $price to $100.
    // NOTE: In the current implementation, if $1 is not defined, it remains $1.
    assert_eq!(binds[0].args.as_ref(), "echo $100");
}

#[test]
fn test_variable_naming_conventions() {
    let _guard = lock_env();
    let content = r#"
        $VAR_1 = underscore
        $var-2 = dash
        bind = SUPER, 1, exec, echo $VAR_1
        bind = SUPER, 2, exec, echo $var-2
    "#;
    let temp = TempFile::new(content);
    std::env::set_var("HYPRKCS_CONFIG", &temp.path);

    let variables = get_variables().expect("Failed to get variables");
    assert_eq!(variables.get("$VAR_1").unwrap(), "underscore");

    // Note: Hyprland usually only supports [a-zA-Z0-9_] for variable names.
    // Let's see if the parser handles dashes.
    let binds = parse_config().expect("Parse failed");
    assert_eq!(binds[0].args.as_ref(), "echo underscore");
}
