use gtk4 as gtk;
use gtk::{gio, glib, prelude::*};
use libadwaita as adw;

mod parser;
mod keybind_object;

use keybind_object::KeybindObject;

const APP_ID: &str = "com.github.hyprkcs";

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_startup(|_| {
        adw::init().unwrap();
        let style_manager = adw::StyleManager::default();
        style_manager.set_color_scheme(adw::ColorScheme::Default);
        load_css();
    });

    app.connect_activate(build_ui);

    app.run()
}

fn load_css() {
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

fn build_ui(app: &adw::Application) {
    let keybinds = parser::parse_config().unwrap_or_else(|err| {
        eprintln!("Error parsing config: {}", err);
        vec![]
    });

    // Detect conflicts
    let mut counts = std::collections::HashMap::new();
    for kb in &keybinds {
        // Normalize for conflict detection: lowercase
        let key = (kb.clean_mods.to_lowercase(), kb.key.to_lowercase());
        *counts.entry(key).or_insert(0) += 1;
    }

    let model = gio::ListStore::new::<KeybindObject>();
    for kb in keybinds {
        let count = counts.get(&(kb.clean_mods.to_lowercase(), kb.key.to_lowercase())).unwrap_or(&0);
        let is_conflicted = *count > 1;
        model.append(&KeybindObject::new(kb, is_conflicted));
    }

    let filter = gtk::CustomFilter::new(|_obj| {
        true
    });
    
    let filter_model = gtk::FilterListModel::new(Some(model.clone()), Some(filter.clone()));
    
    let selection_model = gtk::SingleSelection::new(Some(filter_model.clone()));

    let column_view = gtk::ColumnView::new(Some(selection_model));
    column_view.set_show_row_separators(false); 
    column_view.set_show_column_separators(false);
    column_view.set_vexpand(true);
    column_view.set_hexpand(true);

    let create_column = |title: &str, property_name: &str| {
        let factory = gtk::SignalListItemFactory::new();
        let prop_name = property_name.to_string();
        let prop_name_css = property_name.to_string();
        
        factory.connect_setup(move |_, list_item| {
            let label = gtk::Label::builder()
                .halign(gtk::Align::Start)
                .margin_start(4)
                .margin_end(4)
                .margin_top(8)
                .margin_bottom(8)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .build();
            
            match prop_name_css.as_str() {
                "key" => label.add_css_class("key-label"),
                "mods" => label.add_css_class("mod-label"),
                "dispatcher" => label.add_css_class("dispatcher-label"),
                "args" => label.add_css_class("args-label"),
                _ => {}
            }

            if prop_name_css == "mods" {
                let box_layout = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                let warning_icon = gtk::Image::builder()
                    .icon_name("dialog-warning-symbolic")
                    .visible(false) // Hidden by default
                    .css_classes(["error"])
                    .tooltip_text("Conflicting keybind")
                    .build();
                
                box_layout.append(&warning_icon);
                box_layout.append(&label);
                
                list_item.set_child(Some(&box_layout));
            } else {
                list_item.set_child(Some(&label));
            }
        });

        factory.connect_bind(move |_, list_item| {
            let keybind = list_item
                .item()
                .and_downcast::<KeybindObject>()
                .expect("The item has to be an `KeybindObject`.");
            
            // Logic to handle both complex (Mods) and simple (others) columns
            let (label, icon_opt) = if prop_name == "mods" {
                 let box_layout = list_item.child().and_downcast::<gtk::Box>().expect("Child is not a Box");
                 let icon = box_layout.first_child().and_downcast::<gtk::Image>().expect("First child is not an Image");
                 let label = icon.next_sibling().and_downcast::<gtk::Label>().expect("Second child is not a Label");
                 (label, Some(icon))
            } else {
                 let label = list_item.child().and_downcast::<gtk::Label>().expect("Child is not a Label");
                 (label, None)
            };
            
            let mut binding_label_builder = keybind.bind_property(&prop_name, &label, "label")
                .sync_create();
            
            let mut binding_tooltip_builder = keybind.bind_property(&prop_name, &label, "tooltip-text")
                .sync_create();

            if prop_name == "args" || prop_name == "mods" {
                let transform = |_, val: glib::Value| {
                    let s = val.get::<String>().unwrap_or_default();
                    if let Some(stripped) = s.strip_prefix('$') {
                        Some(stripped.to_string().to_value())
                    } else {
                        Some(val)
                    }
                };
                binding_label_builder = binding_label_builder.transform_to(transform);
                binding_tooltip_builder = binding_tooltip_builder.transform_to(transform);
            }

            let binding_label = binding_label_builder.build();
            let binding_tooltip = binding_tooltip_builder.build();
            
            let mut bindings = vec![binding_label, binding_tooltip];
            
            if let Some(icon) = icon_opt {
                // Manually update the icon visibility based on conflict property
                // Since it's boolean, we can bind it directly?
                // bind_property handles the sync.
                let binding_icon = keybind.bind_property("is-conflicted", &icon, "visible")
                    .sync_create()
                    .build();
                bindings.push(binding_icon);
                
                // Also add .conflicted class to label if conflicted?
                // Let's stick to the icon for now, simpler and cleaner.
            }

            unsafe {
                list_item.set_data("bindings", bindings);
            }
        });
        
        factory.connect_unbind(move |_, list_item| {
            unsafe {
                if let Some(bindings) = list_item.steal_data::<Vec<glib::Binding>>("bindings") {
                    for b in bindings {
                        b.unbind();
                    }
                }
            }
        });

        gtk::ColumnViewColumn::builder()
            .title(title)
            .factory(&factory)
            .expand(true)
            .resizable(true)
            .build()
    };

    column_view.append_column(&create_column("Mods", "mods"));
    column_view.append_column(&create_column("Key", "key"));
    column_view.append_column(&create_column("Dispatcher", "dispatcher"));
    column_view.append_column(&create_column("Args", "args"));

    let model_store = model.clone();
    column_view.connect_activate(move |view, position| {
        let model = view.model().expect("ColumnView needs a model");
        
        let selection_model = model.clone().downcast::<gtk::SingleSelection>().ok();
        
        let item = if let Some(sel) = selection_model {
             sel.item(position)
        } else {
             model.item(position)
        };
        
        if let Some(obj) = item.and_downcast::<KeybindObject>() {
            let current_args = obj.property::<String>("args");
            let current_mods = obj.property::<String>("mods");
            let current_key = obj.property::<String>("key");
            let current_dispatcher = obj.property::<String>("dispatcher");
            let line_number = obj.property::<u64>("line-number");
            
            if let Some(root) = view.root() {
                if let Some(window) = root.downcast_ref::<adw::ApplicationWindow>() {
                    show_edit_dialog(window, &current_mods, &current_key, &current_dispatcher, &current_args, line_number as usize, obj, &model_store);
                }
            }
        }
    });

    let scrolled_window = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .hexpand(true)
        .child(&column_view)
        .build();

    let search_entry = gtk::SearchEntry::builder()
        .placeholder_text("Search keybinds...")
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    search_entry.connect_search_changed(move |entry| {
        let text = entry.text().to_string().to_lowercase();
        filter.set_filter_func(move |obj| {
            let keybind = obj.downcast_ref::<KeybindObject>().unwrap();
            let mods = keybind.property::<String>("mods").to_lowercase();
            let key = keybind.property::<String>("key").to_lowercase();
            let dispatcher = keybind.property::<String>("dispatcher").to_lowercase();
            let args = keybind.property::<String>("args").to_lowercase();
            
            if text.is_empty() {
                return true;
            }
            
            mods.contains(&text) || key.contains(&text) || dispatcher.contains(&text) || args.contains(&text)
        });
    });

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    
    let header = adw::HeaderBar::new();
    
    content.append(&header);
    content.append(&search_entry);
    content.append(&scrolled_window);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("hyprKCS")
        .content(&content)
        .default_width(700)
        .default_height(500)
        .build();

    let controller = gtk::EventControllerKey::new();
    let window_clone = window.clone();
    controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk::gdk::Key::Escape {
            window_clone.close();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    window.add_controller(controller);

        window.present();

    }

    




fn show_edit_dialog(parent: &adw::ApplicationWindow, current_mods: &str, current_key: &str, current_dispatcher: &str, current_args: &str, line_number: usize, obj: KeybindObject, model: &gio::ListStore) {
    let (display_mods, mods_had_prefix) = if let Some(stripped) = current_mods.strip_prefix('$') {
        (stripped, true)
    } else {
        (current_mods, false)
    };

    let (display_args, args_had_prefix) = if let Some(stripped) = current_args.strip_prefix('$') {
        (stripped, true)
    } else {
        (current_args, false)
    };

    let dialog = gtk::Dialog::builder()
        .title("Edit Keybind")
        .transient_for(parent)
        .modal(true)
        .default_width(400)
        .build();

    let content_area = dialog.content_area();
    content_area.set_margin_top(12);
    content_area.set_margin_bottom(12);
    content_area.set_margin_start(12);
    content_area.set_margin_end(12);
    content_area.set_spacing(12);

    let label_mods = gtk::Label::new(Some("Modifiers:"));
    label_mods.set_halign(gtk::Align::Start);
    content_area.append(&label_mods);

    let entry_mods = gtk::Entry::builder()
        .text(display_mods)
        .activates_default(true)
        .build();
    if mods_had_prefix {
        entry_mods.set_placeholder_text(Some("Variable '$' will be added automatically"));
    }
    content_area.append(&entry_mods);

    let label_key = gtk::Label::new(Some("Key:"));
    label_key.set_halign(gtk::Align::Start);
    content_area.append(&label_key);

    let entry_key = gtk::Entry::builder()
        .text(current_key)
        .activates_default(true)
        .build();
    content_area.append(&entry_key);

    let label_dispatcher = gtk::Label::new(Some("Dispatcher:"));
    label_dispatcher.set_halign(gtk::Align::Start);
    content_area.append(&label_dispatcher);

    let entry_dispatcher = gtk::Entry::builder()
        .text(current_dispatcher)
        .activates_default(true)
        .build();
    content_area.append(&entry_dispatcher);

    let label_args = gtk::Label::new(Some("Arguments:"));
    label_args.set_halign(gtk::Align::Start);
    content_area.append(&label_args);

    let entry_args = gtk::Entry::builder()
        .text(display_args)
        .activates_default(true)
        .build();
    if args_had_prefix {
        entry_args.set_placeholder_text(Some("Variable '$' will be added automatically"));
    }
    content_area.append(&entry_args);

    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Save", gtk::ResponseType::Ok);
    dialog.set_default_response(gtk::ResponseType::Ok);

    let obj_clone = obj.clone();
    let model_clone = model.clone();
    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Ok {
            let input_mods = entry_mods.text().to_string();
            let new_mods = if mods_had_prefix {
                format!("${}", input_mods)
            } else {
                input_mods
            };

            let new_key = entry_key.text().to_string();
            let new_dispatcher = entry_dispatcher.text().to_string();

            let input_args = entry_args.text().to_string();
            let new_args = if args_had_prefix {
                format!("${}", input_args)
            } else {
                input_args
            };
            
            match parser::update_line(line_number, &new_mods, &new_key, &new_dispatcher, &new_args) {
                Ok(_) => {
                    obj_clone.set_property("mods", new_mods.to_value());
                    obj_clone.set_property("key", new_key.to_value());
                    obj_clone.set_property("dispatcher", new_dispatcher.to_value());
                    obj_clone.set_property("args", new_args.to_value());
                    
                    refresh_conflicts(&model_clone);
                }
                Err(e) => {
                    eprintln!("Failed to update config: {}", e);
                    let err_dialog = gtk::MessageDialog::builder()
                        .transient_for(dialog)
                        .modal(true)
                        .message_type(gtk::MessageType::Error)
                        .buttons(gtk::ButtonsType::Ok)
                        .text(format!("Failed to save changes: {}", e))
                        .build();
                    err_dialog.connect_response(|d, _| d.close());
                    err_dialog.present();
                    return;
                }
            }
        }
        dialog.close();
    });

    dialog.present();
}

fn refresh_conflicts(model: &gio::ListStore) {
    let mut counts = std::collections::HashMap::new();
    
    // First pass: count occurrences
    for i in 0..model.n_items() {
        if let Some(obj) = model.item(i).and_downcast::<KeybindObject>() {
            let mods = obj.property::<String>("mods");
            let key = obj.property::<String>("key");
            
            // We don't have access to 'clean_mods' property directly if it's not exposed as GObject property?
            // Wait, I didn't add 'clean_mods' as a GObject property in keybind_object.rs!
            // I only added it to the struct `Keybind` in parser.rs.
            // But KeybindObject wraps it.
            // Let's check `src/keybind_object.rs`.
            
            // Checking keybind_object.rs... 
            // It has properties: mods, key, dispatcher, args, line-number, is-conflicted.
            // It assumes `mods` property holds the string.
            // But `mods` property is initialized with `keybind.mods` which is the DISPLAY string (e.g. "[l] SUPER").
            // So we are comparing "[l] SUPER" with "SUPER" if we are not careful?
            // Actually, in `KeybindObject::new`, we passed `keybind.mods`.
            
            // Ideally, we should add `clean_mods` as a hidden property to KeybindObject for accurate comparison.
            // OR, we can try to strip flags here.
            
            // For now, let's just use the `mods` property. It usually contains the full string.
            // If the user edits it, they edit the `mods` property directly.
            // The display mods might contain flags like `[l]`.
            // If one bind is `SUPER, Q` and another is `[l] SUPER, Q`, are they conflicting?
            // Yes, flags usually modify behavior but the key combo is the same.
            
            // Normalizing: remove anything in brackets `[...]` and trim.
            let clean_mods = if let Some(idx) = mods.find(']') {
                if let Some(start) = mods.find('[') {
                     if start < idx {
                         mods[idx+1..].trim().to_string()
                     } else {
                         mods
                     }
                } else {
                    mods
                }
            } else {
                mods
            };
            
            let key_tuple = (clean_mods.to_lowercase(), key.to_lowercase());
            *counts.entry(key_tuple).or_insert(0) += 1;
        }
    }
    
    // Second pass: update is-conflicted
    for i in 0..model.n_items() {
        if let Some(obj) = model.item(i).and_downcast::<KeybindObject>() {
            let mods = obj.property::<String>("mods");
            let key = obj.property::<String>("key");
            
            let clean_mods = if let Some(idx) = mods.find(']') {
                if let Some(start) = mods.find('[') {
                     if start < idx {
                         mods[idx+1..].trim().to_string()
                     } else {
                         mods
                     }
                } else {
                    mods
                }
            } else {
                mods
            };
            
            let count = counts.get(&(clean_mods.to_lowercase(), key.to_lowercase())).unwrap_or(&0);
            obj.set_property("is-conflicted", *count > 1);
        }
    }
}