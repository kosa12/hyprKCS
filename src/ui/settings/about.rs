use gtk4 as gtk;
use libadwaita as adw;
use libadwaita::prelude::*;

pub fn create_about_page(window: &adw::ApplicationWindow) -> adw::PreferencesPage {
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
        .subtitle("GPL-3.0")
        .build();
    let lic_img = gtk::Image::from_icon_name("dialog-information-symbolic");
    lic_row.add_prefix(&lic_img);
    group_about.add(&lic_row);

    let group_docs = adw::PreferencesGroup::builder().title("Documentation").build();

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

    group_docs.add(&create_link(
        "Wiki",
        "Documentation and Guides",
        "system-help-symbolic",
        "https://github.com/kosa12/hyprKCS/wiki",
    ));

    let group_community = adw::PreferencesGroup::builder().title("Community").build();

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

    page_about.add(&group_about);
    page_about.add(&group_docs);
    page_about.add(&group_community);

    page_about
}
