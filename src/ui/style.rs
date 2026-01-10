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
            overflow: hidden;
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
        "
    );

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}