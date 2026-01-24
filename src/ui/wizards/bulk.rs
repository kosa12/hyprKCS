use crate::keybind_object::KeybindObject;
use crate::parser;
use crate::ui::utils::{
    create_page_header, create_suggested_button, perform_backup, reload_keybinds,
};
use gtk::{gio, prelude::*};
use gtk4 as gtk;
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Debug)]
enum ReplaceTarget {
    Modifiers,
    Key,
    Dispatcher,
    Arguments,
}

pub fn create_bulk_replace_wizard(
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
        "Bulk Replace",
        Some("Search and replace parts of your keybinds"),
        "Back",
        move || {
            stack_c.set_visible_child_name("home");
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
        .build();

    // Target Selection
    let target_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    target_row.append(&gtk::Label::new(Some("Target Field:")));

    let target_combo =
        gtk::DropDown::from_strings(&["Modifiers", "Key", "Dispatcher", "Arguments"]);
    target_combo.set_hexpand(true);
    target_row.append(&target_combo);
    form_box.append(&target_row);

    // Search / Replace
    let grid = gtk::Grid::builder()
        .column_spacing(12)
        .row_spacing(12)
        .build();

    let lbl_find = gtk::Label::builder()
        .label("Find:")
        .halign(gtk::Align::Start)
        .build();
    let entry_find = gtk::Entry::builder()
        .placeholder_text("Text to find...")
        .hexpand(true)
        .build();

    let lbl_replace = gtk::Label::builder()
        .label("Replace with:")
        .halign(gtk::Align::Start)
        .build();
    let entry_replace = gtk::Entry::builder()
        .placeholder_text("New text...")
        .hexpand(true)
        .build();

    grid.attach(&lbl_find, 0, 0, 1, 1);
    grid.attach(&entry_find, 1, 0, 1, 1);
    grid.attach(&lbl_replace, 0, 1, 1, 1);
    grid.attach(&entry_replace, 1, 1, 1, 1);

    form_box.append(&grid);
    container.append(&form_box);

    // --- PREVIEW ---
    let preview_label = gtk::Label::builder()
        .label("Preview (0 matches)")
        .css_classes(["heading"])
        .halign(gtk::Align::Start)
        .margin_start(24)
        .build();
    container.append(&preview_label);

    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .margin_start(24)
        .margin_end(24)
        .css_classes(["view"])
        .build();

    let preview_list = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(6)
        .build();
    scroll.set_child(Some(&preview_list));
    container.append(&scroll);

    // --- ACTIONS ---
    let action_bar = gtk::CenterBox::builder().margin_top(12).build();
    let apply_btn = create_suggested_button("Apply Changes", Some("emblem-ok-symbolic"));
    apply_btn.set_sensitive(false);
    action_bar.set_end_widget(Some(&apply_btn));
    container.append(&action_bar);

    // --- LOGIC ---
    let model_c = model.clone();
    let preview_list_c = preview_list.clone();
    let preview_label_c = preview_label.clone();
    let apply_btn_c = apply_btn.clone();

    // Store pending changes: List of (KeybindObject, NewValue)
    // We need to keep this state to apply later.
    type PendingChange = (KeybindObject, String);
    let pending_changes: Rc<RefCell<Vec<PendingChange>>> = Rc::new(RefCell::new(Vec::new()));
    let pending_changes_c = pending_changes.clone();

    let entry_find_c = entry_find.clone();
    let entry_replace_c = entry_replace.clone();
    let target_combo_c_prev = target_combo.clone();

    let update_preview = Rc::new(move || {
        let find_text = entry_find_c.text().to_string();
        let replace_text = entry_replace_c.text().to_string();
        let target_idx = target_combo_c_prev.selected();

        // Clear list
        while let Some(child) = preview_list_c.first_child() {
            preview_list_c.remove(&child);
        }
        pending_changes_c.borrow_mut().clear();

        if find_text.is_empty() {
            preview_label_c.set_label("Preview (0 matches)");
            apply_btn_c.set_sensitive(false);
            return;
        }

        let target = match target_idx {
            0 => ReplaceTarget::Modifiers,
            1 => ReplaceTarget::Key,
            2 => ReplaceTarget::Dispatcher,
            _ => ReplaceTarget::Arguments,
        };

        let find_text_lower = find_text.to_lowercase();
        let mut count = 0;
        for i in 0..model_c.n_items() {
            if let Some(obj) = model_c.item(i).and_downcast::<KeybindObject>() {
                let current_val = match target {
                    ReplaceTarget::Modifiers => obj.property::<String>("mods"),
                    ReplaceTarget::Key => obj.property::<String>("key"),
                    ReplaceTarget::Dispatcher => obj.property::<String>("dispatcher"),
                    ReplaceTarget::Arguments => obj.property::<String>("args"),
                };

                let current_val_lower = current_val.to_lowercase();

                if current_val_lower.contains(&find_text_lower) {
                    // Case-insensitive replacement
                    let mut new_val = String::new();
                    let mut last_end = 0;
                    for (start, _) in current_val_lower.match_indices(&find_text_lower) {
                        new_val.push_str(&current_val[last_end..start]);
                        new_val.push_str(&replace_text);
                        last_end = start + find_text_lower.len();
                    }
                    new_val.push_str(&current_val[last_end..]);

                    // Display Row
                    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
                    row.set_margin_start(12);
                    row.set_margin_end(12);

                    let lbl_orig = gtk::Label::builder()
                        .label(&current_val)
                        .hexpand(true)
                        .halign(gtk::Align::Start)
                        .css_classes(["dim-label"])
                        .ellipsize(gtk::pango::EllipsizeMode::End)
                        .build();

                    let arrow = gtk::Image::from_icon_name("go-next-symbolic");

                    let lbl_new = gtk::Label::builder()
                        .label(&new_val)
                        .hexpand(true)
                        .halign(gtk::Align::Start)
                        .css_classes(["accent"])
                        .ellipsize(gtk::pango::EllipsizeMode::End)
                        .build();

                    // Context info (e.g. key if changing dispatcher)
                    let context_str = match target {
                        ReplaceTarget::Dispatcher | ReplaceTarget::Arguments => {
                            format!(
                                "{} + {}",
                                obj.property::<String>("mods"),
                                obj.property::<String>("key")
                            )
                        }
                        _ => format!("{}", obj.property::<String>("dispatcher")),
                    };
                    let lbl_ctx = gtk::Label::builder()
                        .label(&context_str)
                        .width_chars(12)
                        .halign(gtk::Align::Start)
                        .ellipsize(gtk::pango::EllipsizeMode::End)
                        .build();

                    row.append(&lbl_ctx);
                    row.append(&lbl_orig);
                    row.append(&arrow);
                    row.append(&lbl_new);

                    // Set tooltip to show the full keybind
                    let full_bind = format!(
                        "{} {}, {} {}",
                        obj.property::<String>("mods"),
                        obj.property::<String>("key"),
                        obj.property::<String>("dispatcher"),
                        obj.property::<String>("args")
                    );
                    row.set_tooltip_text(Some(&full_bind));

                    preview_list_c.append(&row);

                    pending_changes_c.borrow_mut().push((obj.clone(), new_val));
                    count += 1;
                }
            }
        }

        preview_label_c.set_label(&format!("Preview ({} matches)", count));
        apply_btn_c.set_sensitive(count > 0);
    });

    // Connect signals
    let up_prev = update_preview.clone();
    target_combo.connect_selected_notify(move |_| up_prev());

    let up_prev = update_preview.clone();
    entry_find.connect_changed(move |_| up_prev());

    let up_prev = update_preview.clone();
    entry_replace.connect_changed(move |_| up_prev());

    // Apply Logic
    let model_apply = model.clone();
    let toast_overlay_c = toast_overlay.clone();
    let stack_c = stack.clone();
    let target_combo_c = target_combo.clone();

    apply_btn.connect_clicked(move |_| {
        let changes = pending_changes.borrow();
        if changes.is_empty() {
            return;
        }

        let target_idx = target_combo_c.selected();
        let target = match target_idx {
            0 => ReplaceTarget::Modifiers,
            1 => ReplaceTarget::Key,
            2 => ReplaceTarget::Dispatcher,
            _ => ReplaceTarget::Arguments,
        };

        // Perform backup ONCE
        if let Err(e) = perform_backup(false) {
            eprintln!("Backup failed: {}", e);
        }

        let mut success_count = 0;
        let mut error_count = 0;

        for (obj, new_val) in changes.iter() {
            let file_path = std::path::PathBuf::from(obj.property::<String>("file-path"));
            let line_number = obj.property::<u64>("line-number") as usize;

            let mut mods = obj.property::<String>("mods");
            let mut key = obj.property::<String>("key");
            let mut disp = obj.property::<String>("dispatcher");
            let mut args = obj.property::<String>("args");
            let desc = obj.property::<String>("description");

            match target {
                ReplaceTarget::Modifiers => mods = new_val.clone(),
                ReplaceTarget::Key => key = new_val.clone(),
                ReplaceTarget::Dispatcher => disp = new_val.clone(),
                ReplaceTarget::Arguments => args = new_val.clone(),
            }

            match parser::update_line(
                file_path,
                line_number,
                &mods,
                &key,
                &disp,
                &args,
                if desc.is_empty() { None } else { Some(desc) },
            ) {
                Ok(_) => success_count += 1,
                Err(e) => {
                    eprintln!("Failed to update line {}: {}", line_number, e);
                    error_count += 1;
                }
            }
        }

        reload_keybinds(&model_apply);

        let msg = if error_count > 0 {
            format!(
                "Updated {} keybinds. {} failed.",
                success_count, error_count
            )
        } else {
            format!("Successfully updated {} keybinds.", success_count)
        };

        let toast = adw::Toast::new(&msg);
        toast_overlay_c.add_toast(toast);
        stack_c.set_visible_child_name("home");
    });

    container.upcast()
}
