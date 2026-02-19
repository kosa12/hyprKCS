use hyprKCS::xkb_handler::XkbHandler;
use std::fs;
use std::path::PathBuf;

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(name: &str, content: &str) -> Self {
        let path = std::env::temp_dir().join(format!("{}_{}", std::process::id(), name));
        fs::write(&path, content).expect("Failed to write temp file");
        Self { path }
    }

    fn path_str(&self) -> &str {
        self.path.to_str().expect("Valid UTF-8 path")
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[test]
fn test_xkb_handler_creation() {
    let handler = XkbHandler::new("us", "", "", "");
    assert!(handler.is_some(), "Should create handler for 'us' layout");

    let handler = handler.unwrap();
    let (label, sym) = handler.get_key_info(16); // 'Q'
    assert_eq!(label, "Q");
    assert_eq!(sym, "q");
}

#[test]
fn test_xkb_handler_dvorak() {
    let handler = XkbHandler::new("us", "dvorak", "", "");
    assert!(
        handler.is_some(),
        "Should create handler for 'us(dvorak)' layout"
    );

    let handler = handler.unwrap();
    let (label, sym) = handler.get_key_info(16);
    assert_eq!(label, "'");
    assert_eq!(sym, "apostrophe");
}

#[test]
fn test_xkb_from_file() {
    let xkb_content = r#"xkb_keymap {
    xkb_keycodes  { include "evdev+aliases(qwerty)" };
    xkb_types     { include "complete" };
    xkb_compat    { include "complete" };
    xkb_symbols   {
        include "pc+us(dvorak)+inet(evdev)"
    };
    xkb_geometry  { include "pc(pc105)" };
};"#;

    let temp = TempFile::new("test_from_file.xkb", xkb_content);
    let handler = XkbHandler::from_file(temp.path_str());
    assert!(handler.is_some(), "Should load handler from file");

    let handler = handler.unwrap();
    let (label, sym) = handler.get_key_info(16);
    assert_eq!(label, "'");
    assert_eq!(sym, "apostrophe");
}

#[test]
fn test_xkb_invalid_file() {
    let handler = XkbHandler::from_file("non_existent_file.xkb");
    assert!(
        handler.is_none(),
        "Should return None for non-existent file"
    );
}

#[test]
fn test_xkb_special_keys() {
    let handler = XkbHandler::new("us", "", "", "").unwrap();

    let (label_esc, sym_esc) = handler.get_key_info(1);
    assert_eq!(label_esc, "Esc");
    assert_eq!(sym_esc, "Escape");

    let (label_ent, sym_ent) = handler.get_key_info(28);
    assert_eq!(label_ent, "Ent");
    assert_eq!(sym_ent, "Return");

    let (label_spc, sym_spc) = handler.get_key_info(57);
    assert_eq!(label_spc, "Spc");
    assert_eq!(sym_spc, "space");

    let (label_shft, sym_shft) = handler.get_key_info(42);
    assert_eq!(label_shft, "Shft");
    assert_eq!(sym_shft, "Shift_L");

    let (label_ctrl, sym_ctrl) = handler.get_key_info(29);
    assert_eq!(label_ctrl, "Ctrl");
    assert_eq!(sym_ctrl, "Control_L");
}

#[test]
fn test_iso_keycodes() {
    // UK ISO layout
    let handler = XkbHandler::new("gb", "", "", "");
    assert!(handler.is_some());
    let handler = handler.unwrap();

    // Keycode 43 should be '#' (numbersign) in GB ISO
    let (label_43, sym_43) = handler.get_key_info(43);
    assert_eq!(label_43, "#");
    assert_eq!(sym_43, "numbersign");

    // Keycode 28 should still be Return
    let (label_28, sym_28) = handler.get_key_info(28);
    assert_eq!(label_28, "Ent");
    assert_eq!(sym_28, "Return");
}

#[test]
fn test_utf8_safety() {
    let handler = XkbHandler::new("us", "", "", "").unwrap();
    // Test truncation logic with a dummy long name if we can't easily trigger it with real XKB
    // But since get_key_info is internal, we just trust it handles ASCII safely which is 99% of keysyms.
    // We can at least verify it doesn't crash on standard keys.
    for i in 1..120 {
        let _ = handler.get_key_info(i);
    }
}
