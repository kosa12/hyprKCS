use gtk4 as gtk;
use libadwaita as adw;
use libadwaita::prelude::*;

pub fn create_about_page(window: &adw::ApplicationWindow) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder().build();

    let main_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(24)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(12)
        .margin_end(12)
        .build();

    // --- Header ---
    let header_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .halign(gtk::Align::Center)
        .build();

    let title = gtk::Label::builder()
        .label("hyprKCS")
        .css_classes(["title-1"])
        .build();

    header_box.append(&title);
    main_box.append(&header_box);

    // --- Row 1: Info (Version - Dev - License) ---
    let info_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Center)
        .css_classes(["card"]) // Optional: card look
        .build();

    // Helper for info badges
    let create_badge = |text: &str, css: &str| {
        gtk::Label::builder()
            .label(text)
            .css_classes(["body", css])
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(12)
            .margin_end(12)
            .build()
    };

    info_box.append(&create_badge(
        &format!("v{}", env!("CARGO_PKG_VERSION")),
        "accent",
    ));
    info_box.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    info_box.append(&create_badge("Dev: kosa12", "dim-label"));
    info_box.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    info_box.append(&create_badge("License: GPL-3.0", "dim-label"));

    main_box.append(&info_box);

    // --- Helper for Buttons ---
    let create_button = |label: &str, subtitle: &str, icon_name: &str, url: &str, css: &[&str]| {
        let btn = gtk::Button::builder()
            .css_classes(css)
            .hexpand(true)
            .build();
        
        // Custom content for button: Icon + Text Stack
        let content_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk::Align::Center)
            .build();
        
        let img = gtk::Image::from_icon_name(icon_name);
        
        let text_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .valign(gtk::Align::Center)
            .build();
            
        let l_title = gtk::Label::builder().label(label).css_classes(["heading"]).halign(gtk::Align::Start).build();
        let l_sub = gtk::Label::builder().label(subtitle).css_classes(["caption-heading"]).halign(gtk::Align::Start).build();
        
        text_box.append(&l_title);
        text_box.append(&l_sub);
        
        content_box.append(&img);
        content_box.append(&text_box);
        
        btn.set_child(Some(&content_box));

        let u = url.to_string();
        let w = window.clone();
        btn.connect_clicked(move |_| {
            let launcher = gtk::UriLauncher::new(&u);
            launcher.launch(Some(&w), None::<&gtk::gio::Cancellable>, |res| {
                if let Err(e) = res {
                    eprintln!("Failed to launch URL: {}", e);
                }
            });
        });
        btn
    };

    // --- Row 2: GitHub & Issues ---
    let row2 = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .homogeneous(true)
        .build();

    row2.append(&create_button(
        "Star on GitHub",
        "View Source",
        "starred-symbolic",
        "https://github.com/kosa12/hyprKCS",
        &["flat", "card"],
    ));
    row2.append(&create_button(
        "Report Issue",
        "Bug or Feature?",
        "dialog-warning-symbolic",
        "https://github.com/kosa12/hyprKCS/issues",
        &["flat", "card"],
    ));

    main_box.append(&row2);

    // --- Row 3: Donate ---
    let row3 = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .homogeneous(true)
        .build();

    row3.append(&create_button(
        "Donate",
        "on Ko-fi",
        "emblem-favorite-symbolic",
        "https://ko-fi.com/kosa12",
        &["suggested-action"],
    ));
    row3.append(&create_button(
        "Sponsor",
        "on GitHub",
        "emblem-favorite-symbolic",
        "https://github.com/sponsors/kosa12",
        &["suggested-action"],
    ));

    main_box.append(&row3);

    // Wrapper Group & Clamp
    let group = adw::PreferencesGroup::builder().build();
    let clamp = adw::Clamp::builder()
        .maximum_size(650)
        .child(&main_box)
        .build();
    
    group.add(&clamp);
    page.add(&group);

    page
}
