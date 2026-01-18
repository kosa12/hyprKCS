use crate::config::StyleConfig;
use gtk::{gio, glib, prelude::*};
use gtk4 as gtk;
use libadwaita as adw;
use std::cell::RefCell;

thread_local! {
    static THEME_MONITOR: RefCell<Option<gio::FileMonitor>> = const { RefCell::new(None) };
    static APP_PROVIDER: RefCell<Option<gtk::CssProvider>> = const { RefCell::new(None) };
}

fn generate_css(config: &StyleConfig) -> String {
    let font_size = config.font_size.as_deref().unwrap_or("0.9rem");
    // Submap is usually smaller
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
    let shadow = &config.shadow_size;

    let alternating_css = if config.alternating_row_colors {
        "columnview row:nth-child(even) {
            background-color: alpha(@window_fg_color, 0.03);
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
            font-size: 0.9em;
            padding: 0;
            margin: 0;
            background-color: alpha(@window_fg_color, 0.1);
            color: alpha(@window_fg_color, 0.7);
            border-radius: 4px;
            border: 1px solid alpha(@window_fg_color, 0.1);
        }}

        .keyboard-key:hover {{
             background-color: alpha(@window_fg_color, 0.2);
        }}

        .keyboard-key.accent {{
            background-color: @accent_bg_color;
            color: @accent_fg_color;
            border-color: @accent_color;
            box-shadow: 0 0 4px @accent_color;
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
            color: alpha(@window_fg_color, 0.70);
            font-size: {font_size};
        }}

        /* Conflict Styling */
        .error-icon {{
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
            background-color: alpha(black, 0.1);
            border-color: alpha(black, 0.15);
        }}

        columnview row:selected .args-label {{
            color: alpha(black, 0.55);
        }}

        columnview row:selected .description-label {{
            color: alpha(black, 0.70);
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
            box-shadow: {shadow}; 
            margin: {win_margin}px;
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
        
        button.record-btn:active, 
        button.record-btn:checked,
        button.record-btn:focus {{
             background-color: alpha(@window_fg_color, 0.16);
             box-shadow: none;
             border: none;
             outline: none;
        }}
    ",
        font_size = font_size,
        submap_font_size = submap_font_size,
        border_size = border_size,
        border_radius = border_radius,
        key_radius = key_radius,
        opacity = opacity,
        win_margin = win_margin,
        row_margin = row_margin,
        shadow = shadow
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

    if let Some(settings) = gtk::Settings::default() {
        let app_prov = app_provider.clone();
        let theme_prov = theme_provider.clone();
        settings.connect_gtk_theme_name_notify(move |_| {
            if let Some(config_dir) = dirs::config_dir() {
                let css_file = gio::File::for_path(config_dir.join("gtk-4.0/gtk.css"));
                theme_prov.load_from_file(&css_file);
            }
            // Reload config on theme change? Maybe not necessary but safe.
            let cfg = StyleConfig::load();
            app_prov.load_from_string(&generate_css(&cfg));
        });
    }

    let manager = adw::StyleManager::default();
    let app_prov = app_provider.clone();
    let theme_prov = theme_provider.clone();
    manager.connect_dark_notify(move |_| {
        if let Some(config_dir) = dirs::config_dir() {
            let css_file = gio::File::for_path(config_dir.join("gtk-4.0/gtk.css"));
            theme_prov.load_from_file(&css_file);
        }
        let cfg = StyleConfig::load();
        app_prov.load_from_string(&generate_css(&cfg));
    });

    start_theme_monitor(app_provider, theme_provider);
}

fn start_theme_monitor(app_provider: gtk::CssProvider, theme_provider: gtk::CssProvider) {
    if let Some(config_dir) = dirs::config_dir() {
        let gtk_config_dir = config_dir.join("gtk-4.0");
        let dir_file = gio::File::for_path(&gtk_config_dir);

        match dir_file.monitor_directory(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE) {
            Ok(monitor) => {
                monitor.connect_changed(move |_, file, _, event| {
                    let path = file.path();
                    if let Some(path) = path {
                        if path.file_name().is_some_and(|n| n == "gtk.css") {
                            match event {
                                gio::FileMonitorEvent::ChangesDoneHint
                                | gio::FileMonitorEvent::Changed
                                | gio::FileMonitorEvent::Created
                                | gio::FileMonitorEvent::AttributeChanged => {
                                    let theme_prov = theme_provider.clone();
                                    let app_prov = app_provider.clone();
                                    let f = gio::File::for_path(&path);

                                    glib::timeout_add_local(
                                        std::time::Duration::from_millis(200),
                                        move || {
                                            theme_prov.load_from_file(&f);
                                            // Also reload our app config in case user changed the config file
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
                });

                THEME_MONITOR.with(|m| {
                    *m.borrow_mut() = Some(monitor);
                });
            }
            Err(e) => eprintln!("Failed to monitor theme directory: {}", e),
        }
    }
}
