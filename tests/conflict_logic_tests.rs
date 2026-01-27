use hyprKCS::parser::Keybind;
use hyprKCS::ui::utils::keybinds::detect_conflicts;
use std::path::PathBuf;
use std::rc::Rc;

fn create_kb(mods: &str, key: &str, disp: &str, args: &str, submap: Option<&str>) -> Keybind {
    Keybind {
        mods: Rc::from(mods),
        clean_mods: Rc::from(mods),
        flags: Rc::from(""),
        key: Rc::from(key),
        dispatcher: Rc::from(disp),
        args: Rc::from(args),
        description: None,
        submap: submap.map(Rc::from),
        line_number: 0,
        file_path: PathBuf::from("test.conf"),
    }
}

#[test]
fn test_detect_conflicts_simple() {
    let kb1 = create_kb("SUPER", "Q", "exec", "kitty", None);
    let kb2 = create_kb("SUPER", "Q", "killactive", "", None);

    let kbs = vec![kb1, kb2];
    let results = detect_conflicts(&kbs);

    assert_eq!(results.len(), 2);
    assert!(results[0].is_some());
    assert!(results[1].is_some());

    assert!(results[0].as_ref().unwrap().contains("killactive"));
    assert!(results[1].as_ref().unwrap().contains("exec kitty"));
}

#[test]
fn test_detect_conflicts_normalization() {
    let kb1 = create_kb("SUPER SHIFT", "A", "exec", "a", None);
    let kb2 = create_kb("SHIFT SUPER", "a", "exec", "b", None);

    let kbs = vec![kb1, kb2];
    let results = detect_conflicts(&kbs);

    assert!(results[0].is_some());
    assert!(results[1].is_some());
}

#[test]
fn test_detect_conflicts_submaps() {
    let kb1 = create_kb("SUPER", "R", "submap", "resize", None);
    let kb2 = create_kb("", "escape", "submap", "reset", Some("resize"));
    let kb3 = create_kb("", "escape", "exec", "something", None);

    let kbs = vec![kb1, kb2, kb3];
    let results = detect_conflicts(&kbs);

    assert!(results[0].is_none());
    assert!(results[1].is_none());
    assert!(results[2].is_none());
}

#[test]
fn test_detect_conflicts_triple() {
    let kb1 = create_kb("SUPER", "1", "workspace", "1", None);
    let kb2 = create_kb("SUPER", "1", "workspace", "2", None);
    let kb3 = create_kb("SUPER", "1", "workspace", "3", None);

    let kbs = vec![kb1, kb2, kb3];
    let results = detect_conflicts(&kbs);

    assert!(results[0].as_ref().unwrap().contains("workspace 2"));
    assert!(results[0].as_ref().unwrap().contains("workspace 3"));
}
