use crate::parser;
use crate::ui::utils::components::{
    create_flags_dropdown, create_mouse_button_dropdown, create_recorder_row, get_flag_from_index,
    get_mouse_code_from_index,
};
use crate::ui::utils::conflicts::{check_conflict, generate_suggestions};
use crate::ui::utils::macro_builder::{compile_macro, create_macro_row};
use crate::ui::utils::{
    command_exists, create_destructive_button, create_form_group, create_page_header,
    create_pill_button, create_suggested_button, execute_keybind, perform_backup, reload_keybinds,
    setup_dispatcher_completion,
};
use gtk::glib;
use gtk::{gio, prelude::*};
use gtk4 as gtk;
use libadwaita as adw;
use std::rc::Rc;

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
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    local_stack.add_named(&container, Some("form"));

    let stack_weak = stack.downgrade();
    let header = create_page_header(
        "Add New Keybind",
        Some("Fill in the details below to add a new keybind"),
        "Back",
        move || {
            if let Some(s) = stack_weak.upgrade() {
                s.set_visible_child_name("home");
            }
        },
    );

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
        .placeholder_text("e.g. SUPER")
        .activates_default(true)
        .build();

    let entry_key = gtk::Entry::builder()
        .placeholder_text("e.g. Q")
        .activates_default(true)
        .build();

    let mouse_dropdown = create_mouse_button_dropdown();
    mouse_dropdown.set_visible(false);

    let key_container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(6)
        .build();
    key_container.append(&entry_key);
    key_container.append(&mouse_dropdown);

    let macro_switch = gtk::Switch::builder()
        .valign(gtk::Align::Center)
        .tooltip_text("Enable Chain Actions (Multiple dispatchers)")
        .build();

    let mouse_switch = gtk::Switch::builder()
        .valign(gtk::Align::Center)
        .tooltip_text("Bind to a mouse button instead of a key")
        .build();

    let recorder_box = create_recorder_row(
        &entry_mods,
        &entry_key,
        &macro_switch,
        Some(&mouse_switch),
        None,
    );

    form_box.append(&recorder_box);

    form_box.append(&create_form_group("Modifiers:", &entry_mods));
    form_box.append(&create_form_group("Key / Button:", &key_container));

    let entry_key_c_vis = entry_key.clone();
    let mouse_dropdown_c_vis = mouse_dropdown.clone();

    mouse_switch.connect_state_set(move |_, state| {
        entry_key_c_vis.set_visible(!state);
        mouse_dropdown_c_vis.set_visible(state);
        glib::Propagation::Proceed
    });

    let flags_dropdown = create_flags_dropdown();
    form_box.append(&create_form_group("Behavior (Flags):", &flags_dropdown));

    // --- Simple Mode Inputs ---
    let simple_container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .build();

    let entry_dispatcher = gtk::Entry::builder()
        .placeholder_text("e.g. exec")
        .activates_default(true)
        .build();
    setup_dispatcher_completion(&entry_dispatcher);
    simple_container.append(&create_form_group("Dispatcher:", &entry_dispatcher));

    let entry_args = gtk::Entry::builder()
        .placeholder_text("e.g. kitty")
        .activates_default(true)
        .build();
    simple_container.append(&create_form_group("Arguments:", &entry_args));

    form_box.append(&simple_container);

    // --- Macro Mode Inputs ---
    let macro_container_wrapper = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .visible(false)
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

    // Logic to add rows
    let macro_list_c = macro_list.clone();
    add_action_btn.connect_clicked(move |_| {
        let (row, _, _, del_btn) = create_macro_row(None, None);
        let list_c = macro_list_c.clone();
        let list_c_del = list_c.clone(); // Clone for closure
        let row_c = row.clone();
        del_btn.connect_clicked(move |_| {
            list_c_del.remove(&row_c);
        });
        list_c.append(&row);
    });
    // Add one initial row
    add_action_btn.emit_clicked();

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

    let entry_submap = gtk::Entry::builder()
        .placeholder_text("e.g. resize (leave empty for global)")
        .activates_default(true)
        .build();
    form_box.append(&create_form_group("Submap (Optional):", &entry_submap));

    let entry_desc = gtk::Entry::builder()
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

    let cancel_btn = create_pill_button("Cancel", None);
    let exec_btn = create_pill_button("Execute", None);
    exec_btn.set_tooltip_text(Some("Test this keybind immediately using hyprctl dispatch"));
    let add_btn = create_suggested_button("Add Keybind", None);

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

    let confirm_back_btn = create_pill_button("Back", None);
    let confirm_proceed_btn = create_destructive_button("Add Anyway", None);

    confirm_buttons.append(&confirm_back_btn);
    confirm_buttons.append(&confirm_proceed_btn);
    confirm_container.append(&confirm_buttons);

    local_stack.add_named(&confirm_container, Some("confirm"));

    // --- Conflict View Construction ---
    use crate::ui::utils::conflicts::create_conflict_panel;
    let conflict_panel = create_conflict_panel("Add Anyway");
    local_stack.add_named(&conflict_panel.container, Some("conflict"));

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

    let stack_weak = stack.downgrade();
    cancel_btn.connect_clicked(move |_| {
        if let Some(s) = stack_weak.upgrade() {
            s.set_visible_child_name("home");
        }
    });

    let local_stack_weak = local_stack.downgrade();
    confirm_back_btn.connect_clicked(move |_| {
        if let Some(ls) = local_stack_weak.upgrade() {
            ls.set_visible_child_name("form");
        }
    });

    let local_stack_weak = local_stack.downgrade();
    conflict_panel.back_btn.connect_clicked(move |_| {
        if let Some(ls) = local_stack_weak.upgrade() {
            ls.set_visible_child_name("form");
        }
    });

    let model_clone = model.clone();
    let toast_overlay_clone = toast_overlay.clone();
    let entry_mods_c = entry_mods.clone();
    let entry_key_c = entry_key.clone();
    let entry_dispatcher_c = entry_dispatcher.clone();
    let entry_args_c = entry_args.clone();
    let entry_submap_c = entry_submap.clone();
    let entry_desc_c = entry_desc.clone();
    let macro_switch_c = macro_switch.clone();
    let macro_list_c = macro_list.clone();
    let flags_dropdown_c = flags_dropdown.clone();
    let mouse_switch_c = mouse_switch.clone();
    let mouse_dropdown_c = mouse_dropdown.clone();
    let stack_weak = stack.downgrade();

    // Core Add Logic
    let perform_add = Rc::new(move || {
        let mods = entry_mods_c.text().to_string();

        let key = if mouse_switch_c.is_active() {
            get_mouse_code_from_index(mouse_dropdown_c.selected()).to_string()
        } else {
            entry_key_c.text().to_string()
        };

        let flag = get_flag_from_index(flags_dropdown_c.selected());

        // Determine Dispatcher/Args based on mode
        let (dispatcher, args) = if macro_switch_c.is_active() {
            match compile_macro(&macro_list_c) {
                Some(res) => res,
                None => {
                    let toast = adw::Toast::builder()
                        .title("Macro is empty or invalid")
                        .timeout(crate::config::constants::TOAST_TIMEOUT)
                        .build();
                    toast_overlay_clone.add_toast(toast);
                    return;
                }
            }
        } else {
            (
                entry_dispatcher_c.text().to_string(),
                entry_args_c.text().to_string(),
            )
        };

        let desc = entry_desc_c.text().to_string();
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
            if desc.is_empty() { None } else { Some(desc) },
            flag,
        ) {
            Ok(_) => {
                reload_keybinds(&model_clone);

                if let Err(e) = perform_backup(false) {
                    eprintln!("Auto-backup failed: {}", e);
                }

                let toast = adw::Toast::builder()
                    .title("Keybind added successfully")
                    .timeout(crate::config::constants::TOAST_TIMEOUT)
                    .build();
                toast_overlay_clone.add_toast(toast);
                if let Some(s) = stack_weak.upgrade() {
                    s.set_visible_child_name("home");
                }
            }
            Err(e) => {
                let toast = adw::Toast::builder()
                    .title(format!("Error: {}", e))
                    .timeout(crate::config::constants::TOAST_TIMEOUT)
                    .build();
                toast_overlay_clone.add_toast(toast);
            }
        }
    });

    let perform_add_c = perform_add.clone();
    confirm_proceed_btn.connect_clicked(move |_| {
        perform_add_c();
    });

    let perform_add_c = perform_add.clone();
    conflict_panel.proceed_btn.connect_clicked(move |_| {
        perform_add_c();
    });

    let entry_mods_c = entry_mods.clone();
    let entry_key_c = entry_key.clone();
    let entry_dispatcher_c = entry_dispatcher.clone();
    let entry_args_c = entry_args.clone();
    let entry_submap_c = entry_submap.clone();
    let macro_switch_c = macro_switch.clone();
    let macro_list_c = macro_list.clone();
    let toast_overlay_clone = toast_overlay.clone();
    let local_stack_c = local_stack.clone();
    let confirm_label_c = confirm_label.clone();
    let conflict_target_label_c = conflict_panel.target_label.clone();
    let conflict_suggestions_box_c = conflict_panel.suggestions_box.clone();

    let mouse_switch_c = mouse_switch.clone();
    let mouse_dropdown_c = mouse_dropdown.clone();
    let model_c = model.clone();

    add_btn.connect_clicked(move |_| {
        let key = if mouse_switch_c.is_active() {
            get_mouse_code_from_index(mouse_dropdown_c.selected()).to_string()
        } else {
            entry_key_c.text().to_string()
        };
        let mods = entry_mods_c.text().to_string();
        let submap_raw = entry_submap_c.text().to_string();
        let submap_trimmed = submap_raw.trim();
        let submap = if submap_trimmed.is_empty() {
            None
        } else {
            Some(submap_trimmed)
        };

        if !mouse_switch_c.is_active() && key.trim().is_empty() {
             let toast = adw::Toast::builder()
                .title("Error: Key cannot be empty")
                .timeout(crate::config::constants::TOAST_TIMEOUT)
                .build();
            toast_overlay_clone.add_toast(toast);
            return;
        }

        if macro_switch_c.is_active() {
            // In macro mode, we skip simple validation for now
            // We could parse the 'bash -c' string but it's complex.
            // Just ensure it's not empty
             if compile_macro(&macro_list_c).is_none() {
                 let toast = adw::Toast::builder()
                    .title("Error: Macro must have at least one valid action")
                    .timeout(crate::config::constants::TOAST_TIMEOUT)
                    .build();
                toast_overlay_clone.add_toast(toast);
                return;
             }
        } else {
            let dispatcher = entry_dispatcher_c.text().to_string();
            let args = entry_args_c.text().to_string();
            if dispatcher.trim().is_empty() {
                let toast = adw::Toast::builder()
                    .title("Error: Dispatcher cannot be empty")
                    .timeout(crate::config::constants::TOAST_TIMEOUT)
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
        }

        let variables = parser::get_variables().unwrap_or_else(|e| {
            eprintln!("Failed to load variables for conflict checking: {}", e);
            std::collections::HashMap::new()
        });

        // Check for conflicts
        if let Some(conflict) = check_conflict(&mods, &key, submap, None, &model_c, &variables) {
            conflict_target_label_c.set_label(&format!(
                "Dispatcher: {}\nArgs: {}\nFile: {}:{}",
                conflict.dispatcher, conflict.args, conflict.file, conflict.line
            ));

            // Populate suggestions
            while let Some(child) = conflict_suggestions_box_c.first_child() {
                conflict_suggestions_box_c.remove(&child);
            }

            let suggestions = generate_suggestions(&mods, &key, submap, &model_c, &variables);
            if suggestions.is_empty() {
                conflict_suggestions_box_c.append(&gtk::Label::new(Some("No simple alternatives found.")));
            } else {
                for (s_mods, s_key) in suggestions {
                    let btn = create_suggested_button(&format!("{} + {}", s_mods, s_key), None);
                    let entry_mods_c_s = entry_mods_c.clone();
                    let entry_key_c_s = entry_key_c.clone();
                    let s_mods_str = s_mods.clone();
                    let s_key_str = s_key.clone();
                    let local_stack_weak = local_stack_c.downgrade();

                    btn.connect_clicked(move |_| {
                        if let Some(ls) = local_stack_weak.upgrade() {
                            entry_mods_c_s.set_text(&s_mods_str);
                            entry_key_c_s.set_text(&s_key_str);
                            ls.set_visible_child_name("form");
                        }
                    });
                    conflict_suggestions_box_c.append(&btn);
                }
            }

            local_stack_c.set_visible_child_name("conflict");
            return;
        }

        perform_add();
    });

    local_stack.upcast::<gtk::Widget>()
}
