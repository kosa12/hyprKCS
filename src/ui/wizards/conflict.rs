use crate::keybind_object::KeybindObject;
use crate::parser;
use crate::ui::utils::{
    create_destructive_button, create_page_header, create_pill_button, create_suggested_button,
    normalize, perform_backup,
};
use crate::ui::views::create_edit_view;
use gtk::{gio, prelude::*};
use gtk4 as gtk;
use libadwaita as adw;
use std::collections::HashMap;
use std::sync::Arc;

pub fn get_conflict_groups(model: &gio::ListStore) -> Vec<Vec<KeybindObject>> {
    let mut map: HashMap<(String, String, Arc<str>), Vec<KeybindObject>> = HashMap::new();

    for i in 0..model.n_items() {
        if let Some(obj) = model.item(i).and_downcast::<KeybindObject>() {
            let (is_conflicted, conflict_key) = obj.with_data(|d| {
                if d.is_conflicted {
                    let (sorted_mods, clean_key) = normalize(&d.clean_mods, &d.key);
                    (
                        true,
                        Some((
                            sorted_mods,
                            clean_key,
                            d.submap.clone().unwrap_or_else(|| "".into()),
                        )),
                    )
                } else {
                    (false, None)
                }
            });

            if is_conflicted {
                if let Some(k) = conflict_key {
                    map.entry(k).or_default().push(obj);
                }
            }
        }
    }

    let mut groups: Vec<Vec<KeybindObject>> = map.into_values().filter(|g| g.len() > 1).collect();

    groups.sort_by(|a, b| {
        let line_a = a
            .first()
            .map(|o| o.with_data(|d| d.line_number))
            .unwrap_or(0);
        let line_b = b
            .first()
            .map(|o| o.with_data(|d| d.line_number))
            .unwrap_or(0);
        line_a.cmp(&line_b)
    });

    groups
}

pub fn create_conflict_wizard(
    stack: &gtk::Stack,
    model: &gio::ListStore,
    column_view: &gtk::ColumnView,
    selection_model: &gtk::SingleSelection,
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

        let stack_weak = stack.downgrade();
        btn.connect_clicked(move |_| {
            if let Some(s) = stack_weak.upgrade() {
                s.set_visible_child_name("home");
            }
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
    let (mods, key_char) = first_obj.with_data(|d| (d.mods.to_string(), d.key.to_string()));

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
    let stack_weak = stack.downgrade();
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
            if let Some(s) = stack_weak.upgrade() {
                s.set_visible_child_name("home");
            }
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
        let (dispatcher, args, file_path, line_num) = obj.with_data(|d| {
            (
                d.dispatcher.to_string(),
                d.args.as_ref().map(|s| s.to_string()).unwrap_or_default(),
                d.file_path.to_string(),
                d.line_number,
            )
        });

        let title = if args.is_empty() {
            dispatcher.clone()
        } else {
            format!("{} {}", dispatcher, args)
        };

        let subtitle = format!(
            "{}:{}",
            std::path::Path::new(&file_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            line_num + 1
        );

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

        let row = crate::ui::utils::create_card_row(&title, Some(&subtitle), &actions_box);
        list_box.append(&row);

        // Wiring up buttons
        let stack_weak = stack.downgrade();
        let model_c = model.clone();
        let column_view_weak = column_view.downgrade();
        let selection_model_weak = selection_model.downgrade();
        let toast_overlay_weak = toast_overlay.downgrade();
        let wizard_container_weak = wizard_container.downgrade();
        let file_path_buf = std::path::PathBuf::from(&file_path);

        // Delete keeps us on the SAME index (the next one slides in)
        delete_btn.connect_clicked(move |_| {
            let stack = match stack_weak.upgrade() {
                Some(s) => s,
                None => return,
            };
            let wizard_container = match wizard_container_weak.upgrade() {
                Some(w) => w,
                None => return,
            };
            let toast_overlay = match toast_overlay_weak.upgrade() {
                Some(t) => t,
                None => return,
            };
            let column_view = match column_view_weak.upgrade() {
                Some(c) => c,
                None => return,
            };
            let selection_model = match selection_model_weak.upgrade() {
                Some(s) => s,
                None => return,
            };

            if let Err(e) = parser::delete_keybind(file_path_buf.clone(), line_num as usize) {
                let toast = adw::Toast::builder()
                    .title(format!("Error: {}", e))
                    .timeout(crate::config::constants::TOAST_TIMEOUT)
                    .build();
                toast_overlay.add_toast(toast);
            } else {
                crate::ui::utils::reload_keybinds(&model_c);

                if let Err(e) = perform_backup(false) {
                    eprintln!("Auto-backup failed: {}", e);
                }

                refresh_wizard(
                    &stack,
                    &model_c,
                    &column_view,
                    &selection_model,
                    &toast_overlay,
                    &wizard_container,
                    actual_index,
                );
            }
        });

        let stack_weak = stack.downgrade();
        let model_c = model.clone();
        let column_view_weak = column_view.downgrade();
        let selection_model_weak = selection_model.downgrade();
        let toast_overlay_weak = toast_overlay.downgrade();
        let obj_clone_2 = obj.clone();

        edit_btn.connect_clicked(move |_| {
            let stack = match stack_weak.upgrade() {
                Some(s) => s,
                None => return,
            };
            let column_view = match column_view_weak.upgrade() {
                Some(c) => c,
                None => return,
            };
            let selection_model = match selection_model_weak.upgrade() {
                Some(s) => s,
                None => return,
            };
            let toast_overlay = match toast_overlay_weak.upgrade() {
                Some(t) => t,
                None => return,
            };

            if let Some(edit_page_container) =
                stack.child_by_name("edit").and_downcast::<gtk::Box>()
            {
                while let Some(child) = edit_page_container.first_child() {
                    edit_page_container.remove(&child);
                }

                let edit_view = create_edit_view(
                    &stack,
                    obj_clone_2.clone(),
                    &model_c,
                    &column_view,
                    &selection_model,
                    &toast_overlay,
                    &edit_page_container,
                );
                edit_page_container.append(&edit_view);
                stack.set_visible_child_name("edit");
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

    let stack_weak = stack.downgrade();
    done_btn.connect_clicked(move |_| {
        if let Some(s) = stack_weak.upgrade() {
            s.set_visible_child_name("home");
        }
    });

    let stack_weak = stack.downgrade();
    let model_c = model.clone();
    let column_view_weak = column_view.downgrade();
    let selection_model_weak = selection_model.downgrade();
    let toast_overlay_weak = toast_overlay.downgrade();
    let wizard_container_weak = wizard_container.downgrade();

    // Skip moves to NEXT index
    skip_btn.connect_clicked(move |_| {
        let stack = match stack_weak.upgrade() {
            Some(s) => s,
            None => return,
        };
        let wizard_container = match wizard_container_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let toast_overlay = match toast_overlay_weak.upgrade() {
            Some(t) => t,
            None => return,
        };
        let column_view = match column_view_weak.upgrade() {
            Some(c) => c,
            None => return,
        };
        let selection_model = match selection_model_weak.upgrade() {
            Some(s) => s,
            None => return,
        };

        refresh_wizard(
            &stack,
            &model_c,
            &column_view,
            &selection_model,
            &toast_overlay,
            &wizard_container,
            actual_index + 1,
        );
    });

    container.append(&bottom_bar);
    container.upcast()
}

fn refresh_wizard(
    stack: &gtk::Stack,
    model: &gio::ListStore,
    column_view: &gtk::ColumnView,
    selection_model: &gtk::SingleSelection,
    toast_overlay: &adw::ToastOverlay,
    wizard_container: &gtk::Box,
    index: usize,
) {
    while let Some(child) = wizard_container.first_child() {
        wizard_container.remove(&child);
    }
    let view = create_conflict_wizard(
        stack,
        model,
        column_view,
        selection_model,
        toast_overlay,
        wizard_container,
        index,
    );
    wizard_container.append(&view);
}
