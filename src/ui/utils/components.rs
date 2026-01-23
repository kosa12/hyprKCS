use gtk::prelude::*;
use gtk4 as gtk;

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

    actions.set_valign(gtk::Align::Center);
    actions.set_margin_end(12);
    row.append(actions);

    row
}

pub fn create_recorder_row(
    entry_mods: &gtk::Entry,
    entry_key: &gtk::Entry,
    macro_switch: &gtk::Switch,
    center_widget: Option<&gtk::Widget>,
) -> gtk::Box {
    let recorder_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    
    crate::ui::utils::widgets::setup_key_recorder(&recorder_box, entry_mods, entry_key);

    if let Some(w) = center_widget {
        recorder_box.append(w);
    } else {
        let spacer = gtk::Box::builder().hexpand(true).build();
        recorder_box.append(&spacer);
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
