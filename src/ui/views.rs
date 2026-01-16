use crate::keybind_object::KeybindObject;
use crate::parser;
use crate::ui::utils::{
    command_exists, execute_hyprctl, execute_keybind, reload_keybinds, setup_dispatcher_completion,
};
use gtk::{gdk, gio, glib, prelude::*};
use gtk4 as gtk;
use libadwaita as adw;
use std::path::PathBuf;
use std::rc::Rc;

fn gdk_to_hypr_mods(mods: gdk::ModifierType) -> String {
    let mut res = Vec::new();
    if mods.contains(gdk::ModifierType::SUPER_MASK) {
        res.push("SUPER");
    }
    if mods.contains(gdk::ModifierType::CONTROL_MASK) {
        res.push("CONTROL");
    }
    if mods.contains(gdk::ModifierType::ALT_MASK) {
        res.push("ALT");
    }
    if mods.contains(gdk::ModifierType::SHIFT_MASK) {
        res.push("SHIFT");
    }
    res.join(" ")
}

fn gdk_to_hypr_key(key: gdk::Key) -> String {
    match key {
        gdk::Key::Return => "Return".to_string(),
        gdk::Key::Tab => "Tab".to_string(),
        gdk::Key::space => "Space".to_string(),
        gdk::Key::Escape => "Escape".to_string(),
        _ => {
            if let Some(name) = key.name() {
                name.to_string()
            } else {
                "".to_string()
            }
        }
    }
}

pub fn setup_key_recorder(container: &gtk::Box, entry_mods: &gtk::Entry, entry_key: &gtk::Entry) {
    let record_btn = gtk::Button::builder()
        .label("Record Combo")
        .tooltip_text("Click then press your key combination")
        .css_classes(["record-btn"])
        .build();

    // Create the controller once and attach it to the button
    let controller = gtk::EventControllerKey::new();
    record_btn.add_controller(controller.clone());

    let entry_mods = entry_mods.clone();
    let entry_key = entry_key.clone();

    let controller_weak = controller.downgrade();
    let entry_mods_weak = entry_mods.downgrade();
    let entry_key_weak = entry_key.downgrade();

    let on_click = move |btn: &gtk::Button| {
        let _controller = match controller_weak.upgrade() {
            Some(c) => c,
            None => return,
        };
        let _entry_mods = match entry_mods_weak.upgrade() {
            Some(c) => c,
            None => return,
        };
        let _entry_key = match entry_key_weak.upgrade() {
            Some(c) => c,
            None => return,
        };

        // If already listening, stop listening and reset
        if btn.label().map_or(false, |l| l == "Listening...") {
            btn.set_label("Record Combo");
            btn.remove_css_class("suggested-action");
            execute_hyprctl(&["reload"]);
            return;
        }

        btn.set_label("Listening...");
        btn.add_css_class("suggested-action");
        btn.grab_focus(); // Ensure we catch keys

        // Define the submap with a dummy bind to ensure it's created and recognized
        execute_hyprctl(&["--batch", "keyword submap hyprkcs_blocking ; keyword bind , code:248, exec, true ; keyword submap reset"]);
        execute_hyprctl(&["dispatch", "submap", "hyprkcs_blocking"]);
    };

    record_btn.connect_clicked(on_click);

    let record_btn_weak = record_btn.downgrade();
    let entry_mods_weak_2 = entry_mods.downgrade();
    let entry_key_weak_2 = entry_key.downgrade();

    let on_keypress = move |_: &gtk::EventControllerKey,
                            key: gdk::Key,
                            _: u32,
                            mods: gdk::ModifierType|
          -> glib::Propagation {
        let record_btn = match record_btn_weak.upgrade() {
            Some(c) => c,
            None => return glib::Propagation::Proceed,
        };
        let entry_mods = match entry_mods_weak_2.upgrade() {
            Some(c) => c,
            None => return glib::Propagation::Proceed,
        };
        let entry_key = match entry_key_weak_2.upgrade() {
            Some(c) => c,
            None => return glib::Propagation::Proceed,
        };

        // Only process if we are actually listening
        if record_btn.label().map_or(true, |l| l != "Listening...") {
            return glib::Propagation::Proceed;
        }

        if matches!(
            key,
            gdk::Key::Control_L
                | gdk::Key::Control_R
                | gdk::Key::Alt_L
                | gdk::Key::Alt_R
                | gdk::Key::Super_L
                | gdk::Key::Super_R
                | gdk::Key::Shift_L
                | gdk::Key::Shift_R
                | gdk::Key::Meta_L
                | gdk::Key::Meta_R
        ) {
            return glib::Propagation::Proceed;
        }

        let hypr_mods = gdk_to_hypr_mods(mods);
        let hypr_key = gdk_to_hypr_key(key);

        if !hypr_key.is_empty() {
            entry_mods.set_text(&hypr_mods);
            entry_key.set_text(&hypr_key);
        }

        record_btn.set_label("Record Combo");
        record_btn.remove_css_class("suggested-action");
        execute_hyprctl(&["reload"]);

        glib::Propagation::Stop
    };

    // Connect the key handler
    controller.connect_key_pressed(on_keypress);

    container.append(&record_btn);
}

pub fn create_add_view(
    stack: &gtk::Stack,
    model: &gio::ListStore,
    toast_overlay: &adw::ToastOverlay,
) -> gtk::Widget {
    let local_stack = gtk::Stack::builder()
        .transition_type(gtk::StackTransitionType::SlideLeftRight)
        .build();

    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    local_stack.add_named(&container, Some("form"));

    let title_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let title = gtk::Label::builder()
        .label("Add New Keybind")
        .css_classes(["title-2"])
        .hexpand(true)
        .halign(gtk::Align::Start)
        .build();
    title_box.append(&title);
    container.append(&title_box);

    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .build();

    let form_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .build();
    scroll.set_child(Some(&form_box));
    container.append(&scroll);

    let entry_mods = gtk::Entry::builder()
        .placeholder_text("e.g. SUPER")
        .activates_default(true)
        .build();

    let entry_key = gtk::Entry::builder()
        .placeholder_text("e.g. Q")
        .activates_default(true)
        .build();

    let recorder_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    setup_key_recorder(&recorder_box, &entry_mods, &entry_key);
    form_box.append(&recorder_box);

    let label_mods = gtk::Label::new(Some("Modifiers:"));
    label_mods.set_halign(gtk::Align::Start);
    form_box.append(&label_mods);
    form_box.append(&entry_mods);

    let label_key = gtk::Label::new(Some("Key:"));
    label_key.set_halign(gtk::Align::Start);
    form_box.append(&label_key);
    form_box.append(&entry_key);

    let label_dispatcher = gtk::Label::new(Some("Dispatcher:"));
    label_dispatcher.set_halign(gtk::Align::Start);
    form_box.append(&label_dispatcher);

    let entry_dispatcher = gtk::Entry::builder()
        .placeholder_text("e.g. exec")
        .activates_default(true)
        .build();
    setup_dispatcher_completion(&entry_dispatcher);
    form_box.append(&entry_dispatcher);

    let label_args = gtk::Label::new(Some("Arguments:"));
    label_args.set_halign(gtk::Align::Start);
    form_box.append(&label_args);

    let entry_args = gtk::Entry::builder()
        .placeholder_text("e.g. kitty")
        .activates_default(true)
        .build();
    form_box.append(&entry_args);

    let label_submap = gtk::Label::new(Some("Submap (Optional):"));
    label_submap.set_halign(gtk::Align::Start);
    form_box.append(&label_submap);

    let entry_submap = gtk::Entry::builder()
        .placeholder_text("e.g. resize (leave empty for global)")
        .activates_default(true)
        .build();
    form_box.append(&entry_submap);

    let button_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::End)
        .margin_top(12)
        .build();

    let cancel_btn = gtk::Button::builder().label("Cancel").build();

    let exec_btn = gtk::Button::builder()
        .label("Execute")
        .tooltip_text("Test this keybind immediately using hyprctl dispatch")
        .build();

    let add_btn = gtk::Button::builder()
        .label("Add Keybind")
        .css_classes(["suggested-action"])
        .build();

    button_box.append(&cancel_btn);
    button_box.append(&exec_btn);
    button_box.append(&add_btn);
    container.append(&button_box);

    // --- Confirmation View Construction ---
    let confirm_container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(24)
        .valign(gtk::Align::Center)
        .halign(gtk::Align::Center)
        .build();
    
    let confirm_icon = gtk::Image::builder()
        .icon_name("dialog-warning-symbolic")
        .pixel_size(64)
        .css_classes(["error-icon"])
        .build();
    confirm_container.append(&confirm_icon);

    let confirm_title = gtk::Label::builder()
        .label("Command Not Found")
        .css_classes(["title-2"])
        .build();
    confirm_container.append(&confirm_title);

    let confirm_label = gtk::Label::builder()
        .label("Placeholder text")
        .justify(gtk::Justification::Center)
        .wrap(true)
        .max_width_chars(40)
        .build();
    confirm_container.append(&confirm_label);

    let confirm_buttons = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Center)
        .build();
    
    let confirm_back_btn = gtk::Button::builder()
        .label("Back")
        .build();
    
    let confirm_proceed_btn = gtk::Button::builder()
        .label("Add Anyway")
        .css_classes(["destructive-action"])
        .build();

    confirm_buttons.append(&confirm_back_btn);
    confirm_buttons.append(&confirm_proceed_btn);
    confirm_container.append(&confirm_buttons);

    local_stack.add_named(&confirm_container, Some("confirm"));

    // --- Logic ---

    let entry_dispatcher_exec = entry_dispatcher.clone();
    let entry_args_exec = entry_args.clone();
    exec_btn.connect_clicked(move |_| {
        let dispatcher = entry_dispatcher_exec.text().to_string();
        let args = entry_args_exec.text().to_string();
        if !dispatcher.trim().is_empty() {
            execute_keybind(&dispatcher, &args);
        }
    });

    let stack_c = stack.clone();
    cancel_btn.connect_clicked(move |_| {
        stack_c.set_visible_child_name("home");
    });

    let local_stack_c = local_stack.clone();
    confirm_back_btn.connect_clicked(move |_| {
        local_stack_c.set_visible_child_name("form");
    });

    // We need to store the pending action (add)
    // Since we don't have easy state management, we'll re-trigger the add logic
    // but bypass validation. Or simpler: use a RefCell for the pending add closure?
    // Actually, create_add_view is one-shot.
    // We can just define the "do_add" logic and call it from confirm_proceed_btn.
    // But confirm_proceed_btn needs access to the entry values.

    let model_clone = model.clone();
    let toast_overlay_clone = toast_overlay.clone();
    let entry_mods_c = entry_mods.clone();
    let entry_key_c = entry_key.clone();
    let entry_dispatcher_c = entry_dispatcher.clone();
    let entry_args_c = entry_args.clone();
    let entry_submap_c = entry_submap.clone();
    let stack_c = stack.clone();

    // Core Add Logic
    let perform_add = Rc::new(move || {
        let mods = entry_mods_c.text().to_string();
        let key = entry_key_c.text().to_string();
        let dispatcher = entry_dispatcher_c.text().to_string();
        let args = entry_args_c.text().to_string();
        let submap_raw = entry_submap_c.text().to_string();
        let submap = if submap_raw.trim().is_empty() {
            None
        } else {
            Some(submap_raw.trim().to_string())
        };

        let config_path = parser::get_config_path().unwrap();
        match parser::add_keybind(
            config_path.clone(),
            &mods,
            &key,
            &dispatcher,
            &args,
            submap.clone(),
        ) {
            Ok(_) => {
                reload_keybinds(&model_clone);
                let toast = adw::Toast::builder()
                    .title("Keybind added successfully")
                    .timeout(3)
                    .build();
                toast_overlay_clone.add_toast(toast);
                stack_c.set_visible_child_name("home");
            }
            Err(e) => {
                let toast = adw::Toast::builder()
                    .title(&format!("Error: {}", e))
                    .timeout(5)
                    .build();
                toast_overlay_clone.add_toast(toast);
            }
        }
    });

    let perform_add_c = perform_add.clone();
    confirm_proceed_btn.connect_clicked(move |_| {
        perform_add_c();
    });

    let entry_key_c = entry_key.clone();
    let entry_dispatcher_c = entry_dispatcher.clone();
    let entry_args_c = entry_args.clone();
    let toast_overlay_clone = toast_overlay.clone();
    let local_stack_c = local_stack.clone();
    let confirm_label_c = confirm_label.clone();

    add_btn.connect_clicked(move |_| {
        let key = entry_key_c.text().to_string();
        let dispatcher = entry_dispatcher_c.text().to_string();
        let args = entry_args_c.text().to_string();

        if key.trim().is_empty() || dispatcher.trim().is_empty() {
            let toast = adw::Toast::builder()
                .title("Error: Key and Dispatcher cannot be empty")
                .timeout(3)
                .build();
            toast_overlay_clone.add_toast(toast);
            return;
        }

        // Validation
        if dispatcher == "exec" || dispatcher == "execr" {
            let cmd = args.trim();
            if !command_exists(cmd) {
                confirm_label_c.set_label(&format!(
                    "The command '{}' was not found in your PATH.\nAre you sure you want to add this keybind?",
                    cmd
                ));
                local_stack_c.set_visible_child_name("confirm");
                return;
            }
        }

        perform_add();
    });

    local_stack.upcast::<gtk::Widget>()
}

pub fn create_edit_view(
    stack: &gtk::Stack,
    obj: KeybindObject,
    model: &gio::ListStore,
    toast_overlay: &adw::ToastOverlay,
    _editing_page: &gtk::Box,
) -> gtk::Widget {
    // We use a local stack to switch between the Edit Form and the Confirmation View
    let local_stack = gtk::Stack::builder()
        .transition_type(gtk::StackTransitionType::SlideLeftRight)
        .build();

    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();
    
    local_stack.add_named(&container, Some("form"));

    let title_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let title = gtk::Label::builder()
        .label("Edit Keybind")
        .css_classes(["title-2"])
        .hexpand(true)
        .halign(gtk::Align::Start)
        .build();
    title_box.append(&title);
    container.append(&title_box);

    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .build();

    let form_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .build();
    scroll.set_child(Some(&form_box));
    container.append(&scroll);

    let current_mods = obj.property::<String>("mods");
    let current_key = obj.property::<String>("key");
    let current_dispatcher = obj.property::<String>("dispatcher");
    let current_args = obj.property::<String>("args");
    let line_number = obj.property::<u64>("line-number") as usize;

    let (display_mods, mods_had_prefix) = if let Some(stripped) = current_mods.strip_prefix('$') {
        (stripped.to_string(), true)
    } else {
        (current_mods.clone(), false)
    };

    let (display_args, args_had_prefix) = if let Some(stripped) = current_args.strip_prefix('$') {
        (stripped.to_string(), true)
    } else {
        (current_args.clone(), false)
    };

    let entry_mods = gtk::Entry::builder()
        .text(&display_mods)
        .activates_default(true)
        .build();
    if mods_had_prefix {
        entry_mods.set_placeholder_text(Some("Variable '$' will be added automatically"));
    }

    let entry_key = gtk::Entry::builder()
        .text(&current_key)
        .activates_default(true)
        .build();

    let recorder_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    setup_key_recorder(&recorder_box, &entry_mods, &entry_key);
    form_box.append(&recorder_box);

    let file_path_display = obj.property::<String>("file-path");
    if !file_path_display.is_empty() {
        let path_label = gtk::Label::builder()
            .label(&format!("Source: {}", file_path_display))
            .halign(gtk::Align::Start)
            .css_classes(["caption", "dim-label"]) // Use adwaita/gtk style classes for smaller text
            .build();
        form_box.append(&path_label);
    }

    let label_mods = gtk::Label::new(Some("Modifiers:"));
    label_mods.set_halign(gtk::Align::Start);
    form_box.append(&label_mods);
    form_box.append(&entry_mods);

    let label_key = gtk::Label::new(Some("Key:"));
    label_key.set_halign(gtk::Align::Start);
    form_box.append(&label_key);
    form_box.append(&entry_key);

    let label_dispatcher = gtk::Label::new(Some("Dispatcher:"));
    label_dispatcher.set_halign(gtk::Align::Start);
    form_box.append(&label_dispatcher);

    let entry_dispatcher = gtk::Entry::builder()
        .text(&current_dispatcher)
        .activates_default(true)
        .build();
    setup_dispatcher_completion(&entry_dispatcher);
    form_box.append(&entry_dispatcher);

    let label_args = gtk::Label::new(Some("Arguments:"));
    label_args.set_halign(gtk::Align::Start);
    form_box.append(&label_args);

    let entry_args = gtk::Entry::builder()
        .text(&display_args)
        .activates_default(true)
        .build();
    if args_had_prefix {
        entry_args.set_placeholder_text(Some("Variable '$' will be added automatically"));
    }
    form_box.append(&entry_args);

    let button_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::End)
        .margin_top(12)
        .build();

    let delete_btn = gtk::Button::builder()
        .label("Delete")
        .css_classes(["destructive-action"])
        .build();

    let exec_btn = gtk::Button::builder()
        .label("Execute")
        .tooltip_text("Test this keybind immediately using hyprctl dispatch")
        .build();

    let cancel_btn = gtk::Button::builder().label("Cancel").build();

    let save_btn = gtk::Button::builder()
        .label("Save Changes")
        .css_classes(["suggested-action"])
        .build();

    button_box.append(&delete_btn);
    let spacer = gtk::Box::builder().hexpand(true).build();
    button_box.append(&spacer);
    button_box.append(&exec_btn);
    button_box.append(&cancel_btn);
    button_box.append(&save_btn);
    container.append(&button_box);

    // --- Confirmation View Construction ---
    let confirm_container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(24)
        .valign(gtk::Align::Center)
        .halign(gtk::Align::Center)
        .build();
    
    let confirm_icon = gtk::Image::builder()
        .icon_name("dialog-warning-symbolic")
        .pixel_size(64)
        .css_classes(["error-icon"])
        .build();
    confirm_container.append(&confirm_icon);

    let confirm_title = gtk::Label::builder()
        .label("Command Not Found")
        .css_classes(["title-2"])
        .build();
    confirm_container.append(&confirm_title);

    let confirm_label = gtk::Label::builder()
        .label("Placeholder text")
        .justify(gtk::Justification::Center)
        .wrap(true)
        .max_width_chars(40)
        .build();
    confirm_container.append(&confirm_label);

    let confirm_buttons = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Center)
        .build();
    
    let confirm_back_btn = gtk::Button::builder()
        .label("Back")
        .build();
    
    let confirm_proceed_btn = gtk::Button::builder()
        .label("Save Anyway")
        .css_classes(["destructive-action"])
        .build();

    confirm_buttons.append(&confirm_back_btn);
    confirm_buttons.append(&confirm_proceed_btn);
    confirm_container.append(&confirm_buttons);

    local_stack.add_named(&confirm_container, Some("confirm"));

    // --- Logic ---

    let entry_dispatcher_exec = entry_dispatcher.clone();
    let entry_args_exec = entry_args.clone();
    exec_btn.connect_clicked(move |_| {
        let dispatcher = entry_dispatcher_exec.text().to_string();
        let args = entry_args_exec.text().to_string();
        if !dispatcher.trim().is_empty() {
            execute_keybind(&dispatcher, &args);
        }
    });

    let stack_c = stack.clone();
    cancel_btn.connect_clicked(move |_| {
        stack_c.set_visible_child_name("home");
    });

    let local_stack_c = local_stack.clone();
    confirm_back_btn.connect_clicked(move |_| {
        local_stack_c.set_visible_child_name("form");
    });

    let model_clone = model.clone();
    let toast_overlay_clone = toast_overlay.clone();
    let file_path_str = obj.property::<String>("file-path");
    let file_path = PathBuf::from(&file_path_str);
    let stack_c = stack.clone();
    
    // Core Save Logic
    let do_save = {
        let file_path = file_path.clone();
        let model_clone = model_clone.clone();
        let toast_overlay_clone = toast_overlay_clone.clone();
        let stack_c = stack_c.clone();
        let entry_mods = entry_mods.clone();
        let entry_key = entry_key.clone();
        let entry_dispatcher = entry_dispatcher.clone();
        let entry_args = entry_args.clone();

        Rc::new(move || {
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

            match parser::update_line(
                file_path.clone(),
                line_number,
                &new_mods,
                &new_key,
                &new_dispatcher,
                &new_args,
            ) {
                Ok(_) => {
                    reload_keybinds(&model_clone);
                    let toast = adw::Toast::builder()
                        .title("Keybind saved")
                        .timeout(3)
                        .build();
                    toast_overlay_clone.add_toast(toast);
                    stack_c.set_visible_child_name("home");
                }
                Err(e) => {
                    let toast = adw::Toast::builder()
                        .title(&format!("Error: {}", e))
                        .timeout(5)
                        .build();
                    toast_overlay_clone.add_toast(toast);
                }
            }
        })
    };

    let do_save_c = do_save.clone();
    confirm_proceed_btn.connect_clicked(move |_| {
        do_save_c();
    });

    let entry_dispatcher_save = entry_dispatcher.clone();
    let local_stack_c = local_stack.clone();
    let confirm_label_c = confirm_label.clone();
    let toast_overlay_clone = toast_overlay.clone();
    let entry_key_c = entry_key.clone();
    let entry_args_c = entry_args.clone();

    save_btn.connect_clicked(move |_| {
        let new_key = entry_key_c.text().to_string();
        let new_dispatcher = entry_dispatcher_save.text().to_string();
        let input_args = entry_args_c.text().to_string();
        let new_args = if args_had_prefix {
            format!("${}", input_args)
        } else {
            input_args
        };

        if new_key.trim().is_empty() || new_dispatcher.trim().is_empty() {
            let toast = adw::Toast::builder()
                .title("Error: Key and Dispatcher cannot be empty")
                .timeout(3)
                .build();
            toast_overlay_clone.add_toast(toast);
            return;
        }

        // Command Validation for exec
        if new_dispatcher == "exec" || new_dispatcher == "execr" {
            let cmd = new_args.trim();
            // Don't validate if it looks like a variable
            if !cmd.starts_with('$') {
                if !command_exists(cmd) {
                    confirm_label_c.set_label(&format!(
                        "The command '{}' was not found in your PATH.\nAre you sure you want to save this keybind?",
                        cmd
                    ));
                    local_stack_c.set_visible_child_name("confirm");
                    return;
                }
            }
        }

        do_save();
    });

    let model_clone = model.clone();
    let toast_overlay_clone = toast_overlay.clone();
    let file_path_str = obj.property::<String>("file-path");
    let file_path = PathBuf::from(&file_path_str);
    let stack_c = stack.clone();

    delete_btn.connect_clicked(move |_| {
        match parser::delete_keybind(file_path.clone(), line_number) {
            Ok(_) => {
                reload_keybinds(&model_clone);
                let toast = adw::Toast::builder()
                    .title("Keybind deleted")
                    .timeout(3)
                    .build();
                toast_overlay_clone.add_toast(toast);
                stack_c.set_visible_child_name("home");
            }
            Err(e) => {
                let toast = adw::Toast::builder()
                    .title(&format!("Error: {}", e))
                    .timeout(5)
                    .build();
                toast_overlay_clone.add_toast(toast);
            }
        }
    });

    local_stack.upcast()
}
