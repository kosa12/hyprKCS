use gtk4 as gtk;
use libadwaita as adw;
use libadwaita::prelude::*;

pub fn create_feedback_page(window: &adw::ApplicationWindow) -> adw::PreferencesPage {
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
    page_feedback
}
