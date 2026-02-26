use crate::config::hud::{
    get_hud_pid_path, is_hud_running, load_hud_config, save_hud_config, HudKeybind, HudPosition,
};
use crate::keybind_object::KeybindObject;
use gtk::{gio, glib};
use gtk4 as gtk;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::fs;
use std::process::Command;
use std::rc::Rc;

pub fn create_hud_page(model: &gio::ListStore, on_show_toast: Rc<dyn Fn(String)>) -> gtk::Widget {
    let main_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    let config = Rc::new(RefCell::new(load_hud_config()));

    // --- Single Line Header ---
    let header_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let search_entry = gtk::SearchEntry::builder()
        .placeholder_text("Search keybinds...")
        .hexpand(true)
        .build();

    // Position Dropdown (Minimal version for single line)
    let pos_model = gtk::StringList::new(&["Top Right", "Top Left", "Bottom Right", "Bottom Left"]);

    let initial_pos = match config.borrow().position {
        HudPosition::TopRight => 0,
        HudPosition::TopLeft => 1,
        HudPosition::BottomRight => 2,
        HudPosition::BottomLeft => 3,
    };

    let pos_dropdown = gtk::DropDown::builder()
        .model(&pos_model)
        .selected(initial_pos)
        .valign(gtk::Align::Center)
        .build();

    let config_pos_ref = Rc::clone(&config);
    pos_dropdown.connect_selected_item_notify(move |row| {
        let mut cfg = config_pos_ref.borrow_mut();
        cfg.position = match row.selected() {
            0 => HudPosition::TopRight,
            1 => HudPosition::TopLeft,
            2 => HudPosition::BottomRight,
            3 => HudPosition::BottomLeft,
            _ => HudPosition::TopRight,
        };
        let _ = save_hud_config(&cfg);
    });

    // Check if process is actually running to determine switch state
    let is_running = is_hud_running();
    if config.borrow().enabled != is_running {
        // Mismatch detected. Sync config to reality (if dead, mark disabled).
        // If we want to auto-restart, we'd do it here, but safer to default to "Off" if dead.
        config.borrow_mut().enabled = is_running;
        let _ = save_hud_config(&config.borrow());
    }

    let enable_switch = gtk::Switch::builder()
        .active(is_running)
        .valign(gtk::Align::Center)
        .build();

    let config_ref = Rc::clone(&config);
    let toast_cb = Rc::clone(&on_show_toast);
    enable_switch.connect_state_set(move |switch, state| {
        config_ref.borrow_mut().enabled = state;
        let _ = save_hud_config(&config_ref.borrow());

        if state {
            if let Ok(exe) = std::env::current_exe() {
                use std::os::unix::process::CommandExt;
                let mut cmd = Command::new(exe);
                cmd.arg("--hud");
                unsafe {
                    cmd.pre_exec(|| {
                        let _ = libc::setsid();
                        Ok(())
                    });
                }
                match cmd.spawn() {
                    Ok(_) => toast_cb("HUD Enabled".into()),
                    Err(e) => {
                        eprintln!("Failed to spawn HUD: {}", e);
                        toast_cb(format!("Failed to start HUD: {}", e));

                        // Revert config and UI
                        config_ref.borrow_mut().enabled = false;
                        let _ = save_hud_config(&config_ref.borrow());
                        switch.set_active(false);
                    }
                }
            } else {
                toast_cb("Failed to locate executable".into());
            }
        } else {
            if let Some(pid_path) = get_hud_pid_path() {
                if let Ok(pid_str) = fs::read_to_string(&pid_path) {
                    if let Ok(pid) = pid_str.trim().parse::<i32>() {
                        unsafe {
                            if libc::kill(pid, libc::SIGTERM) != 0 {
                                eprintln!("Failed to kill HUD process (PID: {})", pid);
                            }
                        }
                    }
                }
                let _ = fs::remove_file(pid_path);
            }
            toast_cb("HUD Disabled".into());
        }
        glib::Propagation::Proceed
    });

    header_box.append(&enable_switch);
    header_box.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    header_box.append(&search_entry);
    header_box.append(&pos_dropdown);

    main_box.append(&header_box);
    main_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    // --- Styling Section (Compact) ---
    let styling_clamp = adw::Clamp::builder().maximum_size(800).build();
    let styling_group = adw::PreferencesGroup::builder()
        .margin_start(12)
        .margin_end(12)
        .margin_top(6)
        .build();

    let expander = adw::ExpanderRow::builder()
        .title("HUD Appearance")
        .subtitle("Customize opacity, corners, and font size")
        .expanded(false)
        .build();

    // Opacity
    let opacity_adj = gtk::Adjustment::new(config.borrow().opacity, 0.0, 1.0, 0.05, 0.1, 0.0);
    let opacity_scale = gtk::Scale::builder()
        .adjustment(&opacity_adj)
        .digits(2)
        .hexpand(true)
        .valign(gtk::Align::Center)
        .width_request(120)
        .build();

    let opacity_label = gtk::Label::builder()
        .label(&format!("{}%", (config.borrow().opacity * 100.0) as i32))
        .valign(gtk::Align::Center)
        .css_classes(["caption", "dim-label"])
        .width_request(40)
        .build();

    let opacity_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();
    opacity_box.append(&opacity_scale);
    opacity_box.append(&opacity_label);

    let opacity_row = adw::ActionRow::builder().title("Opacity").build();
    opacity_row.add_suffix(&opacity_box);

    let config_opacity_ref = Rc::clone(&config);
    let opacity_label_c = opacity_label.clone();
    opacity_scale.connect_value_changed(move |s| {
        let mut cfg = config_opacity_ref.borrow_mut();
        cfg.opacity = s.value();
        opacity_label_c.set_label(&format!("{}%", (s.value() * 100.0) as i32));
        let _ = save_hud_config(&cfg);
    });

    // Border Radius
    let radius_adj = gtk::Adjustment::new(
        config.borrow().border_radius as f64,
        0.0,
        50.0,
        1.0,
        5.0,
        0.0,
    );
    let radius_spin = gtk::SpinButton::builder()
        .adjustment(&radius_adj)
        .valign(gtk::Align::Center)
        .build();

    let radius_row = adw::ActionRow::builder()
        .title("Border Radius (px)")
        .build();
    radius_row.add_suffix(&radius_spin);

    let config_radius_ref = Rc::clone(&config);
    radius_spin.connect_value_changed(move |s| {
        let mut cfg = config_radius_ref.borrow_mut();
        cfg.border_radius = s.value() as i32;
        let _ = save_hud_config(&cfg);
    });

    // Font Size
    let font_adj = gtk::Adjustment::new(config.borrow().font_size as f64, 8.0, 48.0, 1.0, 4.0, 0.0);
    let font_spin = gtk::SpinButton::builder()
        .adjustment(&font_adj)
        .valign(gtk::Align::Center)
        .build();

    let font_row = adw::ActionRow::builder().title("Font Size (px)").build();
    font_row.add_suffix(&font_spin);

    let config_font_ref = Rc::clone(&config);
    font_spin.connect_value_changed(move |s| {
        let mut cfg = config_font_ref.borrow_mut();
        cfg.font_size = s.value() as i32;
        let _ = save_hud_config(&cfg);
    });

    expander.add_row(&opacity_row);
    expander.add_row(&radius_row);
    expander.add_row(&font_row);

    styling_group.add(&expander);
    styling_clamp.set_child(Some(&styling_group));
    main_box.append(&styling_clamp);

    let keybinds_label = gtk::Label::builder()
        .label("Keybind Selection")
        .halign(gtk::Align::Start)
        .margin_start(24)
        .margin_top(6)
        .margin_bottom(6)
        .css_classes(["heading"])
        .build();
    main_box.append(&keybinds_label);

    // --- List Content ---
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .build();

    let clamp = adw::Clamp::builder().maximum_size(800).build();

    let list_box = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["boxed-list"])
        .margin_bottom(24)
        .margin_start(12)
        .margin_end(12)
        .build();

    let n_items = model.n_items();
    for i in 0..n_items {
        let Some(obj) = model.item(i).and_downcast::<KeybindObject>() else {
            continue;
        };

        let (mods, key, disp, args) = obj.with_data(|d| {
            (
                Rc::<str>::from(&*d.mods),
                Rc::<str>::from(&*d.key),
                Rc::<str>::from(&*d.dispatcher),
                Rc::<str>::from(d.args.as_deref().unwrap_or_default()),
            )
        });

        let title_text = if mods.is_empty() {
            key.to_string()
        } else {
            format!("{} + {}", mods, key)
        };
        let subtitle_text = if args.is_empty() { &*disp } else { &*args };

        let row = adw::ActionRow::builder()
            .title(glib::markup_escape_text(&title_text))
            .subtitle(glib::markup_escape_text(subtitle_text))
            .build();

        let check = gtk::CheckButton::builder()
            .valign(gtk::Align::Center)
            .build();

        // Check if this keybind is already selected
        let is_selected = config.borrow().keybinds.iter().any(|kb| {
            *kb.mods == *mods && *kb.key == *key && *kb.dispatcher == *disp && *kb.args == *args
        });
        check.set_active(is_selected);

        let config_ref = Rc::clone(&config);
        check.connect_toggled(move |btn| {
            let kb = HudKeybind::new(&mods, &key, &disp, &args);
            let mut cfg = config_ref.borrow_mut();

            if btn.is_active() {
                if !cfg.keybinds.contains(&kb) {
                    cfg.keybinds.push(kb);
                }
            } else {
                cfg.keybinds.retain(|x| x != &kb);
            }
            let _ = save_hud_config(&cfg);
        });

        row.add_prefix(&check);

        list_box.append(&row);
    }

    // Filter logic - use a shared RefCell for the search text
    let search_text = Rc::new(RefCell::new(String::new()));
    let search_text_ref = Rc::clone(&search_text);
    let list_box_weak = list_box.downgrade();

    search_entry.connect_search_changed(move |entry| {
        *search_text_ref.borrow_mut() = entry.text().to_string().to_lowercase();
        if let Some(lb) = list_box_weak.upgrade() {
            lb.invalidate_filter();
        }
    });

    list_box.set_filter_func(move |row| {
        let text = search_text.borrow();
        if text.is_empty() {
            return true;
        }

        row.downcast_ref::<adw::ActionRow>()
            .map(|action_row| {
                let title = action_row.title();
                let subtitle = action_row.subtitle();
                title.to_lowercase().contains(&*text)
                    || subtitle
                        .as_ref()
                        .is_some_and(|s| s.to_lowercase().contains(&*text))
            })
            .unwrap_or(true)
    });

    clamp.set_child(Some(&list_box));
    scrolled.set_child(Some(&clamp));
    main_box.append(&scrolled);

    main_box.upcast()
}
