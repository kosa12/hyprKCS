use gtk4 as gtk;
use libadwaita as adw;
use gtk::{gio, glib, prelude::*};
use std::cell::RefCell;

const APP_CSS: &str = "
        /* Modern Keycap Look */
        .key-label, .mod-label {
            font-family: 'JetBrains Mono', 'Fira Code', monospace;
            font-weight: 800;
            font-size: 0.9rem;
            background-color: alpha(@window_fg_color, 0.08);
            border: 1px solid alpha(@window_fg_color, 0.1);
            border-bottom-width: 2px;
            border-radius: 6px;
            padding: 2px 8px;
            margin: 4px 0;
            color: @window_fg_color;
        }
        
        .mod-label {
            font-weight: 600;
            background-color: alpha(@accent_color, 0.1);
            border-color: alpha(@accent_color, 0.2);
            color: @accent_color;
        }

        .dispatcher-label {
            font-weight: 700;
            color: @window_fg_color;
        }

        .args-label {
            color: alpha(@window_fg_color, 0.55);
            font-style: italic;
        }

        /* Conflict Styling */
        .error-icon {
            color: @error_color;
        }

        /* ColumnView Refinement */
        columnview {
            background-color: transparent;
        }
        
        columnview listview {
            margin: 8px;
        }

        columnview row {
            border-radius: 10px;
            margin: 2px 0;
            transition: background-color 200ms ease;
        }

        columnview row:hover {
            background-color: alpha(@window_fg_color, 0.04);
        }

        columnview row:selected {
            background-color: @accent_bg_color;
        }

        columnview row:selected label,
        columnview row:selected .key-label, 
        columnview row:selected .mod-label,
        columnview row:selected .dispatcher-label {
            color: #242424;
        }

        columnview row:selected .key-label, 
        columnview row:selected .mod-label {
            background-color: alpha(black, 0.1);
            border-color: alpha(black, 0.15);
        }

        columnview row:selected .args-label {
            color: alpha(black, 0.55);
        }
        
        /* Clean Search Entry */
        searchbar > revealer > box {
            padding: 12px;
            border-bottom: 1px solid alpha(@window_fg_color, 0.1);
        }

        /* Menu Window Styling */
        window.menu-window {
            background-color: transparent;
        }

        .window-content {
            background-color: @theme_bg_color;
            border: 1px solid alpha(@window_fg_color, 0.15);
            border-radius: 12px;
            box-shadow: 0 4px 24px rgba(0,0,0,0.4); 
            margin: 12px;
        }

        button.flat {
            background: transparent;
            box-shadow: none;
            border: none;
        }
        button.flat:hover {
            background-color: alpha(@window_fg_color, 0.08);
        }

        /* Record Button - Absolute Flatness */
        button.record-btn {
            background-color: alpha(@window_fg_color, 0.08);
            border-radius: 9999px;
            border: none;
            box-shadow: none;
            text-shadow: none;
            -gtk-icon-shadow: none;
            outline: none;
        }
        
        button.record-btn:hover {
            background-color: alpha(@window_fg_color, 0.12);
            box-shadow: none;
            border: none;
        }
        
        button.record-btn:active, 
        button.record-btn:checked,
        button.record-btn:focus {
             background-color: alpha(@window_fg_color, 0.16);
             box-shadow: none;
             border: none;
             outline: none;
        }
";

thread_local! {
    static THEME_MONITOR: RefCell<Option<gio::FileMonitor>> = RefCell::new(None);
}

pub fn load_css() {
    let app_provider = gtk::CssProvider::new();
    app_provider.load_from_data(APP_CSS);

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
    
    if let Some(settings) = gtk::Settings::default() {
        let app_prov = app_provider.clone();
        let theme_prov = theme_provider.clone();
        settings.connect_gtk_theme_name_notify(move |_| {
            if let Some(config_dir) = dirs::config_dir() {
                 let css_file = gio::File::for_path(config_dir.join("gtk-4.0/gtk.css"));
                 theme_prov.load_from_file(&css_file);
            }
            app_prov.load_from_data(APP_CSS);
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
        app_prov.load_from_data(APP_CSS);
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
                        if path.file_name().map_or(false, |n| n == "gtk.css") {
                             match event {
                                 gio::FileMonitorEvent::ChangesDoneHint |
                                 gio::FileMonitorEvent::Changed |
                                 gio::FileMonitorEvent::Created |
                                 gio::FileMonitorEvent::AttributeChanged => {
                                     let theme_prov = theme_provider.clone();
                                     let app_prov = app_provider.clone();
                                     let f = gio::File::for_path(&path);
                                     
                                     glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
                                         theme_prov.load_from_file(&f);
                                         app_prov.load_from_data(APP_CSS);
                                         glib::ControlFlow::Break
                                     });
                                 }
                                 _ => {}
                             }
                        }
                    }
                });
                
                THEME_MONITOR.with(|m| {
                    *m.borrow_mut() = Some(monitor);
                });
            },
            Err(e) => eprintln!("Failed to monitor theme directory: {}", e),
        }
    }
}