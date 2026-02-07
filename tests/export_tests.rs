use gtk4::gio;
use hyprKCS::keybind_object::KeybindObject;
use hyprKCS::parser::Keybind;
use hyprKCS::ui::utils::export::export_keybinds_to_markdown;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn test_export_markdown() {
    // We need to initialize GTK for this to work because it uses ListStore and KeybindObject
    if let Err(e) = gtk4::init() {
        eprintln!(
            "Failed to initialize GTK: {}. Skipping test in headless environment.",
            e
        );
        return;
    }

    let model = gio::ListStore::new::<KeybindObject>();

    let kb_data = Keybind {
        mods: Arc::from("SUPER"),
        clean_mods: Arc::from("SUPER"),
        flags: Arc::from(""),
        key: Arc::from("Q"),
        dispatcher: Arc::from("exec"),
        args: Arc::from("kitty"),
        description: Some(Arc::from("Terminal")),
        submap: None,
        line_number: 10,
        file_path: PathBuf::from("hyprland.conf"),
    };

    let obj = KeybindObject::new(
        kb_data.clone(),
        None,
        None,
        false,
        Arc::from("super"),
        Arc::from("super"),
        Arc::from("q"),
        Arc::from("exec"),
        Some(Arc::from("kitty")),
        Some(Arc::from("terminal")),
        Arc::from(""),
    );

    model.append(&obj);

    let mut temp_path = std::env::temp_dir();
    temp_path.push("test_export.md");

    export_keybinds_to_markdown(&model, &temp_path).expect("Export failed");

    let content = std::fs::read_to_string(&temp_path).expect("Read export failed");
    assert!(content.contains("| SUPER | Q | exec | kitty |  | Terminal |"));
    assert!(content.contains("# Hyprland Keybinds"));

    let _ = std::fs::remove_file(temp_path);
}
