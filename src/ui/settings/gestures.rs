use crate::parser::input::{save_input_config, GesturesConfig, InputConfig};
use gtk::glib;
use gtk4 as gtk;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_gestures_page(
    input_config: Rc<RefCell<InputConfig>>,
    gestures_config: Rc<RefCell<GesturesConfig>>,
    on_show_toast: Rc<dyn Fn(String)>,
) -> adw::PreferencesPage {
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
    let group_note = adw::PreferencesGroup::builder().title("Note").build();
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
    save_gestures_row.connect_activated(move |_| {
        match save_input_config(&c.borrow(), &g.borrow()) {
            Ok(_) => toast_cb_g("Gesture configuration saved successfully".to_string()),
            Err(e) => toast_cb_g(format!("Error saving config: {}", e)),
        }
    });
    group_save_gestures.add(&save_gestures_row);
    page_gestures.add(&group_save_gestures);

    page_gestures
}
