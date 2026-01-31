use crate::keybind_object::KeybindObject;
use gtk::gio;
use gtk::prelude::*;
use gtk4 as gtk;
use std::collections::HashSet;

pub fn collect_submaps(model: &gio::ListStore) -> Vec<String> {
    let mut submaps = HashSet::new();
    for obj in model.snapshot() {
        if let Some(obj) = obj.downcast_ref::<KeybindObject>() {
            if let Some(s) = obj.with_data(|d| d.submap.as_ref().map(|r| r.to_string())) {
                if !s.is_empty() {
                    submaps.insert(s);
                }
            }
        }
    }
    let mut sorted: Vec<String> = submaps.into_iter().collect();
    sorted.sort();
    sorted
}

#[allow(deprecated)]
pub fn create_submap_combo(
    model: &gio::ListStore,
    current_submap: Option<&str>,
) -> gtk::ComboBoxText {
    let combo = gtk::ComboBoxText::with_entry();

    // Add "Global" option (empty string)
    combo.append(Some(""), "Global (Default)");

    let submaps = collect_submaps(model);
    for sub in submaps {
        combo.append(Some(&sub), &sub);
    }

    // Set active
    if let Some(s) = current_submap {
        if s.is_empty() {
            combo.set_active_id(Some(""));
        } else {
            // Check if it exists in list, if not, we must set text manually in entry
            // But ComboBoxText with entry allows custom text.
            // set_active_id only works if it matches an appended ID.
            if !combo.set_active_id(Some(s)) {
                if let Some(entry) = combo.child().and_downcast::<gtk::Entry>() {
                    entry.set_text(s);
                }
            }
        }
    } else {
        combo.set_active_id(Some(""));
    }

    combo
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
    title_box.set_hexpand(true);
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
            .wrap(false)
            .ellipsize(gtk::pango::EllipsizeMode::Middle)
            .max_width_chars(40)
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

    actions.set_valign(gtk::Align::Center);
    actions.set_margin_end(12);
    row.append(actions);

    row
}

pub fn create_recorder_row(
    entry_mods: &gtk::Entry,
    entry_key: &gtk::Entry,
    macro_switch: &gtk::Switch,
    mouse_switch: Option<&gtk::Switch>,
    center_widget: Option<&gtk::Widget>,
) -> gtk::Box {
    let recorder_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    recorder_box.set_margin_end(12);

    crate::ui::utils::widgets::setup_key_recorder(&recorder_box, entry_mods, entry_key);

    if let Some(w) = center_widget {
        recorder_box.append(w);
    } else {
        let spacer = gtk::Box::builder().hexpand(true).build();
        recorder_box.append(&spacer);
    }

    // --- Mouse Switch ---
    if let Some(ms) = mouse_switch {
        ms.set_margin_end(12); // Spacing between switches

        let ms_label = gtk::Label::builder()
            .label("Mouse")
            .css_classes(["caption", "dim-label"])
            .valign(gtk::Align::Center)
            .build();

        recorder_box.append(&ms_label);
        recorder_box.append(ms);
    }

    // --- Macro Switch ---
    macro_switch.set_margin_end(8);

    let switch_label = gtk::Label::builder()
        .label("Macro")
        .css_classes(["caption", "dim-label"])
        .valign(gtk::Align::Center)
        .build();

    recorder_box.append(&switch_label);
    recorder_box.append(macro_switch);

    recorder_box
}

pub fn create_flags_dropdown() -> gtk::DropDown {
    let list = gtk::StringList::new(&[
        "Standard (bind)",
        "Locked (bindl)",
        "Repeat (binde)",
        "Release (bindr)",
        "Locked + Repeat (bindel)",
        "Ignore Mods (bindn)",
        "Transparent (bindt)",
        "Ignore Mods + Locked (bindnl)",
        "Mouse (bindm)",
        "Description (bindd)",
    ]);

    gtk::DropDown::builder().model(&list).build()
}

pub fn get_flag_from_index(index: u32) -> &'static str {
    match index {
        0 => "",
        1 => "l",
        2 => "e",
        3 => "r",
        4 => "el",
        5 => "n",
        6 => "t",
        7 => "nl",
        8 => "m",
        9 => "d",
        _ => "",
    }
}

pub fn get_index_from_flag(flag: &str) -> u32 {
    match flag {
        "" => 0,
        "l" => 1,
        "e" => 2,
        "r" => 3,
        "el" => 4,
        "n" => 5,
        "t" => 6,
        "nl" => 7,
        "m" => 8,
        "d" => 9,
        _ => 0,
    }
}

pub fn create_mouse_button_dropdown() -> gtk::DropDown {
    let list = gtk::StringList::new(&[
        "Left Click (mouse:272)",
        "Right Click (mouse:273)",
        "Middle Click (mouse:274)",
        "Side Button 1 (mouse:275)",
        "Side Button 2 (mouse:276)",
        "Extra Button 1 (mouse:277)",
        "Extra Button 2 (mouse:278)",
        "Scroll Up (mouse_up)",
        "Scroll Down (mouse_down)",
    ]);

    gtk::DropDown::builder().model(&list).build()
}

pub fn get_mouse_code_from_index(index: u32) -> &'static str {
    match index {
        0 => "mouse:272",
        1 => "mouse:273",
        2 => "mouse:274",
        3 => "mouse:275",
        4 => "mouse:276",
        5 => "mouse:277",
        6 => "mouse:278",
        7 => "mouse_up",
        8 => "mouse_down",
        _ => "mouse:272",
    }
}

pub fn get_index_from_mouse_code(code: &str) -> u32 {
    match code {
        "mouse:272" => 0,
        "mouse:273" => 1,
        "mouse:274" => 2,
        "mouse:275" => 3,
        "mouse:276" => 4,
        "mouse:277" => 5,
        "mouse:278" => 6,
        "mouse_up" => 7,
        "mouse_down" => 8,
        _ => 0, // Default to Left Click if unknown
    }
}
