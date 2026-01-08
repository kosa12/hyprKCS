use gtk4 as gtk;
use gtk::{gio, glib, prelude::*};
use libadwaita as adw;
use crate::parser;
use crate::keybind_object::KeybindObject;
use crate::ui::dialogs::show_edit_dialog;

pub fn build_ui(app: &adw::Application) {
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
                let binding_icon = keybind.bind_property("is-conflicted", &icon, "visible")
                    .sync_create()
                    .build();
                bindings.push(binding_icon);
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
