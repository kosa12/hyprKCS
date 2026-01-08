use gtk4 as gtk;
use gtk::{gio, prelude::*};
use libadwaita as adw;
use crate::parser;
use crate::keybind_object::KeybindObject;
use crate::ui::utils::refresh_conflicts;

pub fn show_add_dialog(
    parent: &adw::ApplicationWindow,
    model: gio::ListStore,
    toast_overlay: adw::ToastOverlay,
) {
    let dialog = gtk::Dialog::builder()
        .title("Add Keybind")
        .transient_for(parent)
        .modal(true)
        .default_width(400)
        .build();

    let content_area = dialog.content_area();
    content_area.set_margin_top(12);
    content_area.set_margin_bottom(12);
    content_area.set_margin_start(12);
    content_area.set_margin_end(12);
    content_area.set_spacing(12);

    let label_mods = gtk::Label::new(Some("Modifiers:"));
    label_mods.set_halign(gtk::Align::Start);
    content_area.append(&label_mods);

    let entry_mods = gtk::Entry::builder()
        .placeholder_text("e.g. SUPER")
        .activates_default(true)
        .build();
    content_area.append(&entry_mods);

    let label_key = gtk::Label::new(Some("Key:"));
    label_key.set_halign(gtk::Align::Start);
    content_area.append(&label_key);

    let entry_key = gtk::Entry::builder()
        .placeholder_text("e.g. Q")
        .activates_default(true)
        .build();
    content_area.append(&entry_key);

    let label_dispatcher = gtk::Label::new(Some("Dispatcher:"));
    label_dispatcher.set_halign(gtk::Align::Start);
    content_area.append(&label_dispatcher);

    let entry_dispatcher = gtk::Entry::builder()
        .placeholder_text("e.g. exec")
        .activates_default(true)
        .build();
    content_area.append(&entry_dispatcher);

    let label_args = gtk::Label::new(Some("Arguments:"));
    label_args.set_halign(gtk::Align::Start);
    content_area.append(&label_args);

    let entry_args = gtk::Entry::builder()
        .placeholder_text("e.g. kitty")
        .activates_default(true)
        .build();
    content_area.append(&entry_args);

    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Add", gtk::ResponseType::Ok);
    dialog.set_default_response(gtk::ResponseType::Ok);

    let model_clone = model.clone();
    let toast_overlay_clone = toast_overlay.clone();
    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Ok {
            let mods = entry_mods.text().to_string();
            let key = entry_key.text().to_string();
            let dispatcher = entry_dispatcher.text().to_string();
            let args = entry_args.text().to_string();

            match parser::add_keybind(&mods, &key, &dispatcher, &args) {
                Ok(line_number) => {
                    let kb = parser::Keybind {
                        mods: mods.clone(),
                        clean_mods: mods,
                        flags: String::new(),
                        key,
                        dispatcher,
                        args,
                        line_number,
                    };
                    
                    model_clone.append(&KeybindObject::new(kb, false));
                    refresh_conflicts(&model_clone);

                    let toast = adw::Toast::builder()
                        .title("Keybind added successfully")
                        .timeout(3)
                        .build();
                    toast_overlay_clone.add_toast(toast);
                }
                Err(e) => {
                    let err_dialog = gtk::MessageDialog::builder()
                        .transient_for(dialog)
                        .modal(true)
                        .message_type(gtk::MessageType::Error)
                        .buttons(gtk::ButtonsType::Ok)
                        .text(format!("Failed to add keybind: {}", e))
                        .build();
                    err_dialog.connect_response(|d, _| d.close());
                    err_dialog.present();
                    return;
                }
            }
        }
        dialog.close();
    });

    dialog.present();
}

pub fn show_edit_dialog(
    parent: &adw::ApplicationWindow, 
    current_mods: &str, 
    current_key: &str, 
    current_dispatcher: &str, 
    current_args: &str, 
    line_number: usize, 
    obj: KeybindObject, 
    model: &gio::ListStore,
    toast_overlay: adw::ToastOverlay,
) {
    let (display_mods, mods_had_prefix) = if let Some(stripped) = current_mods.strip_prefix('$') {
        (stripped, true)
    } else {
        (current_mods, false)
    };

    let (display_args, args_had_prefix) = if let Some(stripped) = current_args.strip_prefix('$') {
        (stripped, true)
    } else {
        (current_args, false)
    };

    let dialog = gtk::Dialog::builder()
        .title("Edit Keybind")
        .transient_for(parent)
        .modal(true)
        .default_width(400)
        .build();

    let content_area = dialog.content_area();
    content_area.set_margin_top(12);
    content_area.set_margin_bottom(12);
    content_area.set_margin_start(12);
    content_area.set_margin_end(12);
    content_area.set_spacing(12);

    let label_mods = gtk::Label::new(Some("Modifiers:"));
    label_mods.set_halign(gtk::Align::Start);
    content_area.append(&label_mods);

    let entry_mods = gtk::Entry::builder()
        .text(display_mods)
        .activates_default(true)
        .build();
    if mods_had_prefix {
        entry_mods.set_placeholder_text(Some("Variable '$' will be added automatically"));
    }
    content_area.append(&entry_mods);

    let label_key = gtk::Label::new(Some("Key:"));
    label_key.set_halign(gtk::Align::Start);
    content_area.append(&label_key);

    let entry_key = gtk::Entry::builder()
        .text(current_key)
        .activates_default(true)
        .build();
    content_area.append(&entry_key);

    let label_dispatcher = gtk::Label::new(Some("Dispatcher:"));
    label_dispatcher.set_halign(gtk::Align::Start);
    content_area.append(&label_dispatcher);

    let entry_dispatcher = gtk::Entry::builder()
        .text(current_dispatcher)
        .activates_default(true)
        .build();
    content_area.append(&entry_dispatcher);

    let label_args = gtk::Label::new(Some("Arguments:"));
    label_args.set_halign(gtk::Align::Start);
    content_area.append(&label_args);

    let entry_args = gtk::Entry::builder()
        .text(display_args)
        .activates_default(true)
        .build();
    if args_had_prefix {
        entry_args.set_placeholder_text(Some("Variable '$' will be added automatically"));
    }
    content_area.append(&entry_args);

    dialog.add_button("Delete", gtk::ResponseType::Reject);
    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Save", gtk::ResponseType::Ok);
    dialog.set_default_response(gtk::ResponseType::Ok);

    let obj_clone = obj.clone();
    let model_clone = model.clone();
    let toast_overlay_clone = toast_overlay.clone();
    dialog.connect_response(move |dialog, response| {
        match response {
            gtk::ResponseType::Ok => {
                let input_mods = entry_mods.text().to_string();
                let new_mods = if mods_had_prefix {
                    format!("${}", input_mods)
                } else {
                    input_mods
                };

                let new_key = entry_key.text().to_string();
                let new_dispatcher = entry_dispatcher.text().to_string();

                let input_args = entry_args.text().to_string();
                let new_args = if args_had_prefix {
                    format!("${}", input_args)
                } else {
                    input_args
                };
                
                match parser::update_line(line_number, &new_mods, &new_key, &new_dispatcher, &new_args) {
                    Ok(_) => {
                        obj_clone.set_property("mods", new_mods.to_value());
                        obj_clone.set_property("key", new_key.to_value());
                        obj_clone.set_property("dispatcher", new_dispatcher.to_value());
                        obj_clone.set_property("args", new_args.to_value());
                        
                        refresh_conflicts(&model_clone);

                        let toast = adw::Toast::builder()
                            .title("Keybind saved successfully")
                            .timeout(3)
                            .build();
                        toast_overlay_clone.add_toast(toast);
                    }
                    Err(e) => {
                        eprintln!("Failed to update config: {}", e);
                        let err_dialog = gtk::MessageDialog::builder()
                            .transient_for(dialog)
                            .modal(true)
                            .message_type(gtk::MessageType::Error)
                            .buttons(gtk::ButtonsType::Ok)
                            .text(format!("Failed to save changes: {}", e))
                            .build();
                        err_dialog.connect_response(|d, _| d.close());
                        err_dialog.present();
                        return;
                    }
                }
            }
            gtk::ResponseType::Reject => {
                match parser::delete_keybind(line_number) {
                    Ok(_) => {
                        let mut index_to_remove = None;
                        for i in 0..model_clone.n_items() {
                            if let Some(item) = model_clone.item(i).and_downcast::<KeybindObject>() {
                                if item == obj_clone {
                                    index_to_remove = Some(i);
                                } else if index_to_remove.is_some() {
                                    let current_ln = item.property::<u64>("line-number");
                                    item.set_property("line-number", current_ln - 1);
                                }
                            }
                        }
                        if let Some(idx) = index_to_remove {
                            model_clone.remove(idx);
                            refresh_conflicts(&model_clone);

                            let toast = adw::Toast::builder()
                                .title("Keybind deleted")
                                .timeout(3)
                                .build();
                            toast_overlay_clone.add_toast(toast);
                        }
                    }
                    Err(e) => {
                        let err_dialog = gtk::MessageDialog::builder()
                            .transient_for(dialog)
                            .modal(true)
                            .message_type(gtk::MessageType::Error)
                            .buttons(gtk::ButtonsType::Ok)
                            .text(format!("Failed to delete keybind: {}", e))
                            .build();
                        err_dialog.connect_response(|d, _| d.close());
                        err_dialog.present();
                        return;
                    }
                }
            }
            _ => {}
        }
        dialog.close();
    });

    dialog.present();
}

