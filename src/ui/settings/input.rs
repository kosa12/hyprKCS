use crate::parser::input::{save_input_config, GesturesConfig, InputConfig};

use gtk4 as gtk;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_input_page(
    input_config: Rc<RefCell<InputConfig>>,
    gestures_config: Rc<RefCell<GesturesConfig>>,
    on_show_toast: Rc<dyn Fn(String)>,
) -> adw::PreferencesPage {
    let page_input = adw::PreferencesPage::builder().build();
    let group_kb = adw::PreferencesGroup::builder().title("Keyboard").build();

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

    page_input
}
