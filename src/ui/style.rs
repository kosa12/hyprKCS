use crate::config::StyleConfig;
use gtk::{gio, glib, prelude::*};
use gtk4 as gtk;
use libadwaita as adw;
use std::cell::RefCell;

thread_local! {
    static THEME_MONITORS: RefCell<Vec<gio::FileMonitor>> = const { RefCell::new(Vec::new()) };
    static APP_PROVIDER: RefCell<Option<gtk::CssProvider>> = const { RefCell::new(None) };
    static THEME_PROVIDER: RefCell<Option<gtk::CssProvider>> = const { RefCell::new(None) };
}

pub fn cleanup() {
    THEME_MONITORS.with(|m| {
        for monitor in m.borrow_mut().drain(..) {
            monitor.cancel();
        }
    });

    if let Some(display) = gtk::gdk::Display::default() {
        APP_PROVIDER.with(|p| {
            if let Some(provider) = p.borrow().as_ref() {
                gtk::style_context_remove_provider_for_display(&display, provider);
            }
        });
        THEME_PROVIDER.with(|p| {
            if let Some(provider) = p.borrow().as_ref() {
                gtk::style_context_remove_provider_for_display(&display, provider);
            }
        });
    }

    APP_PROVIDER.with(|p| {
        *p.borrow_mut() = None;
    });
    THEME_PROVIDER.with(|p| {
        *p.borrow_mut() = None;
    });
}

fn generate_css(config: &StyleConfig) -> String {
    let font_size = config.font_size.as_deref().unwrap_or("0.9rem");
    let submap_font_size = if let Some(fs) = &config.font_size {
        format!("calc({} * 0.9)", fs)
    } else {
        "0.8rem".to_string()
    };

    let border_size = config.border_size.as_deref().unwrap_or("1px");
    let border_radius = config.border_radius.as_deref().unwrap_or("12px");
    let key_radius = config.border_radius.as_deref().unwrap_or("6px");
    let opacity = config.opacity.unwrap_or(1.0);

    let win_margin = config.monitor_margin;
    let row_margin = config.row_padding;

    let alternating_css = if config.alternating_row_colors {
        "columnview row:nth-child(even) {
            background-color: alpha(currentColor, 0.03);
        }"
    } else {
        ""
    };

    format!(
        "
        /* Modern Keycap Look */
        .key-label, .mod-label {{
            font-family: monospace;
            font-weight: 800;
            font-size: {font_size};
            background-color: alpha(@window_fg_color, 0.08);
            border: {border_size} solid alpha(@window_fg_color, 0.1);
            border-bottom-width: calc({border_size} + 1px);
            border-radius: {key_radius};
            padding: 2px 8px;
            margin: 4px 0;
            color: @window_fg_color;
        }}

        .mod-label {{
            font-weight: 600;
            background-color: alpha(@accent_color, 0.1);
            border-color: alpha(@accent_color, 0.2);
            color: @accent_color;
        }}

        .submap-label {{
            font-family: monospace;
            font-size: {submap_font_size};
            font-weight: 700;
            background-color: alpha(@accent_color, 0.15);
            color: @accent_color;
            border-radius: 4px;
            padding: 2px 6px;
            margin: 4px 0;
        }}

        /* Visual Keyboard */
        .keyboard-container {{
            background-color: alpha(@window_fg_color, 0.03);
            border-radius: {border_radius};
            padding: 12px;
        }}

        .keyboard-key {{
            font-family: monospace;
            font-weight: bold;
            font-size: 0.85em;
            padding: 0;
            margin: 0;
            background-color: alpha(@window_fg_color, 0.1);
            color: alpha(@window_fg_color, 0.7);
            border-radius: 4px;
            border: 1px solid alpha(@window_fg_color, 0.1);
        }}

        .mod-toggle {{
            font-size: 0.75rem;
            padding: 2px 8px;
            min-height: 24px;
        }}

        .keyboard-key:hover {{
             background-color: alpha(@window_fg_color, 0.2);
        }}

        .keyboard-key.accent {{
            background-color: @accent_bg_color;
            background-image: none;
            color: @accent_fg_color;
            border-color: @accent_color;
        }}

        .dim-label {{
             opacity: 0.6;
        }}

        .dispatcher-label {{
            font-weight: 700;
            color: @window_fg_color;
            font-size: {font_size};
        }}

        .args-label {{
            color: alpha(@window_fg_color, 0.55);
            font-style: italic;
            font-size: {font_size};
        }}

        .description-label {{
            color: alpha(@window_fg_color, 0.7);
            font-size: {font_size};
        }}

        /* Conflict & Broken Styling */
        .error-icon, .destructive-action {{
            color: @error_color;
        }}

        /* ColumnView Refinement */
        columnview {{
            background-color: transparent;
        }}

        columnview listview {{
            margin: 8px;
        }}

        columnview row {{
            border-radius: {key_radius};
            margin: {row_margin}px 0;
        }}

        {alternating_css}

        columnview row:hover {{
            background-color: alpha(@window_fg_color, 0.04);
        }}

        columnview row:selected {{
            background-color: @accent_bg_color;
        }}

        columnview row:selected label,
        columnview row:selected .key-label,
        columnview row:selected .mod-label,
        columnview row:selected .dispatcher-label {{
            color: #242424;
        }}

        columnview row:selected .key-label,
        columnview row:selected .mod-label {{
            background-color: rgba(0, 0, 0, 0.1);
            border-color: rgba(0, 0, 0, 0.15);
        }}

        columnview row:selected .args-label {{
            color: rgba(0, 0, 0, 0.55);
        }}

        columnview row:selected .description-label {{
            color: rgba(0, 0, 0, 0.7);
        }}

        columnview row:selected .error-icon,
        columnview row:selected button {{
            color: #242424;
        }}

        columnview row:selected .warning,
        columnview row:selected button.warning {{
            color: #7a5000;
        }}

        /* Clean Search Entry */
        searchbar > revealer > box {{
            padding: 12px;
            border-bottom: {border_size} solid alpha(@window_fg_color, 0.1);
        }}

        /* Menu Window Styling */
        window.menu-window {{
            background-color: transparent;
        }}

        .window-content {{
            background-color: alpha(@theme_bg_color, {opacity});
            border: {border_size} solid alpha(@window_fg_color, 0.15);
            border-radius: {border_radius};
            margin: {win_margin}px;
        }}

        /* Settings / Adwaita Overrides for Opacity */
        preferencespage,
        preferencespage > scrolledwindow > viewport > clamp > box {{
            background-color: transparent;
        }}

        preferencesgroup list {{
            background-color: alpha(@window_fg_color, 0.04);
            border: {border_size} solid alpha(@window_fg_color, 0.1);
            border-radius: {border_radius};
        }}

        preferencesgroup row {{
             background-color: transparent;
        }}

        preferencesgroup row:hover {{
             background-color: alpha(@window_fg_color, 0.05);
        }}

        button.flat {{
            background: transparent;
            box-shadow: none;
            border: none;
        }}
        button.flat:hover {{
            background-color: alpha(@window_fg_color, 0.08);
        }}

        /* Record Button - Absolute Flatness */
        button.record-btn {{
            background-color: alpha(@window_fg_color, 0.08);
            border-radius: 9999px;
            border: none;
            box-shadow: none;
            text-shadow: none;
            -gtk-icon-shadow: none;
            outline: none;
        }}

        button.record-btn:hover {{
            background-color: alpha(@window_fg_color, 0.12);
            box-shadow: none;
            border: none;
        }}

        button.record-btn:focus {{
             background-color: alpha(@window_fg_color, 0.16);
             box-shadow: none;
             border: none;
             outline: none;
        }}

        /* Compact buttons */
        button.small {{
            padding: 4px 10px;
            min-height: 28px;
            font-size: 1rem;
        }}

        button.small.circular {{
            padding: 4px;
            min-width: 28px;
            min-height: 28px;
        }}
    ",
        font_size = font_size,
        submap_font_size = submap_font_size,
        border_size = border_size,
        border_radius = border_radius,
        key_radius = key_radius,
        opacity = opacity,
        win_margin = win_margin,
        row_margin = row_margin
    )
}

pub fn reload_style() {
    APP_PROVIDER.with(|p| {
        if let Some(provider) = p.borrow().as_ref() {
            let config = StyleConfig::load();
            let css_content = generate_css(&config);
            provider.load_from_string(&css_content);
        }
    });
}

pub fn load_css() {
    let config = StyleConfig::load();
    let css_content = generate_css(&config);

    let app_provider = gtk::CssProvider::new();
    app_provider.load_from_string(&css_content);

    APP_PROVIDER.with(|p| {
        *p.borrow_mut() = Some(app_provider.clone());
    });

    let theme_provider = gtk::CssProvider::new();
    let display = gtk::gdk::Display::default().expect("Could not connect to a display.");

    if let Some(config_dir) = dirs::config_dir() {
        let gtk_css_path = config_dir.join("gtk-4.0/gtk.css");
        let css_file = gio::File::for_path(&gtk_css_path);
        if gtk_css_path.exists() {
            theme_provider.load_from_file(&css_file);
        }
    }

    // Store theme provider reference for cleanup
    THEME_PROVIDER.with(|p| {
        *p.borrow_mut() = Some(theme_provider.clone());
    });

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

    // We capture the config to regenerate CSS on theme changes
    // Ideally we should reload config too, but for now reuse it.

    let app_prov = app_provider.clone();
    let theme_prov = theme_provider.clone();
    let reload_all = move || {
        let Some(display) = gtk::gdk::Display::default() else {
            return;
        };

        // Remove providers first to force a refresh
        gtk::style_context_remove_provider_for_display(&display, &theme_prov);
        gtk::style_context_remove_provider_for_display(&display, &app_prov);

        if let Some(config_dir) = dirs::config_dir() {
            let css_path = config_dir.join("gtk-4.0/gtk.css");
            if css_path.exists() {
                let css_file = gio::File::for_path(&css_path);
                theme_prov.load_from_file(&css_file);
            } else {
                // Clear if file doesn't exist anymore
                theme_prov.load_from_string("");
            }
        }

        let cfg = StyleConfig::load();
        app_prov.load_from_string(&generate_css(&cfg));

        // Re-add providers
        gtk::style_context_add_provider_for_display(
            &display,
            &theme_prov,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );
        gtk::style_context_add_provider_for_display(
            &display,
            &app_prov,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    };

    if let Some(settings) = gtk::Settings::default() {
        let reload = reload_all.clone();
        settings.connect_notify_local(None, move |_, pspec| {
            let name = pspec.name();
            if matches!(&*name, "gtk-theme-name" | "gtk-color-scheme" | "gtk-application-prefer-dark-theme") {
                reload();
            }
        });
    }

    let manager = adw::StyleManager::default();
    let reload = reload_all;
    manager.connect_notify_local(None, move |_, pspec| {
        let name = pspec.name();
        if matches!(&*name, "dark" | "accent-color" | "color-scheme") {
            reload();
        }
    });

    start_theme_monitor(app_provider, theme_provider);
}

fn start_theme_monitor(app_provider: gtk::CssProvider, theme_provider: gtk::CssProvider) {
    if let Some(config_dir) = dirs::config_dir() {
        // Monitor gtk-4.0
        monitor_dir(config_dir.join("gtk-4.0"), app_provider.clone(), theme_provider.clone());
        // Monitor gtk-3.0 (often used by theming tools like nwg-look/lxappearance)
        monitor_dir(config_dir.join("gtk-3.0"), app_provider, theme_provider);
    }
}

fn monitor_dir(path: std::path::PathBuf, app_provider: gtk::CssProvider, theme_provider: gtk::CssProvider) {
    let dir_file = gio::File::for_path(&path);
    match dir_file.monitor_directory(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE) {
        Ok(monitor) => {
            monitor.connect_changed(move |_, file, _, event| {
                let path = file.path();
                if let Some(path) = path {
                    if let Some(name) = path.file_name() {
                        if name == "gtk.css" || name == "settings.ini" {
                             match event {
                                gio::FileMonitorEvent::ChangesDoneHint
                                | gio::FileMonitorEvent::Changed
                                | gio::FileMonitorEvent::Created
                                | gio::FileMonitorEvent::AttributeChanged => {
                                    let theme_prov = theme_provider.clone();
                                    let app_prov = app_provider.clone();
                                    
                                    // Always try to load gtk-4.0/gtk.css even if gtk-3.0 triggered the change
                                    // (As we only care about applying the GTK4 css, but the trigger might come from elsewhere)
                                    let config_dir = dirs::config_dir().unwrap();
                                    let css_path = config_dir.join("gtk-4.0/gtk.css");
                                    let f = gio::File::for_path(&css_path);

                                    glib::timeout_add_local(
                                        std::time::Duration::from_millis(200),
                                        move || {
                                            if css_path.exists() {
                                                theme_prov.load_from_file(&f);
                                            }
                                            // Also reload our app config
                                            let cfg = StyleConfig::load();
                                            app_prov.load_from_string(&generate_css(&cfg));
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
            // We need to store both.
            THEME_MONITORS.with(|m| {
                m.borrow_mut().push(monitor);
            });
        }
        Err(e) => eprintln!("Failed to monitor theme directory {:?}: {}", path, e),
    }
}
