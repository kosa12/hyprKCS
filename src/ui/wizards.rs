use crate::keybind_object::KeybindObject;
use crate::parser;
use crate::ui::utils::{
    create_destructive_button, create_page_header, create_pill_button, create_suggested_button,
    normalize, perform_backup,
};
use gtk::{gio, prelude::*};
use gtk4 as gtk;
use libadwaita as adw;
use std::collections::HashMap;

pub fn get_conflict_groups(model: &gio::ListStore) -> Vec<Vec<KeybindObject>> {
    let mut map: HashMap<(Vec<String>, String, String), Vec<KeybindObject>> = HashMap::new();

    for i in 0..model.n_items() {
        if let Some(obj) = model.item(i).and_downcast::<KeybindObject>() {
            if obj.property::<bool>("is-conflicted") {
                let mods = obj.property::<String>("clean-mods");
                let key_raw = obj.property::<String>("key");
                let submap = obj.property::<String>("submap");

                let (sorted_mods, clean_key) = normalize(&mods, &key_raw);
                let key = (sorted_mods, clean_key, submap);

                map.entry(key).or_default().push(obj);
            }
        }
    }

    let mut groups: Vec<Vec<KeybindObject>> = map.into_values().filter(|g| g.len() > 1).collect();

    groups.sort_by(|a, b| {
        let line_a = a
            .first()
            .map(|o| o.property::<u64>("line-number"))
            .unwrap_or(0);
        let line_b = b
            .first()
            .map(|o| o.property::<u64>("line-number"))
            .unwrap_or(0);
        line_a.cmp(&line_b)
    });

    groups
}

pub fn create_conflict_wizard(
    stack: &gtk::Stack,
    model: &gio::ListStore,
    toast_overlay: &adw::ToastOverlay,
    wizard_container: &gtk::Box,
    group_index: usize,
) -> gtk::Widget {
    // Determine current groups
    let groups = get_conflict_groups(model);

    if groups.is_empty() {
        // No conflicts!
        let status = adw::StatusPage::builder()
            .title("No Conflicts Found")
            .description("All keybinds are unique.")
            .icon_name("object-select-symbolic")
            .vexpand(true)
            .build();

        let btn = create_suggested_button("Return Home", None);

        let stack_clone = stack.clone();
        btn.connect_clicked(move |_| {
            stack_clone.set_visible_child_name("home");
        });

        status.set_child(Some(&btn));
        return status.upcast();
    }

    // Wrap index or clamp
    let actual_index = if group_index >= groups.len() {
        0
    } else {
        group_index
    };
    let group = &groups[actual_index];
    let first_obj = &group[0];
    let mods = first_obj.property::<String>("mods");
    let key_char = first_obj.property::<String>("key");

    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .vexpand(true)
        .build();

    // Header
    let stack_c = stack.clone();
    let header_box = create_page_header(
        &format!(
            "Conflict {} of {}: {} {}",
            actual_index + 1,
            groups.len(),
            mods,
            key_char
        ),
        Some(&format!(
            "{} conflicting definitions found. Select an action for each.",
            group.len()
        )),
        "Back",
        move || {
            stack_c.set_visible_child_name("home");
        },
    );
    container.append(&header_box);

    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .build();

    let list_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .build();
    scroll.set_child(Some(&list_box));
    container.append(&scroll);

    // List Items
    for obj in group {
        let row = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .css_classes(["card"])
            .margin_start(4)
            .margin_end(4)
            .build();

        // Inner content
        let content = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let info_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .hexpand(true)
            .build();

        let dispatcher = obj.property::<String>("dispatcher");
        let args = obj.property::<String>("args");
        let file_path = obj.property::<String>("file-path");
        let line_num = obj.property::<u64>("line-number");

        let action_label = gtk::Label::builder()
            .label(&if args.is_empty() {
                dispatcher.clone()
            } else {
                format!("{} {}", dispatcher, args)
            })
            .halign(gtk::Align::Start)
            .css_classes(["heading"])
            .build();

        let file_label = gtk::Label::builder()
            .label(&format!(
                "{}:{}",
                std::path::Path::new(&file_path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
                line_num + 1
            ))
            .halign(gtk::Align::Start)
            .css_classes(["caption", "dim-label"])
            .build();

        info_box.append(&action_label);
        info_box.append(&file_label);
        content.append(&info_box);

        // Actions for this item
        let actions_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .build();

        let delete_btn = create_destructive_button("", Some("user-trash-symbolic"));
        delete_btn.set_tooltip_text(Some("Delete this keybind"));

        let edit_btn = create_pill_button("", Some("document-edit-symbolic"));
        edit_btn.set_tooltip_text(Some("Edit this keybind"));

        actions_box.append(&edit_btn);
        actions_box.append(&delete_btn);
        content.append(&actions_box);

        row.append(&content);
        list_box.append(&row);

        // Wiring up buttons
        let stack_c = stack.clone();
        let model_c = model.clone();
        let toast_overlay_c = toast_overlay.clone();
        let wizard_container_c = wizard_container.clone();
        let _obj_clone = obj.clone();
        let file_path_buf = std::path::PathBuf::from(&file_path);

        // Delete keeps us on the SAME index (the next one slides in)
        delete_btn.connect_clicked(move |_| {
            if let Err(e) = parser::delete_keybind(file_path_buf.clone(), line_num as usize) {
                let toast = adw::Toast::new(&format!("Error: {}", e));
                toast_overlay_c.add_toast(toast);
            } else {
                crate::ui::utils::reload_keybinds(&model_c);

                if let Err(e) = perform_backup(false) {
                    eprintln!("Auto-backup failed: {}", e);
                }

                refresh_wizard(
                    &stack_c,
                    &model_c,
                    &toast_overlay_c,
                    &wizard_container_c,
                    actual_index,
                );
            }
        });

        let stack_c = stack.clone();
        let model_c = model.clone();
        let toast_overlay_c = toast_overlay.clone();
        let _wizard_container_c = wizard_container.clone();
        let obj_clone_2 = obj.clone();

        edit_btn.connect_clicked(move |_| {
            if let Some(edit_page_container) =
                stack_c.child_by_name("edit").and_downcast::<gtk::Box>()
            {
                while let Some(child) = edit_page_container.first_child() {
                    edit_page_container.remove(&child);
                }

                let edit_view = crate::ui::views::create_edit_view(
                    &stack_c,
                    obj_clone_2.clone(),
                    &model_c,
                    &toast_overlay_c,
                    &edit_page_container,
                );
                edit_page_container.append(&edit_view);
                stack_c.set_visible_child_name("edit");
            }
        });
    }

    // Bottom Controls
    let bottom_bar = gtk::CenterBox::builder().margin_top(12).build();

    let skip_btn = create_pill_button("Next Group", None);
    let done_btn = create_pill_button("Finish", None);

    if groups.len() > 1 {
        bottom_bar.set_start_widget(Some(&skip_btn));
    }
    bottom_bar.set_end_widget(Some(&done_btn));

    container.append(&bottom_bar);

    let stack_c = stack.clone();
    done_btn.connect_clicked(move |_| {
        stack_c.set_visible_child_name("home");
    });

    let stack_c = stack.clone();
    let model_c = model.clone();
    let toast_overlay_c = toast_overlay.clone();
    let wizard_container_c = wizard_container.clone();

    // Skip moves to NEXT index
    skip_btn.connect_clicked(move |_| {
        refresh_wizard(
            &stack_c,
            &model_c,
            &toast_overlay_c,
            &wizard_container_c,
            actual_index + 1,
        );
    });

    container.upcast()
}

fn refresh_wizard(
    stack: &gtk::Stack,
    model: &gio::ListStore,
    toast_overlay: &adw::ToastOverlay,
    wizard_container: &gtk::Box,
    index: usize,
) {
    while let Some(child) = wizard_container.first_child() {
        wizard_container.remove(&child);
    }
    let view = create_conflict_wizard(stack, model, toast_overlay, wizard_container, index);
    wizard_container.append(&view);
}
