use hyprKCS::parser::Keybind;
use hyprKCS::ui::utils::detect_conflicts;
use std::path::PathBuf;
use std::sync::Arc;

fn create_kb(mods: &str, key: &str, disp: &str, args: &str, submap: Option<&str>) -> Keybind {
    Keybind {
        mods: Arc::from(mods),
        clean_mods: Arc::from(mods),
        flags: Arc::from(""),
        key: Arc::from(key),
        dispatcher: Arc::from(disp),
        args: Arc::from(args),
        description: None,
        submap: submap.map(Arc::from),
        line_number: 0,
        file_path: PathBuf::from("test.conf"),
    }
}

#[test]
fn test_detect_conflicts_simple() {
    let kbs = vec![
        create_kb("SUPER", "Q", "killactive", "", None),
        create_kb("SUPER", "Q", "exec", "kitty", None),
    ];

    let results = detect_conflicts(&kbs);
    assert!(results[0].is_some());
    assert!(results[1].is_some());
    assert!(results[0].as_ref().unwrap().contains("exec kitty"));
    assert!(results[1].as_ref().unwrap().contains("killactive"));
}

#[test]
fn test_detect_conflicts_normalization() {
    let kbs = vec![
        create_kb("SUPER SHIFT", "A", "disp1", "", None),
        create_kb("SHIFT+SUPER", "a", "disp2", "", None),
    ];

    let results = detect_conflicts(&kbs);
    assert!(results[0].is_some());
    assert!(results[1].is_some());
}

#[test]
fn test_detect_conflicts_submaps() {
    let kbs = vec![
        create_kb("SUPER", "X", "disp1", "", Some("sub1")),
        create_kb("SUPER", "X", "disp2", "", Some("sub1")),
        create_kb("SUPER", "X", "disp3", "", None), // Different submap (None)
    ];

    let results = detect_conflicts(&kbs);
    assert!(results[0].is_some());
    assert!(results[1].is_some());
    assert!(results[2].is_none());
}

#[test]
fn test_detect_conflicts_triple() {
    let kbs = vec![
        create_kb("SUPER", "Z", "d1", "", None),
        create_kb("SUPER", "Z", "d2", "", None),
        create_kb("SUPER", "Z", "d3", "", None),
    ];

    let results = detect_conflicts(&kbs);
    assert!(results[0].as_ref().unwrap().contains("d2"));
    assert!(results[0].as_ref().unwrap().contains("d3"));
}
