use crate::keybind_object::KeybindObject;
use crate::parser;
use crate::ui::utils::components::create_recorder_row;
use crate::ui::utils::macro_builder::{compile_macro, create_macro_row, parse_macro};
use crate::ui::utils::{
    command_exists, create_destructive_button, create_form_group, create_page_header,
    create_pill_button, create_suggested_button, execute_keybind, perform_backup, reload_keybinds,
    setup_dispatcher_completion,
};
use gtk::glib;
use gtk::{gio, prelude::*};
use gtk4 as gtk;
use libadwaita as adw;
use std::path::PathBuf;
use std::rc::Rc;

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
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    local_stack.add_named(&container, Some("form"));

    // Extract properties early
    let current_mods = obj.property::<String>("mods");
    let current_key = obj.property::<String>("key");
    let current_dispatcher = obj.property::<String>("dispatcher");
    let current_args = obj.property::<String>("args");
    let line_number = obj.property::<u64>("line-number") as usize;
    let file_path_display = obj.property::<String>("file-path");

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

    // Check if current bind is a macro
    let parsed_macro = parse_macro(&current_dispatcher, &current_args);
    let is_macro = parsed_macro.is_some();

    let stack_c = stack.clone();
    let subtitle = if !file_path_display.is_empty() {
        Some(format!("Source: {}", file_path_display))
    } else {
        None
    };

    let header = create_page_header("Edit Keybind", subtitle.as_deref(), "Back", move || {
        stack_c.set_visible_child_name("home");
    });

    container.append(&header);

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

    let macro_switch = gtk::Switch::builder()
        .valign(gtk::Align::Center)
        .active(is_macro)
        .tooltip_text("Enable Chain Actions (Macro Mode)")
        .build();

    let description = obj.property::<String>("description");

    let desc_label = if !description.is_empty() {
        Some(
            gtk::Label::builder()
                .label(&format!("Description: {}", description))
                .css_classes(["dim-label"])
                .hexpand(true)
                .halign(gtk::Align::Start)
                .build(),
        )
    } else {
        None
    };

    let center_widget: Option<&gtk::Widget> = if let Some(ref l) = desc_label {
        Some(l.upcast_ref())
    } else {
        None
    };

    let recorder_box = create_recorder_row(&entry_mods, &entry_key, &macro_switch, center_widget);
    form_box.append(&recorder_box);

    form_box.append(&create_form_group("Modifiers:", &entry_mods));
    form_box.append(&create_form_group("Key:", &entry_key));

    // --- Simple Mode Inputs ---
    let simple_container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .visible(!is_macro)
        .build();

    let entry_dispatcher = gtk::Entry::builder()
        .text(&current_dispatcher)
        .activates_default(true)
        .build();
    setup_dispatcher_completion(&entry_dispatcher);
    simple_container.append(&create_form_group("Dispatcher:", &entry_dispatcher));

    let entry_args = gtk::Entry::builder()
        .text(&display_args)
        .activates_default(true)
        .build();
    if args_had_prefix {
        entry_args.set_placeholder_text(Some("Variable '$' will be added automatically"));
    }
    simple_container.append(&create_form_group("Arguments:", &entry_args));

    form_box.append(&simple_container);

    // --- Macro Mode Inputs ---
    let macro_container_wrapper = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .visible(is_macro)
        .build();

    let macro_list = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .build();
    macro_container_wrapper.append(&macro_list);

    let add_action_btn = gtk::Button::builder()
        .label("Add Action")
        .icon_name("list-add-symbolic")
        .build();

    let macro_list_c = macro_list.clone();
    let add_row = move |disp: Option<&str>, arg: Option<&str>| {
        let (row, _, _, del_btn) = create_macro_row(disp, arg);
        let list_c = macro_list_c.clone();
        let list_c_del = list_c.clone(); // Clone for closure
        let row_c = row.clone();
        del_btn.connect_clicked(move |_| {
            list_c_del.remove(&row_c);
        });
        list_c.append(&row);
    };

    let add_row_c = Rc::new(add_row);
    let add_row_cb = add_row_c.clone();

    add_action_btn.connect_clicked(move |_| {
        add_row_cb(None, None);
    });

    // Populate existing macro rows if any
    if let Some(actions) = parsed_macro {
        for (d, a) in actions {
            add_row_c(Some(&d), Some(&a));
        }
    } else if is_macro {
        // Fallback should not happen due to `is_macro` check but just in case
        add_row_c(None, None);
    } else {
        // If switching TO macro mode from simple, maybe pre-fill with current?
        // For now start empty or with one row
        add_row_c(None, None);
    }

    macro_container_wrapper.append(&add_action_btn);
    form_box.append(&macro_container_wrapper);

    // Toggle Visibility
    let simple_c = simple_container.clone();
    let macro_c = macro_container_wrapper.clone();
    macro_switch.connect_state_set(move |_, state| {
        simple_c.set_visible(!state);
        macro_c.set_visible(state);
        glib::Propagation::Proceed
    });

    let entry_desc = gtk::Entry::builder()
        .text(&obj.property::<String>("description"))
        .placeholder_text("Comment appended to the config line")
        .activates_default(true)
        .build();
    form_box.append(&create_form_group("Description (Optional):", &entry_desc));

    let button_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::End)
        .margin_top(12)
        .build();

    let delete_btn = create_destructive_button("Delete", None);
    let exec_btn = create_pill_button("Execute", None);
    exec_btn.set_tooltip_text(Some("Test this keybind immediately using hyprctl dispatch"));
    let cancel_btn = create_pill_button("Cancel", None);
    let save_btn = create_suggested_button("Save Changes", None);

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

    let confirm_back_btn = create_pill_button("Back", None);
    let confirm_proceed_btn = create_destructive_button("Save Anyway", None);

    confirm_buttons.append(&confirm_back_btn);
    confirm_buttons.append(&confirm_proceed_btn);
    confirm_container.append(&confirm_buttons);

    local_stack.add_named(&confirm_container, Some("confirm"));

    // --- Logic ---

    let entry_dispatcher_exec = entry_dispatcher.clone();
    let entry_args_exec = entry_args.clone();
    let macro_switch_exec = macro_switch.clone();
    let macro_list_exec = macro_list.clone();

    exec_btn.connect_clicked(move |_| {
        let (dispatcher, args) = if macro_switch_exec.is_active() {
            if let Some((d, a)) = compile_macro(&macro_list_exec) {
                (d, a)
            } else {
                return;
            }
        } else {
            (
                entry_dispatcher_exec.text().to_string(),
                entry_args_exec.text().to_string(),
            )
        };

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
        let entry_desc = entry_desc.clone();
        let macro_switch_c = macro_switch.clone();
        let macro_list_c = macro_list.clone();

        Rc::new(move || {
            let input_mods = entry_mods.text().to_string();
            let new_mods = if mods_had_prefix {
                format!("${}", input_mods)
            } else {
                input_mods
            };

            let new_key = entry_key.text().to_string();
            let desc = entry_desc.text().to_string();

            // Resolve Dispatcher/Args
            let (new_dispatcher, new_args) = if macro_switch_c.is_active() {
                match compile_macro(&macro_list_c) {
                    Some(res) => res,
                    None => {
                        let toast = adw::Toast::builder()
                            .title("Macro is empty or invalid")
                            .timeout(3)
                            .build();
                        toast_overlay_clone.add_toast(toast);
                        return;
                    }
                }
            } else {
                let d = entry_dispatcher.text().to_string();
                let input_args = entry_args.text().to_string();
                let a = if args_had_prefix {
                    format!("${}", input_args)
                } else {
                    input_args
                };
                (d, a)
            };

            match parser::update_line(
                file_path.clone(),
                line_number,
                &new_mods,
                &new_key,
                &new_dispatcher,
                &new_args,
                if desc.is_empty() { None } else { Some(desc) },
            ) {
                Ok(_) => {
                    reload_keybinds(&model_clone);

                    if let Err(e) = perform_backup(false) {
                        eprintln!("Auto-backup failed: {}", e);
                    }

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
    let macro_switch_c = macro_switch.clone();
    let macro_list_c = macro_list.clone();

    save_btn.connect_clicked(move |_| {
        let new_key = entry_key_c.text().to_string();

        if new_key.trim().is_empty() {
             let toast = adw::Toast::builder()
                .title("Error: Key cannot be empty")
                .timeout(3)
                .build();
            toast_overlay_clone.add_toast(toast);
            return;
        }

        if macro_switch_c.is_active() {
             if compile_macro(&macro_list_c).is_none() {
                 let toast = adw::Toast::builder()
                    .title("Error: Macro must have at least one valid action")
                    .timeout(3)
                    .build();
                toast_overlay_clone.add_toast(toast);
                return;
             }
        } else {
            let new_dispatcher = entry_dispatcher_save.text().to_string();
            let new_args = entry_args_c.text().to_string(); // Note: prefix is added in do_save, but validation should check raw

             if new_dispatcher.trim().is_empty() {
                let toast = adw::Toast::builder()
                    .title("Error: Dispatcher cannot be empty")
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

                if let Err(e) = perform_backup(false) {
                    eprintln!("Auto-backup failed: {}", e);
                }

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
