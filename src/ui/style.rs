use gtk4 as gtk;

pub fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        "
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
        columnview header button label {
            text-align: center;
            margin-left: auto;
            margin-right: auto;
        }

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
            color: @accent_fg_color;
        }

        columnview row:selected .key-label, 
        columnview row:selected .mod-label {
            background-color: alpha(white, 0.15);
            border-color: alpha(white, 0.2);
            color: white;
        }
        
        /* Clean Search Entry */
        searchbar > revealer > box {
            padding: 12px;
            border-bottom: 1px solid alpha(@window_fg_color, 0.1);
        }
        "
    );

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}