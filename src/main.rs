use gtk4 as gtk;
use gtk::{glib, prelude::*};
use libadwaita as adw;

mod parser;
mod keybind_object;
mod ui;

const APP_ID: &str = "com.github.hyprkcs";

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_startup(|_| {
        adw::init().unwrap();
        let style_manager = adw::StyleManager::default();
        style_manager.set_color_scheme(adw::ColorScheme::Default);
        ui::style::load_css();
    });

    app.connect_activate(ui::window::build_ui);

    app.run()
}
