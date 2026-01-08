use gtk4 as gtk;

pub fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        "
        .key-label {
            font-family: monospace;
            font-weight: 800;
            background-color: alpha(currentColor, 0.1);
            border-radius: 6px;
            padding: 2px 8px;
            margin: 2px 0;
        }
        .mod-label {
            font-family: monospace;
            font-weight: bold;
            color: alpha(currentColor, 0.8);
            background-color: alpha(currentColor, 0.05);
            border-radius: 6px;
            padding: 2px 8px;
            margin: 2px 0;
        }
        .dispatcher-label {
            font-weight: 700;
            color: @accent_color;
        }
        .args-label {
            color: alpha(currentColor, 0.7);
        }
        .conflicted {
            color: @error_color;
            font-weight: bold;
        }
        columnview row:selected .key-label, 
        columnview row:selected .mod-label {
            background-color: alpha(white, 0.2);
            color: white;
        }
        "
    );

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
