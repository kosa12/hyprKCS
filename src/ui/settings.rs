use crate::config::StyleConfig;
use crate::parser::input::{load_input_config, save_input_config};
use gtk::gio;
use gtk::glib;
use gtk4 as gtk;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_settings_view(
    window: &adw::ApplicationWindow,
    stack: &gtk::Stack,
    model: &gio::ListStore,
    on_desc_toggle: Rc<dyn Fn(bool)>,
    on_fav_toggle: Rc<dyn Fn(bool)>,
    on_args_toggle: Rc<dyn Fn(bool)>,
    on_submap_toggle: Rc<dyn Fn(bool)>,
    on_sort_change: Rc<dyn Fn(String)>,
    on_show_toast: Rc<dyn Fn(String)>,
) -> gtk::Widget {
    let config = Rc::new(RefCell::new(StyleConfig::load()));

    let main_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    // --- Header ---
    let header = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let back_btn = gtk::Button::builder()
        .icon_name("go-previous-symbolic")
        .css_classes(["flat", "circular"])
        .tooltip_text("Back")
        .build();

    let stack_c = stack.clone();
    back_btn.connect_clicked(move |_| {
        stack_c.set_visible_child_name("home");
    });

    let title = gtk::Label::builder()
        .label("Settings")
        .css_classes(["title-2"])
        .build();

    header.append(&back_btn);
    header.append(&title);
    main_box.append(&header);
    main_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    // --- Sidebar Layout ---
    let sidebar_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .vexpand(true)
        .build();

    let settings_stack = gtk::Stack::builder()
        .transition_type(gtk::StackTransitionType::None)
        .vexpand(true)
        .hexpand(true)
        .build();

    let sidebar = gtk::StackSidebar::builder()
        .stack(&settings_stack)
        .vexpand(true)
        .width_request(200) // Sidebar width
        .build();

    let sidebar_scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&sidebar)
        .build();

    sidebar_box.append(&sidebar_scroll);
    sidebar_box.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    sidebar_box.append(&settings_stack);

    main_box.append(&sidebar_box);

    // ================== PAGE 1: GENERAL ==================
    let page_general = adw::PreferencesPage::builder().build();
    let group_backup = adw::PreferencesGroup::builder()
        .title("Backup and Restore")
        .build();

    // Auto-Backup
    let auto_backup_switch = gtk::Switch::builder()
        .active(config.borrow().auto_backup)
        .valign(gtk::Align::Center)
        .build();
    let auto_backup_row = adw::ActionRow::builder()
        .title("Auto-Backup")
        .subtitle("Backup config on every save")
        .activatable_widget(&auto_backup_switch)
        .build();
    auto_backup_row.add_suffix(&auto_backup_switch);
    let c = config.clone();
    auto_backup_switch.connect_state_set(move |_, s| {
        c.borrow_mut().auto_backup = s;
        let _ = c.borrow().save();
        glib::Propagation::Proceed
    });
    group_backup.add(&auto_backup_row);

    // Max Backups Enabled
    let max_backups_switch = gtk::Switch::builder()
        .active(config.borrow().max_backups_enabled)
        .valign(gtk::Align::Center)
        .build();
    let max_backups_row = adw::ActionRow::builder()
        .title("Limit Backups")
        .subtitle("Delete old backups")
        .activatable_widget(&max_backups_switch)
        .build();
    max_backups_row.add_suffix(&max_backups_switch);

    // Backup Count
    let count_adj = gtk::Adjustment::new(
        config.borrow().max_backups_count as f64,
        1.0,
        1000.0,
        1.0,
        10.0,
        0.0,
    );
    let count_spin = gtk::SpinButton::builder()
        .adjustment(&count_adj)
        .valign(gtk::Align::Center)
        .build();
    let count_row = adw::ActionRow::builder()
        .title("Max Backups")
        .subtitle("Number of backups to keep")
        .build();
    count_row.add_suffix(&count_spin);
    let c = config.clone();
    count_spin.connect_value_changed(move |s| {
        c.borrow_mut().max_backups_count = s.value() as i32;
        let _ = c.borrow().save();
    });

    // Initial state
    count_row.set_sensitive(config.borrow().max_backups_enabled);

    let count_row_ref = count_row.clone();
    let c = config.clone();
    max_backups_switch.connect_state_set(move |_, s| {
        c.borrow_mut().max_backups_enabled = s;
        let _ = c.borrow().save();
        count_row_ref.set_sensitive(s);
        glib::Propagation::Proceed
    });
    group_backup.add(&max_backups_row);
    group_backup.add(&count_row);

    page_general.add(&group_backup);

    // Export Group
    let group_export = adw::PreferencesGroup::builder().title("Export").build();

    let export_row = adw::ActionRow::builder()
        .title("Export Keybinds")
        .subtitle("Save all keybinds to a Markdown file")
        .activatable(true)
        .build();
    let export_icon = gtk::Image::from_icon_name("document-save-as-symbolic");
    export_row.add_prefix(&export_icon);

    let suffix = gtk::Image::from_icon_name("go-next-symbolic");
    export_row.add_suffix(&suffix);

    let window_c = window.clone();
    let model_c = model.clone();
    let toast_cb = on_show_toast.clone();

    export_row.connect_activated(move |_| {
        let file_dialog = gtk::FileDialog::builder()
            .title("Export Keybinds")
            .accept_label("Export")
            .initial_name("keybinds.md")
            .build();

        let m = model_c.clone();
        let t_cb = toast_cb.clone();
        file_dialog.save(
            Some(&window_c),
            None::<&gtk::gio::Cancellable>,
            move |res| {
                match res {
                    Ok(file) => {
                        if let Some(path) = file.path() {
                            match crate::ui::utils::export_keybinds_to_markdown(&m, &path) {
                                Ok(_) => t_cb(format!("Successfully exported to {:?}", path)),
                                Err(e) => t_cb(format!("Export failed: {}", e)),
                            }
                        }
                    }
                    Err(e) => {
                        // User cancelled usually
                        println!("Export cancelled/error: {}", e);
                    }
                }
            },
        );
    });
    group_export.add(&export_row);
    page_general.add(&group_export);

    settings_stack.add_titled(&page_general, Some("general"), "General");

    // ================== PAGE 2: WINDOW ==================
    let page_window = adw::PreferencesPage::builder().build();
    let group_dims = adw::PreferencesGroup::builder().title("Dimensions").build();

    // Width
    let width_adj = gtk::Adjustment::new(
        config.borrow().width as f64,
        100.0,
        3840.0,
        10.0,
        100.0,
        0.0,
    );
    let width_spin = gtk::SpinButton::builder()
        .adjustment(&width_adj)
        .valign(gtk::Align::Center)
        .build();
    let width_row = adw::ActionRow::builder()
        .title("Window Width")
        .subtitle("px")
        .build();
    width_row.add_suffix(&width_spin);
    let c = config.clone();
    let window_c = window.clone();
    width_spin.connect_value_changed(move |s| {
        let val = s.value() as i32;
        {
            c.borrow_mut().width = val;
            let _ = c.borrow().save();
        }
        let h = c.borrow().height;
        window_c.set_default_size(val, h);
        window_c.set_size_request(val, h);
        crate::ui::style::reload_style();
    });
    group_dims.add(&width_row);

    // Height
    let height_adj = gtk::Adjustment::new(
        config.borrow().height as f64,
        100.0,
        2160.0,
        10.0,
        100.0,
        0.0,
    );
    let height_spin = gtk::SpinButton::builder()
        .adjustment(&height_adj)
        .valign(gtk::Align::Center)
        .build();
    let height_row = adw::ActionRow::builder()
        .title("Window Height")
        .subtitle("px")
        .build();
    height_row.add_suffix(&height_spin);
    let c = config.clone();
    let window_c = window.clone();
    height_spin.connect_value_changed(move |s| {
        let val = s.value() as i32;
        {
            c.borrow_mut().height = val;
            let _ = c.borrow().save();
        }
        let w = c.borrow().width;
        window_c.set_default_size(w, val);
        window_c.set_size_request(w, val);
        crate::ui::style::reload_style();
    });
    group_dims.add(&height_row);

    // Monitor Margin
    let margin_adj = gtk::Adjustment::new(
        config.borrow().monitor_margin as f64,
        0.0,
        500.0,
        1.0,
        10.0,
        0.0,
    );
    let margin_spin = gtk::SpinButton::builder()
        .adjustment(&margin_adj)
        .valign(gtk::Align::Center)
        .build();
    let margin_row = adw::ActionRow::builder()
        .title("Monitor Margin")
        .subtitle("Spacing from screen edges (px)")
        .build();
    margin_row.add_suffix(&margin_spin);
    let c = config.clone();
    margin_spin.connect_value_changed(move |s| {
        c.borrow_mut().monitor_margin = s.value() as i32;
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_dims.add(&margin_row);

    // Row Padding
    let pad_adj =
        gtk::Adjustment::new(config.borrow().row_padding as f64, 0.0, 50.0, 1.0, 5.0, 0.0);
    let pad_spin = gtk::SpinButton::builder()
        .adjustment(&pad_adj)
        .valign(gtk::Align::Center)
        .build();
    let pad_row = adw::ActionRow::builder()
        .title("Row Padding")
        .subtitle("Spacing between list rows (px)")
        .build();
    pad_row.add_suffix(&pad_spin);
    let c = config.clone();
    pad_spin.connect_value_changed(move |s| {
        c.borrow_mut().row_padding = s.value() as i32;
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_dims.add(&pad_row);

    let group_style = adw::PreferencesGroup::builder()
        .title("Window Style")
        .build();

    // Opacity
    let op_adj = gtk::Adjustment::new(
        config.borrow().opacity.unwrap_or(1.0),
        0.1,
        1.0,
        0.05,
        0.1,
        0.0,
    );
    let op_spin = gtk::SpinButton::builder()
        .adjustment(&op_adj)
        .digits(2)
        .valign(gtk::Align::Center)
        .build();
    let op_row = adw::ActionRow::builder()
        .title("Opacity")
        .subtitle("0.0 - 1.0")
        .build();
    op_row.add_suffix(&op_spin);
    let c = config.clone();
    op_spin.connect_value_changed(move |s| {
        c.borrow_mut().opacity = Some(s.value());
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_style.add(&op_row);

    // Shadow Size
    let shadow_entry = gtk::Entry::builder()
        .text(&config.borrow().shadow_size)
        .valign(gtk::Align::Center)
        .width_chars(25)
        .build();
    let shadow_row = adw::ActionRow::builder()
        .title("Shadow Size")
        .subtitle("CSS box-shadow format")
        .build();
    shadow_row.add_suffix(&shadow_entry);
    let c = config.clone();
    shadow_entry.connect_changed(move |e| {
        c.borrow_mut().shadow_size = e.text().to_string();
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_style.add(&shadow_row);

    page_window.add(&group_dims);
    page_window.add(&group_style);
    settings_stack.add_titled(&page_window, Some("window"), "Window");

    // ================== PAGE 2.5: INPUT ==================
    let page_input = adw::PreferencesPage::builder().build();

    let group_info = adw::PreferencesGroup::builder()
        .title("Input Configuration")
        .build();
    page_input.add(&group_info);

    let group_kb = adw::PreferencesGroup::builder().title("Keyboard").build();

    let (input_config, gestures_config) = match load_input_config() {
        Ok((i, g)) => (Rc::new(RefCell::new(i)), Rc::new(RefCell::new(g))),
        Err(e) => {
            eprintln!("Failed to load input/gestures config: {}", e);
            (
                Rc::new(RefCell::new(crate::parser::input::InputConfig::default())),
                Rc::new(RefCell::new(crate::parser::input::GesturesConfig::default())),
            )
        }
    };

    let layout_entry = gtk::Entry::builder()
        .text(&input_config.borrow().kb_layout)
        .valign(gtk::Align::Center)
        .build();
    let layout_row = adw::ActionRow::builder()
        .title("Layout")
        .subtitle("Keyboard layout code (e.g. us, hu, jp)")
        .build();
    layout_row.add_suffix(&layout_entry);
    let c = input_config.clone();
    layout_entry.connect_changed(move |e| {
        c.borrow_mut().kb_layout = e.text().to_string();
    });
    group_kb.add(&layout_row);

    // Keyboard Variant
    let variant_entry = gtk::Entry::builder()
        .text(&input_config.borrow().kb_variant)
        .valign(gtk::Align::Center)
        .build();
    let variant_row = adw::ActionRow::builder()
        .title("Variant")
        .subtitle("Layout variant (e.g. intl, abnt2)")
        .build();
    variant_row.add_suffix(&variant_entry);
    let c = input_config.clone();
    variant_entry.connect_changed(move |e| {
        c.borrow_mut().kb_variant = e.text().to_string();
    });
    group_kb.add(&variant_row);

    let options_entry = gtk::Entry::builder()
        .text(&input_config.borrow().kb_options)
        .valign(gtk::Align::Center)
        .tooltip_text("e.g. grp:alt_shift_toggle")
        .build();
    let options_row = adw::ActionRow::builder()
        .title("Options")
        .subtitle("XKB options (e.g. caps:escape, grp:alt_shift_toggle)")
        .build();
    options_row.add_suffix(&options_entry);
    let c = input_config.clone();
    options_entry.connect_changed(move |e| {
        c.borrow_mut().kb_options = e.text().to_string();
    });
    group_kb.add(&options_row);

    // Repeat Rate
    let rate_adj = gtk::Adjustment::new(
        input_config.borrow().repeat_rate as f64,
        1.0,
        200.0,
        1.0,
        10.0,
        0.0,
    );
    let rate_spin = gtk::SpinButton::builder()
        .adjustment(&rate_adj)
        .valign(gtk::Align::Center)
        .build();
    let rate_row = adw::ActionRow::builder()
        .title("Repeat Rate")
        .subtitle("Key repeats per second")
        .build();
    rate_row.add_suffix(&rate_spin);
    let c = input_config.clone();
    rate_spin.connect_value_changed(move |s| {
        c.borrow_mut().repeat_rate = s.value() as i32;
    });
    group_kb.add(&rate_row);

    let delay_adj = gtk::Adjustment::new(
        input_config.borrow().repeat_delay as f64,
        100.0,
        2000.0,
        50.0,
        100.0,
        0.0,
    );
    let delay_spin = gtk::SpinButton::builder()
        .adjustment(&delay_adj)
        .valign(gtk::Align::Center)
        .build();
    let delay_row = adw::ActionRow::builder()
        .title("Repeat Delay")
        .subtitle("Delay before repeat starts (ms)")
        .build();
    delay_row.add_suffix(&delay_spin);
    let c = input_config.clone();
    delay_spin.connect_value_changed(move |s| {
        c.borrow_mut().repeat_delay = s.value() as i32;
    });
    group_kb.add(&delay_row);

    page_input.add(&group_kb);

    let group_mouse = adw::PreferencesGroup::builder()
        .title("Mouse / Touchpad")
        .build();

    let follow_combo =
        gtk::DropDown::from_strings(&["0 - Disabled", "1 - Always", "2 - Cursor", "3 - Loose"]);
    follow_combo.set_selected(input_config.borrow().follow_mouse as u32);
    follow_combo.set_valign(gtk::Align::Center);
    let follow_row = adw::ActionRow::builder()
        .title("Follow Mouse")
        .subtitle("How window focus follows the mouse cursor")
        .build();
    follow_row.add_suffix(&follow_combo);
    let c = input_config.clone();
    follow_combo.connect_selected_notify(move |d| {
        c.borrow_mut().follow_mouse = d.selected() as i32;
    });
    group_mouse.add(&follow_row);

    // Sensitivity
    let sens_adj =
        gtk::Adjustment::new(input_config.borrow().sensitivity, -1.0, 1.0, 0.1, 0.2, 0.0);
    let sens_scale = gtk::Scale::builder()
        .adjustment(&sens_adj)
        .draw_value(true)
        .hexpand(true)
        .build();
    let sens_row = adw::ActionRow::builder()
        .title("Sensitivity")
        .subtitle("(-1.0 to 1.0)")
        .build();
    let sens_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    sens_box.set_width_request(150);
    sens_box.append(&sens_scale);
    sens_row.add_suffix(&sens_box);
    let c = input_config.clone();
    sens_scale.connect_value_changed(move |s| {
        c.borrow_mut().sensitivity = s.value();
    });
    group_mouse.add(&sens_row);

    page_input.add(&group_mouse);

    // Save Button
    let group_save = adw::PreferencesGroup::new();
    let save_row = adw::ActionRow::builder()
        .title("Save Input Configuration")
        .subtitle("Writes changes to hyprland.conf")
        .activatable(true)
        .build();
    let save_icon = gtk::Image::from_icon_name("document-save-symbolic");
    save_row.add_prefix(&save_icon);

    let c = input_config.clone();
    let g = gestures_config.clone();
    let toast_cb_input = on_show_toast.clone();
    save_row.connect_activated(move |_| match save_input_config(&c.borrow(), &g.borrow()) {
        Ok(_) => toast_cb_input("Input configuration saved successfully".to_string()),
        Err(e) => toast_cb_input(format!("Error saving config: {}", e)),
    });
    group_save.add(&save_row);
    page_input.add(&group_save);

    settings_stack.add_titled(&page_input, Some("input"), "Input");

    // ================== PAGE 2.6: GESTURES ==================
    let page_gestures = adw::PreferencesPage::builder().build();
    let group_swipe = adw::PreferencesGroup::builder()
        .title("Workspace Swipe")
        .description("Configure standard workspace swipe gesture (Hyprland v0.51+)")
        .build();

    // Enable Swipe
    let swipe_switch = gtk::Switch::builder()
        .active(gestures_config.borrow().workspace_swipe)
        .valign(gtk::Align::Center)
        .build();
    let swipe_row = adw::ActionRow::builder()
        .title("Enable Swipe")
        .subtitle("gesture = fingers, horizontal, workspace")
        .activatable_widget(&swipe_switch)
        .build();
    swipe_row.add_suffix(&swipe_switch);
    let c = gestures_config.clone();
    swipe_switch.connect_state_set(move |_, s| {
        c.borrow_mut().workspace_swipe = s;
        glib::Propagation::Proceed
    });
    group_swipe.add(&swipe_row);

    // Fingers
    let fingers_adj = gtk::Adjustment::new(
        gestures_config.borrow().workspace_swipe_fingers as f64,
        1.0,
        5.0,
        1.0,
        1.0,
        0.0,
    );
    let fingers_spin = gtk::SpinButton::builder()
        .adjustment(&fingers_adj)
        .valign(gtk::Align::Center)
        .build();
    let fingers_row = adw::ActionRow::builder()
        .title("Fingers")
        .subtitle("Number of fingers")
        .build();
    fingers_row.add_suffix(&fingers_spin);
    let c = gestures_config.clone();
    fingers_spin.connect_value_changed(move |s| {
        c.borrow_mut().workspace_swipe_fingers = s.value() as i32;
    });
    group_swipe.add(&fingers_row);

    page_gestures.add(&group_swipe);

    // Note about deprecated settings
    let group_note = adw::PreferencesGroup::builder()
        .title("Note")
        .build();
    let note_row = adw::ActionRow::builder()
        .title("Advanced Settings Removed")
        .subtitle("Hyprland v0.51+ replaced the 'gestures' block with specific 'gesture' bindings. Fine-grained controls like speed, cancel ratio, and inversion are no longer available as global variables.")
        .build();
    group_note.add(&note_row);
    page_gestures.add(&group_note);

    // Save Button for Gestures
    let group_save_gestures = adw::PreferencesGroup::new();
    let save_gestures_row = adw::ActionRow::builder()
        .title("Save Gesture Configuration")
        .subtitle("Writes 'gesture = ...' to hyprland.conf")
        .activatable(true)
        .build();
    let save_icon_g = gtk::Image::from_icon_name("document-save-symbolic");
    save_gestures_row.add_prefix(&save_icon_g);

    let c = input_config.clone();
    let g = gestures_config.clone();
    let toast_cb_g = on_show_toast.clone();
    save_gestures_row.connect_activated(move |_| match save_input_config(&c.borrow(), &g.borrow()) {
        Ok(_) => toast_cb_g("Gesture configuration saved successfully".to_string()),
        Err(e) => toast_cb_g(format!("Error saving config: {}", e)),
    });
    group_save_gestures.add(&save_gestures_row);
    page_gestures.add(&group_save_gestures);

    settings_stack.add_titled(&page_gestures, Some("gestures"), "Gestures");

    // ================== PAGE 3: APPEARANCE ==================
    let page_app = adw::PreferencesPage::builder().build();
    let group_font = adw::PreferencesGroup::builder()
        .title("Typography and Borders")
        .build();

    // Font Size
    let font_entry = gtk::Entry::builder()
        .text(config.borrow().font_size.as_deref().unwrap_or("0.9rem"))
        .valign(gtk::Align::Center)
        .width_chars(10)
        .build();
    let font_row = adw::ActionRow::builder()
        .title("Font Size")
        .subtitle("CSS value (e.g. 12px, 1rem)")
        .build();
    font_row.add_suffix(&font_entry);
    let c = config.clone();
    font_entry.connect_changed(move |e| {
        c.borrow_mut().font_size = Some(e.text().to_string());
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_font.add(&font_row);

    // Border Size
    let b_size_entry = gtk::Entry::builder()
        .text(config.borrow().border_size.as_deref().unwrap_or("1px"))
        .valign(gtk::Align::Center)
        .width_chars(10)
        .build();
    let b_size_row = adw::ActionRow::builder()
        .title("Border Size")
        .subtitle("CSS value (e.g. 2px)")
        .build();
    b_size_row.add_suffix(&b_size_entry);
    let c = config.clone();
    b_size_entry.connect_changed(move |e| {
        c.borrow_mut().border_size = Some(e.text().to_string());
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_font.add(&b_size_row);

    // Border Radius
    let b_rad_entry = gtk::Entry::builder()
        .text(config.borrow().border_radius.as_deref().unwrap_or("12px"))
        .valign(gtk::Align::Center)
        .width_chars(10)
        .build();
    let b_rad_row = adw::ActionRow::builder()
        .title("Border Radius")
        .subtitle("CSS value (e.g. 10px)")
        .build();
    b_rad_row.add_suffix(&b_rad_entry);
    let c = config.clone();
    b_rad_entry.connect_changed(move |e| {
        c.borrow_mut().border_radius = Some(e.text().to_string());
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_font.add(&b_rad_row);

    // Keyboard Layout
    let layout_opts = ["ANSI", "ISO", "JIS", "ABNT2", "Hungarian", "Ortholinear"];
    let layout_list = gtk::StringList::new(&layout_opts);

    // Map current string to index
    let current_layout = config.borrow().keyboard_layout.to_uppercase();
    let layout_idx = match current_layout.as_str() {
        "ISO" => 1,
        "JIS" => 2,
        "ABNT2" => 3,
        "HU" | "HUNGARIAN" => 4,
        "ORTHO" | "ORTHOLINEAR" => 5,
        _ => 0,
    };

    let layout_drop = gtk::DropDown::builder()
        .model(&layout_list)
        .selected(layout_idx)
        .valign(gtk::Align::Center)
        .build();
    let layout_row = adw::ActionRow::builder()
        .title("Keyboard Layout")
        .subtitle("Visual keyboard map type")
        .build();
    layout_row.add_suffix(&layout_drop);

    let c = config.clone();
    layout_drop.connect_selected_notify(move |d| {
        let val = match d.selected() {
            1 => "ISO",
            2 => "JIS",
            3 => "ABNT2",
            4 => "HUNGARIAN",
            5 => "ORTHOLINEAR",
            _ => "ANSI",
        };
        c.borrow_mut().keyboard_layout = val.to_string();
        let _ = c.borrow().save();
    });
    group_font.add(&layout_row);

    // Alternating Colors
    let alt_switch = gtk::Switch::builder()
        .active(config.borrow().alternating_row_colors)
        .valign(gtk::Align::Center)
        .build();
    let alt_row = adw::ActionRow::builder()
        .title("Alternating Row Colors")
        .activatable_widget(&alt_switch)
        .build();
    alt_row.add_suffix(&alt_switch);
    let c = config.clone();
    alt_switch.connect_state_set(move |_, s| {
        c.borrow_mut().alternating_row_colors = s;
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
        glib::Propagation::Proceed
    });
    group_font.add(&alt_row);

    page_app.add(&group_font);
    settings_stack.add_titled(&page_app, Some("appearance"), "Appearance");

    // ================== PAGE 4: UI ELEMENTS ==================
    let page_ui = adw::PreferencesPage::builder().build();
    let group_cols = adw::PreferencesGroup::builder()
        .title("Table Columns")
        .build();

    // Submaps
    let sub_switch = gtk::Switch::builder()
        .active(config.borrow().show_submaps)
        .valign(gtk::Align::Center)
        .build();
    let sub_row = adw::ActionRow::builder()
        .title("Show Submaps")
        .activatable_widget(&sub_switch)
        .build();
    sub_row.add_suffix(&sub_switch);
    let c = config.clone();
    let on_sub = on_submap_toggle.clone();
    sub_switch.connect_state_set(move |_, s| {
        c.borrow_mut().show_submaps = s;
        let _ = c.borrow().save();
        on_sub(s);
        glib::Propagation::Proceed
    });
    group_cols.add(&sub_row);

    // Args
    let args_switch = gtk::Switch::builder()
        .active(config.borrow().show_args)
        .valign(gtk::Align::Center)
        .build();
    let args_row = adw::ActionRow::builder()
        .title("Show Arguments")
        .activatable_widget(&args_switch)
        .build();
    args_row.add_suffix(&args_switch);
    let c = config.clone();
    let on_args = on_args_toggle.clone();
    args_switch.connect_state_set(move |_, s| {
        c.borrow_mut().show_args = s;
        let _ = c.borrow().save();
        on_args(s);
        glib::Propagation::Proceed
    });
    group_cols.add(&args_row);

    // Favorites
    let fav_switch = gtk::Switch::builder()
        .active(config.borrow().show_favorites)
        .valign(gtk::Align::Center)
        .build();
    let fav_row = adw::ActionRow::builder()
        .title("Show Favorites")
        .activatable_widget(&fav_switch)
        .build();
    fav_row.add_suffix(&fav_switch);
    let c = config.clone();
    let on_fav = on_fav_toggle.clone();
    fav_switch.connect_state_set(move |_, s| {
        c.borrow_mut().show_favorites = s;
        let _ = c.borrow().save();
        on_fav(s);
        glib::Propagation::Proceed
    });
    group_cols.add(&fav_row);

    // Description
    let desc_switch = gtk::Switch::builder()
        .active(config.borrow().show_description)
        .valign(gtk::Align::Center)
        .build();
    let desc_row = adw::ActionRow::builder()
        .title("Show Description")
        .activatable_widget(&desc_switch)
        .build();
    desc_row.add_suffix(&desc_switch);
    let c = config.clone();
    let on_toggle = on_desc_toggle.clone();
    desc_switch.connect_state_set(move |_, s| {
        c.borrow_mut().show_description = s;
        let _ = c.borrow().save();
        on_toggle(s);
        glib::Propagation::Proceed
    });
    group_cols.add(&desc_row);

    let group_sort = adw::PreferencesGroup::builder().title("Sorting").build();

    // Default Sort
    let sort_opts = ["Key", "Modifiers", "Action", "Arguments", "Submap"];
    let sort_list = gtk::StringList::new(&sort_opts);

    // Map current string to index
    let current_sort = config.borrow().default_sort.to_lowercase();
    let selected_idx = if current_sort.contains("mod") {
        1
    } else if current_sort.contains("disp") || current_sort.contains("action") {
        2
    } else if current_sort.contains("arg") {
        3
    } else if current_sort.contains("sub") {
        4
    } else {
        0
    }; // Default Key

    let sort_drop = gtk::DropDown::builder()
        .model(&sort_list)
        .selected(selected_idx)
        .valign(gtk::Align::Center)
        .build();
    let sort_row = adw::ActionRow::builder()
        .title("Default Sort Column")
        .build();
    sort_row.add_suffix(&sort_drop);

    let c = config.clone();
    let on_sort = on_sort_change.clone();
    sort_drop.connect_selected_notify(move |d| {
        let val = match d.selected() {
            1 => "mods",
            2 => "dispatcher",
            3 => "args",
            4 => "submap",
            _ => "key",
        };
        c.borrow_mut().default_sort = val.to_string();
        let _ = c.borrow().save();
        on_sort(val.to_string());
    });
    group_sort.add(&sort_row);

    page_ui.add(&group_cols);
    page_ui.add(&group_sort);
    settings_stack.add_titled(&page_ui, Some("ui"), "UI Elements");

    // ================== PAGE 5: FEEDBACK ==================
    let page_feedback = adw::PreferencesPage::builder().build();
    let group_community = adw::PreferencesGroup::builder().title("Community").build();

    let create_link = |title: &str, subtitle: &str, icon: &str, url: &str| {
        let row = adw::ActionRow::builder()
            .title(title)
            .subtitle(subtitle)
            .activatable(true)
            .build();

        let img = gtk::Image::from_icon_name(icon);
        row.add_prefix(&img);

        let suffix = gtk::Image::from_icon_name("external-link-symbolic");
        row.add_suffix(&suffix);

        let u = url.to_string();
        let w = window.clone();
        row.connect_activated(move |_| {
            let launcher = gtk::UriLauncher::new(&u);
            launcher.launch(Some(&w), None::<&gtk::gio::Cancellable>, |res| {
                if let Err(e) = res {
                    eprintln!("Failed to launch URL: {}", e);
                }
            });
        });
        row
    };

    group_community.add(&create_link(
        "GitHub Repository",
        "Star the project on GitHub!",
        "starred-symbolic",
        "https://github.com/kosa12/hyprKCS",
    ));
    group_community.add(&create_link(
        "Report a Bug or Suggest a Feature",
        "Found an issue? Have a suggestion? Let me know.",
        "dialog-warning-symbolic",
        "https://github.com/kosa12/hyprKCS/issues",
    ));
    group_community.add(&create_link(
        "Donate",
        "Support the project on Ko-fi",
        "emblem-favorite-symbolic",
        "https://ko-fi.com/kosa12",
    ));
    group_community.add(&create_link(
        "Donate",
        "Support the project on Github Sponsors",
        "emblem-favorite-symbolic",
        "https://github.com/sponsors/kosa12",
    ));

    page_feedback.add(&group_community);
    settings_stack.add_titled(&page_feedback, Some("feedback"), "Feedback");

    // ================== PAGE 6: ABOUT ==================
    let page_about = adw::PreferencesPage::builder().build();
    let group_about = adw::PreferencesGroup::builder()
        .title("Application Information")
        .build();

    let ver_row = adw::ActionRow::builder()
        .title("Version")
        .subtitle(env!("CARGO_PKG_VERSION"))
        .build();
    let ver_img = gtk::Image::from_icon_name("help-about-symbolic");
    ver_row.add_prefix(&ver_img);
    group_about.add(&ver_row);

    let dev_row = adw::ActionRow::builder()
        .title("Developer")
        .subtitle("kosa12")
        .build();
    let dev_img = gtk::Image::from_icon_name("avatar-default-symbolic");
    dev_row.add_prefix(&dev_img);
    group_about.add(&dev_row);

    let lic_row = adw::ActionRow::builder()
        .title("License")
        .subtitle("MIT")
        .build();
    let lic_img = gtk::Image::from_icon_name("dialog-information-symbolic");
    lic_row.add_prefix(&lic_img);
    group_about.add(&lic_row);

    let group_links = adw::PreferencesGroup::builder().title("Links").build();
    group_links.add(&create_link(
        "Source Code",
        "View on GitHub",
        "document-properties-symbolic",
        "https://github.com/kosa12/hyprKCS",
    ));
    group_links.add(&create_link(
        "Wiki",
        "Documentation and Guides",
        "system-help-symbolic",
        "https://github.com/kosa12/hyprKCS/wiki",
    ));

    page_about.add(&group_about);
    page_about.add(&group_links);
    settings_stack.add_titled(&page_about, Some("about"), "About");

    main_box.upcast::<gtk::Widget>()
}
