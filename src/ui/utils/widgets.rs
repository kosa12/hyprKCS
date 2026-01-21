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

pub fn create_back_button(tooltip: &str) -> gtk::Button {
    gtk::Button::builder()
        .icon_name("go-previous-symbolic")
        .css_classes(["flat", "circular", "small"])
        .tooltip_text(tooltip)
        .build()
}

pub fn create_pill_button(label: &str, icon: Option<&str>) -> gtk::Button {
    let btn = gtk::Button::builder()
        .label(label)
        .css_classes(["pill", "small"])
        .build();
    if let Some(i) = icon {
        btn.set_icon_name(i);
    }
    btn
}

pub fn create_suggested_button(label: &str, icon: Option<&str>) -> gtk::Button {
    let btn = gtk::Button::builder()
        .label(label)
        .css_classes(["suggested-action", "pill", "small"])
        .build();
    if let Some(i) = icon {
        btn.set_icon_name(i);
    }
    btn
}

pub fn create_destructive_button(label: &str, icon: Option<&str>) -> gtk::Button {
    let btn = gtk::Button::builder()
        .label(label)
        .css_classes(["destructive-action", "pill", "small"])
        .build();
    if let Some(i) = icon {
        btn.set_icon_name(i);
    }
    btn
}

pub fn create_flat_button(icon: &str, tooltip: &str) -> gtk::Button {
    gtk::Button::builder()
        .icon_name(icon)
        .tooltip_text(tooltip)
        .css_classes(["flat", "small"])
        .build()
}

pub fn create_page_header(
    title: &str,
    subtitle: Option<&str>,
    back_tooltip: &str,
    on_back: impl Fn() + 'static,
) -> gtk::Box {
    let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let back_btn = create_back_button(back_tooltip);
    back_btn.connect_clicked(move |_| on_back());

    let title_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let title_label = gtk::Label::builder()
        .label(title)
        .css_classes(["title-2"])
        .halign(gtk::Align::Start)
        .build();
    title_box.append(&title_label);

    if let Some(sub) = subtitle {
        let sub_label = gtk::Label::builder()
            .label(sub)
            .css_classes(["dim-label", "caption"])
            .halign(gtk::Align::Start)
            .wrap(true)
            .build();
        title_box.append(&sub_label);
    }

    header_box.append(&back_btn);
    header_box.append(&title_box);
    header_box
}

pub fn create_form_group(label_text: &str, widget: &impl IsA<gtk::Widget>) -> gtk::Box {
    let group = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let label = gtk::Label::builder()
        .label(label_text)
        .halign(gtk::Align::Start)
        .css_classes(["caption", "dim-label"])
        .build();
    group.append(&label);
    group.append(widget);
    group
}

pub fn create_card_row(
    title: &str,
    subtitle: Option<&str>,
    actions: &impl IsA<gtk::Widget>,
) -> gtk::Box {
    let row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .css_classes(["card"])
        .margin_start(4)
        .margin_end(4)
        .build();

    let info_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .hexpand(true)
        .build();

    let title_label = gtk::Label::builder()
        .label(title)
        .halign(gtk::Align::Start)
        .css_classes(["heading"])
        .wrap(true)
        .build();
    info_box.append(&title_label);

    if let Some(sub) = subtitle {
        let sub_label = gtk::Label::builder()
            .label(sub)
            .halign(gtk::Align::Start)
            .css_classes(["caption", "dim-label"])
            .ellipsize(gtk::pango::EllipsizeMode::Middle)
            .max_width_chars(45)
            .wrap(true)
            .build();
        info_box.append(&sub_label);
    }

    row.append(&info_box);

    // Ensure actions are vertically centered and have some margin
    actions.set_valign(gtk::Align::Center);
    actions.set_margin_end(12);
    row.append(actions);

    row
}
