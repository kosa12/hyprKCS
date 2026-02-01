use crate::parser;
use crate::ui::utils::{create_page_header, create_suggested_button, reload_keybinds};
use gtk::{gio, prelude::*};
use gtk4 as gtk;
use libadwaita as adw;

pub fn create_add_submap_wizard(
    stack: &gtk::Stack,
    model: &gio::ListStore,
    toast_overlay: &adw::ToastOverlay,
) -> gtk::Widget {
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .vexpand(true)
        .build();

    let stack_c = stack.clone();
    let header = create_page_header(
        "Add New Submap",
        Some("Create a new Hyprland mode (submap)"),
        "Back",
        move || {
            stack_c.set_visible_child_name("settings");
        },
    );
    container.append(&header);

    // --- FORM ---
    let form_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .css_classes(["card"])
        .margin_start(24)
        .margin_end(24)
        .margin_top(12)
        .build();

    let grid = gtk::Grid::builder()
        .column_spacing(12)
        .row_spacing(12)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    let lbl_name = gtk::Label::builder()
        .label("Submap Name:")
        .halign(gtk::Align::Start)
        .build();
    let entry_name = gtk::Entry::builder()
        .placeholder_text("e.g. resize")
        .hexpand(true)
        .build();

    let lbl_enter_mods = gtk::Label::builder()
        .label("Enter Modifiers:")
        .halign(gtk::Align::Start)
        .build();
    let entry_enter_mods = gtk::Entry::builder()
        .placeholder_text("e.g. SUPER")
        .hexpand(true)
        .build();

    let lbl_enter_key = gtk::Label::builder()
        .label("Enter Key:")
        .halign(gtk::Align::Start)
        .build();
    let entry_enter_key = gtk::Entry::builder()
        .placeholder_text("e.g. R")
        .hexpand(true)
        .build();

    let lbl_reset = gtk::Label::builder()
        .label("Reset Key:")
        .halign(gtk::Align::Start)
        .build();
    let entry_reset = gtk::Entry::builder()
        .placeholder_text("e.g. escape")
        .hexpand(true)
        .build();

    grid.attach(&lbl_name, 0, 0, 1, 1);
    grid.attach(&entry_name, 1, 0, 1, 1);

    grid.attach(&lbl_enter_mods, 0, 1, 1, 1);
    grid.attach(&entry_enter_mods, 1, 1, 1, 1);

    grid.attach(&lbl_enter_key, 0, 2, 1, 1);
    grid.attach(&entry_enter_key, 1, 2, 1, 1);

    grid.attach(&lbl_reset, 0, 3, 1, 1);
    grid.attach(&entry_reset, 1, 3, 1, 1);

    form_box.append(&grid);
    container.append(&form_box);

    let info_label = gtk::Label::builder()
        .label("Adding a submap will append a new block to your hyprland.conf.\nThe reset key will be bound to exit the mode.")
        .css_classes(["dim-label", "caption"])
        .halign(gtk::Align::Start)
        .margin_start(24)
        .margin_top(6)
        .build();
    container.append(&info_label);

    // Spacer
    let spacer = gtk::Box::builder().vexpand(true).build();
    container.append(&spacer);

    // --- ACTIONS ---
    let action_bar = gtk::CenterBox::builder()
        .margin_top(12)
        .margin_start(24)
        .margin_end(24)
        .build();

    let apply_btn = create_suggested_button("Create Submap", Some("emblem-ok-symbolic"));
    action_bar.set_end_widget(Some(&apply_btn));
    container.append(&action_bar);

    // --- LOGIC ---
    let model_c = model.clone();
    let toast_overlay_c = toast_overlay.clone();
    let stack_c = stack.clone();
    let entry_name_c = entry_name.clone();
    let entry_enter_mods_c = entry_enter_mods.clone();
    let entry_enter_key_c = entry_enter_key.clone();
    let entry_reset_c = entry_reset.clone();

    apply_btn.connect_clicked(move |_| {
        let name = entry_name_c.text().trim().to_string();
        let enter_mods = entry_enter_mods_c.text().trim().to_string();
        let enter_key = entry_enter_key_c.text().trim().to_string();
        let reset_key = entry_reset_c.text().trim().to_string();

        if name.is_empty() {
            let toast = adw::Toast::builder()
                .title("Submap name cannot be empty")
                .timeout(crate::config::constants::TOAST_TIMEOUT)
                .build();
            toast_overlay_c.add_toast(toast);
            return;
        }

        if let Ok(config_path) = parser::get_config_path() {
            if let Err(e) = parser::create_submap(
                config_path,
                &name,
                if enter_mods.is_empty() {
                    Some("")
                } else {
                    Some(&enter_mods)
                },
                if enter_key.is_empty() {
                    None
                } else {
                    Some(&enter_key)
                },
                if reset_key.is_empty() {
                    None
                } else {
                    Some(&reset_key)
                },
            ) {
                let toast = adw::Toast::builder()
                    .title(format!("Failed to create submap: {}", e))
                    .timeout(crate::config::constants::TOAST_TIMEOUT)
                    .build();
                toast_overlay_c.add_toast(toast);
            } else {
                reload_keybinds(&model_c);

                let toast = adw::Toast::builder()
                    .title(format!("Submap '{}' created successfully", name))
                    .timeout(crate::config::constants::TOAST_TIMEOUT)
                    .build();
                toast_overlay_c.add_toast(toast);

                stack_c.set_visible_child_name("settings");
            }
        }
    });

    container.upcast()
}
