use crate::ui::utils::execution::execute_hyprctl;
use gtk::{gdk, glib, prelude::*};
use gtk4 as gtk;

#[allow(deprecated)]
pub fn setup_dispatcher_completion(entry: &gtk::Entry) {
    let dispatchers = [
        "exec",
        "execr",
        "pass",
        "killactive",
        "closewindow",
        "workspace",
        "movetoworkspace",
        "movetoworkspacesilent",
        "togglefloating",
        "fullscreen",
        "fakefullscreen",
        "dpms",
        "pin",
        "movefocus",
        "movewindow",
        "centerwindow",
        "resizeactive",
        "moveactive",
        "cyclenext",
        "swapnext",
        "focuswindow",
        "focusmonitor",
        "splitratio",
        "toggleopaque",
        "movecursortocorner",
        "workspaceopt",
        "exit",
        "forcerendererreload",
        "movecurrentworkspacetomonitor",
        "focusworkspaceoncurrentmonitor",
        "togglespecialworkspace",
        "focusurgentorlast",
        "togglegroup",
        "changegroupactive",
        "swapprev",
        "focuscurrentorlast",
        "lockgroups",
        "lockactivegroup",
        "moveintogroup",
        "moveoutofgroup",
        "movewindoworgroup",
        "movegroupwindow",
        "denywindowfromgroup",
        "setignoregrouplock",
        "alterzorder",
        "tag",
        "layoutmsg",
        "sendshortcut",
        "sendkeystate",
    ];

    let list_store = gtk::ListStore::new(&[glib::Type::STRING]);
    for dispatcher in dispatchers {
        list_store.set(&list_store.append(), &[(0, &dispatcher)]);
    }

    let completion = gtk::EntryCompletion::builder()
        .model(&list_store)
        .text_column(0)
        .inline_completion(true)
        .popup_completion(false)
        .build();

    entry.set_completion(Some(&completion));
}

pub fn get_all_key_names() -> Vec<String> {
    let mut keys = std::collections::HashSet::new();

    // Collect keys from all layouts
    use crate::ui::views::keyboard_layouts::*;
    let layouts = [
        ANSI_ROW_1,
        ANSI_ROW_2,
        ANSI_ROW_3,
        ANSI_ROW_4,
        ANSI_ROW_5,
        ISO_ROW_2,
        ISO_ROW_3,
        ISO_ROW_4,
        JIS_ROW_1,
        JIS_ROW_2,
        JIS_ROW_3,
        JIS_ROW_4,
        JIS_ROW_5,
        ABNT2_ROW_2,
        ABNT2_ROW_3,
        ABNT2_ROW_4,
        HU_ROW_1,
        HU_ROW_2,
        HU_ROW_3,
        HU_ROW_4,
        ROW_FUNC,
        ROW_ARROWS,
    ];

    for row in layouts {
        for key in row {
            if !key.hypr_name.is_empty() {
                keys.insert(key.hypr_name.to_string());
            }
        }
    }

    // Add common XF86 keys
    let xf86_keys = [
        "XF86AudioRaiseVolume",
        "XF86AudioLowerVolume",
        "XF86AudioMute",
        "XF86AudioMicMute",
        "XF86MonBrightnessUp",
        "XF86MonBrightnessDown",
        "XF86AudioPlay",
        "XF86AudioStop",
        "XF86AudioPrev",
        "XF86AudioNext",
        "XF86Search",
        "XF86Mail",
        "XF86Calculator",
        "XF86Sleep",
        "XF86WLAN",
        "XF86Bluetooth",
        "XF86TouchpadToggle",
    ];
    for k in xf86_keys {
        keys.insert(k.to_string());
    }

    let mut sorted_keys: Vec<String> = keys.into_iter().collect();
    sorted_keys.sort();
    sorted_keys
}

#[allow(deprecated)]
pub fn setup_key_completion(entry: &gtk::Entry) {
    let sorted_keys = get_all_key_names();

    let list_store = gtk::ListStore::new(&[glib::Type::STRING]);
    for key in sorted_keys {
        list_store.set(&list_store.append(), &[(0, &key)]);
    }

    let completion = gtk::EntryCompletion::builder()
        .model(&list_store)
        .text_column(0)
        .inline_completion(true)
        .popup_completion(false)
        .build();

    entry.set_completion(Some(&completion));
}

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
        gdk::Key::space => "space".to_string(),
        gdk::Key::Escape => "Escape".to_string(),
        gdk::Key::BackSpace => "BackSpace".to_string(),
        gdk::Key::Left => "Left".to_string(),
        gdk::Key::Right => "Right".to_string(),
        gdk::Key::Up => "Up".to_string(),
        gdk::Key::Down => "Down".to_string(),
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
        .margin_end(24)
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
        if btn.label().is_some_and(|l| l == "Listening...") {
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
        if record_btn.label().is_none_or(|l| l != "Listening...") {
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
