use crate::parser::{self, Variable};
use crate::ui::utils::components::*;
use gtk4 as gtk;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_variables_page(
    _window: &adw::ApplicationWindow,
    on_show_toast: Rc<dyn Fn(String)>,
) -> gtk::Widget {
    let stack = gtk::Stack::builder()
        .transition_type(gtk::StackTransitionType::SlideLeftRight)
        .build();

    // ================== LIST VIEW ==================
    let list_box_container = gtk::Box::new(gtk::Orientation::Vertical, 0);

    // Top Bar for List (Add + Search)
    let top_bar = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let search_entry = gtk::SearchEntry::builder()
        .placeholder_text("Search variables...")
        .hexpand(true)
        .build();

    let add_btn = create_suggested_button("Add New", Some("list-add-symbolic"));

    top_bar.append(&search_entry);
    top_bar.append(&add_btn);
    list_box_container.append(&top_bar);

    // Scrollable List
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .build();

    // We use a clamp for better aesthetics
    let clamp = adw::Clamp::builder().maximum_size(800).build();

    let list_group_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
    list_group_box.set_margin_top(0);
    list_group_box.set_margin_bottom(24);
    list_group_box.set_margin_start(12);
    list_group_box.set_margin_end(12);

    let list_box = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(vec!["boxed-list".to_string()])
        .build();

    list_group_box.append(&list_box);
    clamp.set_child(Some(&list_group_box));
    scrolled.set_child(Some(&clamp));

    list_box_container.append(&scrolled);
    stack.add_named(&list_box_container, Some("list"));

    // ================== EDIT VIEW ==================
    let edit_container = gtk::Box::new(gtk::Orientation::Vertical, 0);

    // Header
    let stack_c = stack.clone();
    let header = create_page_header("Edit Variable", Some(" "), "Back", move || {
        stack_c.set_visible_child_name("list");
    });
    header.set_margin_top(12);
    header.set_margin_bottom(12);
    header.set_margin_start(12);
    header.set_margin_end(12);

    // Add Save button to header
    let save_btn = create_suggested_button("", Some("document-save-symbolic"));
    save_btn.add_css_class("circular");
    save_btn.set_tooltip_text(Some("Save Variable"));
    header.append(&save_btn);

    edit_container.append(&header);
    edit_container.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    // Form
    let form_scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .build();

    let form_clamp = adw::Clamp::builder().maximum_size(600).build();

    let form_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(24)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(12)
        .margin_end(12)
        .build();

    let pref_group = adw::PreferencesGroup::builder()
        .title("Variable Details")
        .description("Define the name and value for this variable.")
        .build();

    // Use ActionRow with Entry for cleaner look (EntryRow is 1.2+)
    let name_entry = gtk::Entry::builder()
        .placeholder_text("mainMod")
        .valign(gtk::Align::Center)
        .build();
    let name_row = adw::ActionRow::builder()
        .title("Name (without $)")
        .activatable(true) // Focus entry on click
        .build();
    name_row.add_suffix(&name_entry);
    // Connect row activation to entry focus
    let ne = name_entry.clone();
    name_row.connect_activate(move |_| {
        ne.grab_focus();
    });

    let value_entry = gtk::Entry::builder()
        .placeholder_text("SUPER")
        .valign(gtk::Align::Center)
        .build();
    let value_row = adw::ActionRow::builder()
        .title("Value")
        .activatable(true)
        .build();
    value_row.add_suffix(&value_entry);
    let ve = value_entry.clone();
    value_row.connect_activate(move |_| {
        ve.grab_focus();
    });

    let refactor_switch = gtk::Switch::builder().valign(gtk::Align::Center).build();
    let refactor_row = adw::ActionRow::builder()
        .title("Refactor Keybinds")
        .subtitle("Replace hardcoded values in keybinds with this variable")
        .activatable(false)
        .build();
    refactor_row.add_suffix(&refactor_switch);

    pref_group.add(&name_row);
    pref_group.add(&value_row);
    pref_group.add(&refactor_row);

    form_box.append(&pref_group);
    form_clamp.set_child(Some(&form_box));
    form_scroll.set_child(Some(&form_clamp));

    edit_container.append(&form_scroll);
    stack.add_named(&edit_container, Some("edit"));

    // ================== DELETE CONFIRM VIEW ==================
    let delete_container = gtk::Box::new(gtk::Orientation::Vertical, 0);

    // Header for Delete
    let stack_cd = stack.clone();
    let delete_header = create_page_header("Confirm Deletion", None, "Cancel", move || {
        stack_cd.set_visible_child_name("list");
    });
    delete_header.set_margin_top(12);
    delete_header.set_margin_bottom(12);
    delete_header.set_margin_start(12);
    delete_header.set_margin_end(12);
    delete_container.append(&delete_header);

    let delete_content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .vexpand(true)
        .build();

    let delete_content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(24)
        .valign(gtk::Align::Center)
        .halign(gtk::Align::Center)
        .margin_top(48)
        .margin_bottom(48)
        .margin_start(24)
        .margin_end(24)
        .build();

    let warning_icon = gtk::Image::builder()
        .icon_name("dialog-warning-symbolic")
        .pixel_size(32)
        .css_classes(vec!["warning".to_string()])
        .build();

    let warning_title = gtk::Label::builder()
        .label("Dependency Detected")
        .css_classes(vec!["title-2".to_string()])
        .build();

    let warning_body = gtk::Label::builder()
        .label("Variable is used in configuration.")
        .wrap(true)
        .justify(gtk::Justification::Center)
        .max_width_chars(40)
        .build();

    let delete_confirm_btn = create_destructive_button("", Some("edit-delete-symbolic"));
    delete_confirm_btn.add_css_class("circular"); // Use circular for 1:1 ratio
    delete_confirm_btn.set_halign(gtk::Align::Center);
    delete_confirm_btn.set_tooltip_text(Some("Delete & Replace All References"));

    delete_content.append(&warning_icon);
    delete_content.append(&warning_title);
    delete_content.append(&warning_body);
    delete_content.append(&delete_confirm_btn);

    let delete_clamp = adw::Clamp::builder()
        .maximum_size(500)
        .child(&delete_content)
        .build();
    delete_content_box.append(&delete_clamp);
    delete_container.append(&delete_content_box);

    stack.add_named(&delete_container, Some("delete_confirm"));

    // ================== DUPLICATE CONFIRM VIEW ==================
    let dup_container = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let stack_dup = stack.clone();
    let dup_header = create_page_header("Duplicate Variable", None, "Back", move || {
        stack_dup.set_visible_child_name("edit"); // Go back to edit
    });
    dup_header.set_margin_top(12);
    dup_header.set_margin_bottom(12);
    dup_header.set_margin_start(12);
    dup_header.set_margin_end(12);
    dup_container.append(&dup_header);

    let dup_content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .vexpand(true)
        .build();

    let dup_content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(24)
        .valign(gtk::Align::Center)
        .halign(gtk::Align::Center)
        .margin_top(48)
        .margin_bottom(48)
        .margin_start(24)
        .margin_end(24)
        .build();

    let dup_icon = gtk::Image::builder()
        .icon_name("dialog-information-symbolic")
        .pixel_size(32)
        .css_classes(vec!["accent".to_string()])
        .build();

    let dup_title = gtk::Label::builder()
        .label("Variable Already Exists")
        .css_classes(vec!["title-2".to_string()])
        .build();

    let dup_body = gtk::Label::builder()
        .label("A variable with this name already exists.")
        .wrap(true)
        .justify(gtk::Justification::Center)
        .max_width_chars(40)
        .build();

    let dup_confirm_btn = create_suggested_button("", Some("document-save-symbolic"));
    dup_confirm_btn.add_css_class("circular");
    dup_confirm_btn.set_halign(gtk::Align::Center);
    dup_confirm_btn.set_tooltip_text(Some("Save Anyway"));

    dup_content.append(&dup_icon);
    dup_content.append(&dup_title);
    dup_content.append(&dup_body);
    dup_content.append(&dup_confirm_btn);

    let dup_clamp = adw::Clamp::builder()
        .maximum_size(500)
        .child(&dup_content)
        .build();
    dup_content_box.append(&dup_clamp);
    dup_container.append(&dup_content_box);

    stack.add_named(&dup_container, Some("duplicate_confirm"));

    // ================== LOGIC ==================
    let current_var: Rc<RefCell<Option<Variable>>> = Rc::new(RefCell::new(None));

    // -- Refresh Forward Declaration --
    let refresh_handle: Rc<RefCell<Option<Rc<dyn Fn()>>>> = Rc::new(RefCell::new(None));

    // -- Header Update Logic --
    let header_c = header.clone();
    let update_header_title: Rc<dyn Fn(&str, Option<&str>)> =
        Rc::new(move |title: &str, subtitle: Option<&str>| {
            if let Some(_) = header_c.last_child() {
                let mut i = 0;
                let mut child_opt = header_c.first_child();
                while let Some(widget) = child_opt {
                    if i == 1 {
                        if let Some(title_box) = widget.downcast_ref::<gtk::Box>() {
                            // First child is Title
                            if let Some(lbl_widget) = title_box.first_child() {
                                if let Some(lbl) = lbl_widget.downcast_ref::<gtk::Label>() {
                                    lbl.set_label(title);
                                }
                            }
                            // Second child is Subtitle (if it exists)
                            if let Some(lbl_widget) =
                                title_box.first_child().and_then(|w| w.next_sibling())
                            {
                                if let Some(lbl) = lbl_widget.downcast_ref::<gtk::Label>() {
                                    if let Some(sub) = subtitle {
                                        lbl.set_label(sub);
                                        lbl.set_visible(true);
                                    } else {
                                        lbl.set_visible(false);
                                    }
                                }
                            }
                        }
                        break;
                    }
                    child_opt = widget.next_sibling();
                    i += 1;
                }
            }
        });

    // -- Shared Save Logic --
    let stack_s = stack.clone();
    let current_var_s = current_var.clone();
    let name_entry_s = name_entry.clone();
    let value_entry_s = value_entry.clone();
    let toast_s = on_show_toast.clone();
    let refresh_s_handle = refresh_handle.clone();
    let refactor_switch_s = refactor_switch.clone();

    let perform_save = Rc::new(move || {
        let name = name_entry_s.text().to_string();
        let value = value_entry_s.text().to_string();

        if name.trim().is_empty() {
            toast_s("Variable name cannot be empty".to_string());
            return;
        }

        // Backup before modifying
        if let Err(e) = crate::ui::utils::backup::perform_backup(false) {
            eprintln!("Failed to backup config: {}", e);
        }

        let clean_name = if name.starts_with('$') {
            name[1..].to_string()
        } else {
            name
        };

        let res = if let Some(v) = &*current_var_s.borrow() {
            let old_name_clean = v.name.trim_start_matches('$');
            if old_name_clean != clean_name {
                if let Err(e) = parser::rename_variable_references(old_name_clean, &clean_name) {
                    toast_s(format!("Failed to rename variable references: {}", e));
                    return;
                }
            }
            parser::update_variable(v.file_path.clone(), v.line_number, &clean_name, &value)
        } else {
            match parser::get_config_path() {
                Ok(path) => parser::add_variable(path, &clean_name, &value),
                Err(e) => Err(anyhow::anyhow!("Config path not found: {}", e)),
            }
        };

        match res {
            Ok(_) => {
                // Handle Refactor
                if refactor_switch_s.is_active() {
                    match parser::refactor_hardcoded_references(&value, &clean_name) {
                        Ok(count) => {
                            if count > 0 {
                                toast_s(format!(
                                    "Saved successfully. Refactored {} usages.",
                                    count
                                ));
                            } else {
                                toast_s(
                                    "Saved successfully. No usages found to refactor.".to_string(),
                                );
                            }
                        }
                        Err(e) => {
                            toast_s(format!("Saved, but refactor failed: {}", e));
                        }
                    }
                } else {
                    toast_s("Saved successfully".to_string());
                }

                stack_s.set_visible_child_name("list");
                if let Some(refresh) = &*refresh_s_handle.borrow() {
                    refresh();
                }
            }
            Err(e) => {
                toast_s(format!("Error: {}", e));
            }
        }
    });

    // Delete Confirm Action
    let current_var_del = current_var.clone();
    let toast_del = on_show_toast.clone();
    let stack_del = stack.clone();
    let refresh_del_handle = refresh_handle.clone();

    delete_confirm_btn.connect_clicked(move |_| {
        if let Some(var) = &*current_var_del.borrow() {
            // Backup before modifying
            if let Err(e) = crate::ui::utils::backup::perform_backup(false) {
                eprintln!("Failed to backup config: {}", e);
            }

            let name_clean = var.name.trim_start_matches('$');

            // 1. Delete Definition
            match parser::delete_variable(var.file_path.clone(), var.line_number) {
                Ok(_) => {
                    // 2. Replace References
                    match parser::inline_variable_references(name_clean, &var.value) {
                        Ok(_) => {
                            toast_del("Variable deleted and references replaced".to_string());
                        }
                        Err(e) => {
                            toast_del(format!(
                                "Deleted variable, but failed to replace some references: {}",
                                e
                            ));
                        }
                    }

                    stack_del.set_visible_child_name("list");
                    if let Some(refresh) = &*refresh_del_handle.borrow() {
                        refresh();
                    }
                }
                Err(e) => {
                    toast_del(format!("Error deleting variable: {}", e));
                }
            }
        }
    });

    // Save Anyway Action
    let perform_save_dup = perform_save.clone();
    dup_confirm_btn.connect_clicked(move |_| {
        perform_save_dup();
    });

    // Main Save Action
    let perform_save_main = perform_save.clone();
    let name_entry_check = name_entry.clone();
    let current_var_check = current_var.clone();
    let stack_check = stack.clone();
    let dup_body_c = dup_body.clone();

    save_btn.connect_clicked(move |_| {
        let name = name_entry_check.text().to_string();
        if name.trim().is_empty() {
             perform_save_main(); // Let shared logic handle empty error
             return;
        }

        let clean_name = if name.starts_with('$') {
            name[1..].to_string()
        } else {
            name
        };
        let target_name = format!("${}", clean_name);

        // Check for duplicates
        let is_duplicate = if let Ok(vars) = parser::get_variables() {
            if vars.contains_key(&target_name) {
                // If editing, check if it's strictly a *rename* to an existing var
                if let Some(current) = &*current_var_check.borrow() {
                     current.name.as_ref() != target_name
                } else {
                     true // Adding new var that exists
                }
            } else {
                false
            }
        } else {
            false
        };

        if is_duplicate {
            dup_body_c.set_label(&format!("A variable named '{}' already exists.\nSaving will overwrite usage of that variable.", target_name));
            stack_check.set_visible_child_name("duplicate_confirm");
        } else {
            perform_save_main();
        }
    });

    // -- Refresh Definition --
    // We need to capture widgets for refresh_list_ui
    let list_box_c = list_box.clone();
    let search_entry_c = search_entry.clone();
    let stack_refresh = stack.clone();
    let current_var_refresh = current_var.clone();
    let name_entry_c = name_entry.clone();
    let value_entry_c = value_entry.clone();
    let toast_c = on_show_toast.clone();
    let update_header_title_refresh = update_header_title.clone();
    let warning_body_refresh = warning_body.clone();
    let delete_confirm_btn_refresh = delete_confirm_btn.clone();
    // Also reset the refactor switch on add/edit
    let refactor_switch_refresh = refactor_switch.clone();

    let refresh_impl = Rc::new(move || {
        refresh_list_ui(
            &list_box_c,
            &search_entry_c,
            &stack_refresh,
            &current_var_refresh,
            &name_entry_c,
            &value_entry_c,
            &toast_c,
            &update_header_title_refresh,
            &warning_body_refresh,
            &delete_confirm_btn_refresh,
            &refactor_switch_refresh,
        );
    });

    *refresh_handle.borrow_mut() = Some(refresh_impl.clone());

    // Initial
    refresh_impl();

    // Search
    let refresh_search = refresh_impl.clone();
    search_entry.connect_search_changed(move |_| {
        refresh_search();
    });

    // Add Action
    let stack_c = stack.clone();
    let current_var_c = current_var.clone();
    let name_entry_c = name_entry.clone();
    let value_entry_c = value_entry.clone();
    let update_title_c = update_header_title.clone();
    let refactor_switch_add = refactor_switch.clone();

    add_btn.connect_clicked(move |_| {
        *current_var_c.borrow_mut() = None;
        name_entry_c.set_text("");
        value_entry_c.set_text("");
        refactor_switch_add.set_active(false); // Reset switch
        update_title_c("Add New Variable", None);
        stack_c.set_visible_child_name("edit");
        name_entry_c.grab_focus();
    });

    // Previous save_btn.connect_clicked removed as it is replaced above logic

    stack.upcast()
}

#[allow(clippy::too_many_arguments)]
fn refresh_list_ui(
    list_box: &gtk::ListBox,
    search_entry: &gtk::SearchEntry,
    stack: &gtk::Stack,
    current_var: &Rc<RefCell<Option<Variable>>>,
    name_entry: &gtk::Entry,
    value_entry: &gtk::Entry,
    toast: &Rc<dyn Fn(String)>,
    update_header_title: &Rc<dyn Fn(&str, Option<&str>)>,
    warning_body: &gtk::Label,
    delete_confirm_btn: &gtk::Button,
    refactor_switch: &gtk::Switch,
) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    // Capture list_box_r context recursively
    // We need to clone the Refresh callback components to pass recursively
    // This is getting complex with closures.
    // Instead of full recursion with all args, maybe we can simplify.
    // But we need to re-call `refresh_list_ui`.
    // Let's just pass the same args.

    let filter = search_entry.text().to_string().to_lowercase();

    match parser::get_defined_variables() {
        Ok(mut vars) => {
            if !filter.is_empty() {
                vars.retain(|v| {
                    v.name.to_lowercase().contains(&filter)
                        || v.value.to_lowercase().contains(&filter)
                });
            }

            vars.sort_by(|a, b| a.name.cmp(&b.name));

            if vars.is_empty() {
                let row = adw::ActionRow::builder()
                    .title(if filter.is_empty() {
                        "No variables defined"
                    } else {
                        "No matches found"
                    })
                    .sensitive(false)
                    .build();
                list_box.append(&row);
                return;
            }

            for var in vars {
                let row = adw::ActionRow::builder()
                    .title(&*var.name) // name already has $ usually, display as is
                    .subtitle(&*var.value)
                    .selectable(false)
                    .activatable(true) // Clicking row edits it
                    .build();

                // Actions Suffix
                let box_actions = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                box_actions.set_valign(gtk::Align::Center);

                let delete_btn = create_destructive_button("", Some("user-trash-symbolic"));
                delete_btn.set_tooltip_text(Some("Delete"));
                delete_btn.add_css_class("flat"); // Make it flat initially

                box_actions.append(&delete_btn);
                row.add_suffix(&box_actions);

                // Add chevron to indicate edit
                let chevron = gtk::Image::from_icon_name("go-next-symbolic");
                row.add_suffix(&chevron);

                list_box.append(&row);

                // Edit Logic (Row Click)
                let stack_c = stack.clone();
                let current_var_c = current_var.clone();
                let name_entry_c = name_entry.clone();
                let value_entry_c = value_entry.clone();
                let var_clone = var.clone();
                let update_title_c = update_header_title.clone();

                row.connect_activated(move |_| {
                    *current_var_c.borrow_mut() = Some(var_clone.clone());
                    // Strip $ for entry
                    let clean_name = var_clone.name.trim_start_matches('$');
                    name_entry_c.set_text(clean_name);
                    value_entry_c.set_text(&var_clone.value);

                    let path_str = var_clone.file_path.to_string_lossy();
                    let subtitle = format!("{}:{}", path_str, var_clone.line_number + 1);
                    update_title_c("Edit Variable", Some(&subtitle));

                    stack_c.set_visible_child_name("edit");
                });

                // Delete Logic
                let confirm_state = Rc::new(RefCell::new(false));
                let var_del = var.clone();
                let toast_c = toast.clone();

                // Helpers for refresh recursion
                let list_box_r = list_box.clone();
                let search_entry_r = search_entry.clone();
                let stack_r = stack.clone();
                let current_var_r = current_var.clone();
                let name_entry_r = name_entry.clone();
                let value_entry_r = value_entry.clone();
                let toast_r = toast.clone();
                let update_title_r = update_header_title.clone();

                let warning_body_r = warning_body.clone();
                let delete_confirm_btn_r = delete_confirm_btn.clone();
                let refactor_switch_r = refactor_switch.clone();

                // Disconnect previous signal handlers on the shared confirm button?
                // Actually, the button is shared globally in the UI function, so we need to be careful.
                // Every time we click delete on a row, we might set up the confirm button.
                // Ideally, we should use a "SignalHandlerId" to disconnect, but in GTK4-rs/closure world it's tricky.
                // A simpler way: The confirm button logic is generic. It uses `current_var` (which we set before showing).

                // Wait, I need to update the logic for `delete_confirm_btn`.
                // It should read `current_var`.
                // So I should define `delete_confirm_btn` click handler ONCE in `create_variables_page` (outside loop),
                // and here just update `current_var` and show the stack.

                // But `refresh_list_ui` is called repeatedly.
                // If I define the handler inside `create_variables_page`, I need `refresh_list_ui` to be callable from there.
                // Yes, `refresh` callback captures everything.

                // So:
                // 1. In `create_variables_page`, set up `delete_confirm_btn.connect_clicked`.
                //    It reads `current_var`, calls `inline_variable_references`, then `delete_variable`.
                //    Then calls `refresh()`.

                // 2. Here in `delete_btn` (row), we:
                //    Check usages.
                //    If > 0:
                //       Set `current_var` to `var_del`.
                //       Update `warning_body` text.
                //       Switch stack to `delete_confirm`.
                //    If == 0:
                //       Inline confirm logic (existing).

                delete_btn.connect_clicked(move |btn| {
                    let name_clean = var_del.name.trim_start_matches('$');

                    // Check usages
                    let usage_count = match parser::count_variable_references(name_clean) {
                         Ok(c) => c,
                         Err(_) => 0,
                    };

                    if usage_count > 0 {
                         // Dependent Case
                         *current_var_r.borrow_mut() = Some(var_del.clone());
                         warning_body_r.set_label(&format!(
                             "Variable '{}' is used in {} places.\nDeleting it will replace these references with '{}'.\n\nDo you want to proceed?",
                             var_del.name, usage_count, var_del.value
                         ));
                         stack_r.set_visible_child_name("delete_confirm");
                    } else {
                        // Non-dependent Case: Inline Confirm
                        let mut state = confirm_state.borrow_mut();
                        if *state {
                            // Backup before modifying
                            if let Err(e) = crate::ui::utils::backup::perform_backup(false) {
                                eprintln!("Failed to backup config: {}", e);
                            }

                            match parser::delete_variable(var_del.file_path.clone(), var_del.line_number) {
                                Ok(_) => {
                                    toast_c("Variable deleted".to_string());
                                    refresh_list_ui(
                                        &list_box_r,
                                        &search_entry_r,
                                        &stack_r,
                                        &current_var_r,
                                        &name_entry_r,
                                        &value_entry_r,
                                        &toast_r,
                                        &update_title_r,
                                        &warning_body_r,
                                        &delete_confirm_btn_r,
                                        &refactor_switch_r,
                                    );
                                }
                                Err(e) => {
                                    toast_c(format!("Error: {}", e));
                                    // Reset confirmation state on error
                                    *state = false;
                                    btn.add_css_class("flat");
                                    btn.set_icon_name("user-trash-symbolic");
                                    btn.set_label("");
                                    btn.set_tooltip_text(Some("Delete"));
                                }
                            }
                        } else {
                            *state = true;
                            btn.remove_css_class("flat");
                            btn.set_icon_name("edit-delete-symbolic");
                            btn.set_label("Confirm");
                            btn.set_tooltip_text(Some("Click again to confirm delete"));
                        }
                    }
                });
            }
        }
        Err(e) => {
            let row = adw::ActionRow::builder()
                .title(format!("Error: {}", e))
                .css_classes(vec!["error".to_string()])
                .build();
            list_box.append(&row);
        }
    }
}
