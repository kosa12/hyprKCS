use crate::config::hud::{get_hud_pid_path, is_hud_running, load_hud_config, HudPosition};
use crate::config::StyleConfig;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk4 as gtk;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use libadwaita as adw;
use libc;
use std::fs;
use std::rc::Rc;

fn update_window_position(window: &gtk::ApplicationWindow, position: HudPosition) {
    // Reset anchors first
    window.set_anchor(Edge::Top, false);
    window.set_anchor(Edge::Bottom, false);
    window.set_anchor(Edge::Left, false);
    window.set_anchor(Edge::Right, false);

    match position {
        HudPosition::TopRight => {
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Right, true);
        }
        HudPosition::TopLeft => {
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Left, true);
        }
        HudPosition::BottomRight => {
            window.set_anchor(Edge::Bottom, true);
            window.set_anchor(Edge::Right, true);
        }
        HudPosition::BottomLeft => {
            window.set_anchor(Edge::Bottom, true);
            window.set_anchor(Edge::Left, true);
        }
    }
}

fn generate_hud_css(style: &StyleConfig) -> String {
    let border_radius = style.border_radius.as_deref().unwrap_or("16px");

    let font_size = style.font_size.as_deref().unwrap_or("0.9rem");

    let opacity = style.opacity.unwrap_or(0.75);

    format!(
        r#"

        window, .background, .main {{

            background-color: transparent;

            background-image: none;

            box-shadow: none;

        }}

        .hud-container {{

            background: alpha(@window_bg_color, {});

            padding: 24px;

            border-radius: {};

            border: 1px solid alpha(@window_fg_color, 0.1);

            color: @window_fg_color;

        }}

        .hud-title {{

            font-size: calc({} * 1.3);

            font-weight: 800;

            margin-bottom: 4px;

            color: @window_fg_color;

        }}

        .hud-keys {{

            font-size: {};

            font-weight: 600;

            color: @accent_color;

            font-family: monospace;

        }}

        .hud-action {{

            font-size: {};

            color: alpha(@window_fg_color, 0.8);

            font-style: italic;

        }}

        .hud-empty {{

            color: alpha(@window_fg_color, 0.4);

            padding: 10px;

        }}

        separator {{

            background-color: alpha(@window_fg_color, 0.1);

            margin-bottom: 8px;

        }}

    "#,
        opacity, border_radius, font_size, font_size, font_size
    )
}

fn update_keybind_list(container: &gtk::Box) {
    // Clear current list (skip title and separator)

    let mut child = container.first_child(); // Title

    if let Some(c) = child {
        child = c.next_sibling(); // Separator

        if let Some(s) = child {
            child = s.next_sibling(); // First row or empty label

            while let Some(row) = child {
                let next = row.next_sibling();

                container.remove(&row);

                child = next;
            }
        }
    }

    let config = load_hud_config();

    if config.keybinds.is_empty() {
        container.append(
            &gtk::Label::builder()
                .label("No keybinds selected")
                .css_classes(["hud-empty"])
                .build(),
        );
    } else {
        for kb in &config.keybinds {
            let row = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(24)
                .build();

            let key_text = if kb.mods.is_empty() {
                kb.key.to_string()
            } else {
                format!("{} + {}", kb.mods, kb.key)
            };

            row.append(
                &gtk::Label::builder()
                    .label(glib::markup_escape_text(&key_text))
                    .css_classes(["hud-keys"])
                    .halign(gtk::Align::Start)
                    .hexpand(true)
                    .build(),
            );

            row.append(
                &gtk::Label::builder()
                    .label(glib::markup_escape_text(&kb.args))
                    .css_classes(["hud-action"])
                    .halign(gtk::Align::End)
                    .build(),
            );

            container.append(&row);
        }
    }
}

pub fn run_hud() {
    let config = load_hud_config();

    // --- Single Instance Locking ---
    if is_hud_running() {
        eprintln!("HUD is already running");
        return;
    }

    // Write current PID
    if let Some(pid_path) = get_hud_pid_path() {
        if let Some(parent) = pid_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&pid_path, std::process::id().to_string());
    }

    glib::log_set_writer_func(|level, fields| {
        for field in fields {
            if field.key() == "MESSAGE" {
                if let Some(msg) = field.value_str() {
                    if msg.contains("gtk-application-prefer-dark-theme") {
                        return glib::LogWriterOutput::Handled;
                    }
                }
            }
        }

        glib::log_writer_default(level, fields)
    });

    adw::init().unwrap_or_else(|e| {
        eprintln!("Failed to initialize libadwaita: {}", e);
    });

    let app = adw::Application::builder()
        .application_id("com.github.hyprkcs.hud")
        .build();

    app.connect_activate(move |app| {
        let app_provider = gtk::CssProvider::new();

        let theme_provider = gtk::CssProvider::new();

        let style = StyleConfig::load();

        app_provider.load_from_string(&generate_hud_css(&style));

        if let Some(config_dir) = dirs::config_dir() {
            let gtk_css_path = config_dir.join("gtk-4.0/gtk.css");

            if gtk_css_path.exists() {
                theme_provider.load_from_file(&gio::File::for_path(&gtk_css_path));
            }
        }

        if let Some(display) = gtk::gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &theme_provider,
                gtk::STYLE_PROVIDER_PRIORITY_USER,
            );

            gtk::style_context_add_provider_for_display(
                &display,
                &app_provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("hyprKCS HUD")
            .resizable(false)
            .decorated(false)
            .build();

        window.init_layer_shell();

        window.set_layer(Layer::Background);

        window.set_namespace(Some("hyprkcs-hud"));

        update_window_position(&window, config.position);

        window.set_margin(Edge::Top, 40);

        window.set_margin(Edge::Bottom, 40);

        window.set_margin(Edge::Left, 40);

        window.set_margin(Edge::Right, 40);

        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .css_classes(["hud-container"])
            .build();

        container.append(
            &gtk::Label::builder()
                .label("hyprKCS HUD")
                .css_classes(["hud-title"])
                .halign(gtk::Align::Start)
                .build(),
        );

        container.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        update_keybind_list(&container);

        // --- Theme and Config Listeners ---

        let manager = adw::StyleManager::default();
        let app_prov_c = app_provider.clone();
        let theme_prov_c = theme_provider.clone();

        let reload_all = move || {
            if let Some(config_dir) = dirs::config_dir() {
                let gtk_css_path = config_dir.join("gtk-4.0/gtk.css");
                if gtk_css_path.exists() {
                    theme_prov_c.load_from_file(&gio::File::for_path(&gtk_css_path));
                } else {
                    theme_prov_c.load_from_string("");
                }
            }

            let style = StyleConfig::load();
            app_prov_c.load_from_string(&generate_hud_css(&style));
        };

        let reload = reload_all.clone();
        manager.connect_notify_local(None, move |_, pspec| {
            let name = pspec.name();
            if matches!(name, "dark" | "accent-color" | "color-scheme") {
                reload();
            }
        });

        if let Some(settings) = gtk::Settings::default() {
            let reload = reload_all.clone();
            settings.connect_notify_local(None, move |_, pspec| {
                let name = pspec.name();
                if matches!(
                    name,
                    "gtk-theme-name" | "gtk-color-scheme" | "gtk-application-prefer-dark-theme"
                ) {
                    reload();
                }
            });
        }

        // Listen for config changes and THEME changes (file system)

        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir
                .join(crate::config::constants::HYPRKCS_DIR)
                .join(crate::config::constants::HYPRKCS_CONF);

            let hud_json_path = config_dir
                .join(crate::config::constants::HYPRKCS_DIR)
                .join(crate::config::constants::HUD_CONF);

            // --- Theme Monitoring (File System) ---
            let dirs_to_monitor = [config_dir.join("gtk-4.0"), config_dir.join("gtk-3.0")];

            for (i, dir_path) in dirs_to_monitor.iter().enumerate() {
                let dir_file = gio::File::for_path(dir_path);
                if let Ok(monitor) =
                    dir_file.monitor_directory(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE)
                {
                    let theme_prov_f = theme_provider.clone();
                    let app_prov_f = app_provider.clone();

                    monitor.connect_changed(move |_, file, _, event| {
                        if let Some(path) = file.path() {
                            if let Some(name) = path.file_name() {
                                if name == "gtk.css" || name == "settings.ini" {
                                    match event {
                                        gio::FileMonitorEvent::ChangesDoneHint
                                        | gio::FileMonitorEvent::Changed
                                        | gio::FileMonitorEvent::Created
                                        | gio::FileMonitorEvent::AttributeChanged => {
                                            let tp = theme_prov_f.clone();
                                            let ap = app_prov_f.clone();
                                            let config_dir = dirs::config_dir().unwrap();
                                            let css_path = config_dir.join("gtk-4.0/gtk.css");
                                            let f = gio::File::for_path(&css_path);

                                            glib::timeout_add_local(
                                                std::time::Duration::from_millis(200),
                                                move || {
                                                    // Force reload theme
                                                    if css_path.exists() {
                                                        tp.load_from_file(&f);
                                                    } else {
                                                        tp.load_from_string("");
                                                    }
                                                    let style = StyleConfig::load();
                                                    ap.load_from_string(&generate_hud_css(&style));
                                                    glib::ControlFlow::Break
                                                },
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    });

                    unsafe {
                        window.set_data(&format!("theme-monitor-{}", i), Rc::new(monitor));
                    }
                }
            }

            let app_prov_f = app_provider.clone();
            let container_f = container.clone();

            // Monitor hyprkcs.conf (style)

            let file_conf = gio::File::for_path(&config_path);

            if let Ok(monitor) =
                file_conf.monitor(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE)
            {
                let app_prov_f2 = app_prov_f.clone();

                monitor.connect_changed(move |_, _, _, _| {
                    let style = StyleConfig::load();

                    app_prov_f2.load_from_string(&generate_hud_css(&style));
                });

                unsafe {
                    window.set_data("config-monitor", Rc::new(monitor));
                }
            }

            // Monitor hud.json (keybind selection and position)

            let file_hud = gio::File::for_path(&hud_json_path);

            if let Ok(monitor) =
                file_hud.monitor(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE)
            {
                let window_p = window.clone();

                monitor.connect_changed(move |_, _, _, _| {
                    let cfg = load_hud_config();

                    update_keybind_list(&container_f);

                    update_window_position(&window_p, cfg.position);
                });

                unsafe {
                    window.set_data("hud-monitor", Rc::new(monitor));
                }
            }
        }

        window.set_child(Some(&container));
        window.present();
    });

    app.connect_shutdown(|_| {
        if let Some(pid_path) = get_hud_pid_path() {
            let _ = fs::remove_file(pid_path);
        }
    });

    // Handle signals to exit cleanly
    let app_clone = app.clone();
    glib::unix_signal_add_local(libc::SIGTERM, move || {
        app_clone.quit();
        glib::ControlFlow::Break
    });

    let app_clone = app.clone();
    glib::unix_signal_add_local(libc::SIGINT, move || {
        app_clone.quit();
        glib::ControlFlow::Break
    });

    app.run_with_args::<String>(&[]);
}
