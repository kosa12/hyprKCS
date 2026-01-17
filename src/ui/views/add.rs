use crate::parser;
use crate::ui::utils::{
    command_exists,
    execute_keybind,
    reload_keybinds,
    setup_dispatcher_completion,
    setup_key_recorder,
    perform_backup,
};
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
                
                if let Err(e) = perform_backup(false) {
                    eprintln!("Auto-backup failed: {}", e);
                }

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
