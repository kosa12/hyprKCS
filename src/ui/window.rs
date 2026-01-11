use gtk4 as gtk;
use gtk::{gio, glib, prelude::*};
use libadwaita as adw;
use gtk4_layer_shell::{Layer, LayerShell, KeyboardMode};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use chrono::Local;
use std::fs;
use crate::parser;
use crate::keybind_object::KeybindObject;
use crate::ui::views::{create_add_view, create_edit_view};

pub fn build_ui(app: &adw::Application) {
    let model = gio::ListStore::new::<KeybindObject>();
    crate::ui::utils::reload_keybinds(&model);
    
    let filter = gtk::CustomFilter::new(|_obj| true);
    let filter_model = gtk::FilterListModel::new(Some(model.clone()), Some(filter.clone()));
    let selection_model = gtk::SingleSelection::new(Some(filter_model.clone()));

    let column_view = gtk::ColumnView::new(Some(selection_model.clone()));
    column_view.set_show_row_separators(false); 
    column_view.set_show_column_separators(false);
    column_view.set_vexpand(true);

    let create_column = |title: &str, property_name: &str| {
        let factory = gtk::SignalListItemFactory::new();
        let prop_name = property_name.to_string();
        let prop_name_css = property_name.to_string();
        
        factory.connect_setup(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let label = gtk::Label::builder()
                .halign(gtk::Align::Start)
                .margin_start(8)
                .margin_end(8)
                .margin_top(4)
                .margin_bottom(4)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .build();
            
            match prop_name_css.as_str() {
                "key" => label.add_css_class("key-label"),
                "mods" => label.add_css_class("mod-label"),
                "dispatcher" => label.add_css_class("dispatcher-label"),
                "args" => label.add_css_class("args-label"),
                "submap" => label.add_css_class("submap-label"),
                _ => {}
            }

            if prop_name_css == "mods" {
                let box_layout = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                let warning_icon = gtk::Image::builder()
                    .icon_name("dialog-warning-symbolic")
                    .visible(false)
                    .css_classes(["error-icon"])
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
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let keybind = list_item.item().and_downcast::<KeybindObject>().unwrap();
            
            let (label, icon_opt) = if prop_name == "mods" {
                 let box_layout = list_item.child().and_downcast::<gtk::Box>().unwrap();
                 let icon = box_layout.first_child().and_downcast::<gtk::Image>().unwrap();
                 let label = icon.next_sibling().and_downcast::<gtk::Label>().unwrap();
                 (label, Some(icon))
            } else {
                 let label = list_item.child().and_downcast::<gtk::Label>().unwrap();
                 (label, None)
            };
            
            let text = keybind.property::<String>(&prop_name);
            label.set_label(&text);
            label.set_tooltip_text(Some(&text));

            if prop_name == "submap" {
                 let submap_val = keybind.property::<String>("submap");
                 label.set_visible(!submap_val.is_empty());
            }

            if let Some(icon) = icon_opt {
                let is_conflicted = keybind.property::<bool>("is-conflicted");
                let reason = keybind.property::<String>("conflict-reason");
                icon.set_visible(is_conflicted);
                icon.set_tooltip_text(Some(&reason));
            }
        });

        gtk::ColumnViewColumn::builder()
            .title(title)
            .factory(&factory)
            .expand(true)
            .build()
    };
    column_view.append_column(&create_column("Modifiers", "mods"));
    column_view.append_column(&create_column("Key", "key"));
    column_view.append_column(&create_column("Action", "dispatcher"));
    column_view.append_column(&create_column("Arguments", "args"));
    column_view.append_column(&create_column("Submap", "submap"));

    // Compact Top Bar Layout
    let search_entry = gtk::SearchEntry::builder()
        .placeholder_text("Search keybinds...")
        .hexpand(true)
        .build();

    let add_button = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Add New Keybind")
        .css_classes(["flat"])
        .build();

    let backup_button = gtk::Button::builder()
        .icon_name("document-save-symbolic")
        .tooltip_text("Backup Current Config")
        .css_classes(["flat"])
        .build();
    
    let categories = gtk::StringList::new(&["All", "Workspace", "Window", "Media", "Custom"]);
    let category_dropdown = gtk::DropDown::builder()
        .model(&categories)
        .selected(0)
        .tooltip_text("Filter by Category")
        .build();

    let top_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();
    
    top_box.append(&category_dropdown);
    top_box.append(&search_entry);
    top_box.append(&add_button);
    top_box.append(&backup_button);

    // Status Page (Empty State)
    let status_page = adw::StatusPage::builder()
        .title("No Keybinds Found")
        .description("Try a different search term or add a new keybind.")
        .icon_name("system-search-symbolic")
        .vexpand(true)
        .visible(false)
        .build();

    let scrolled_window = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&column_view)
        .vexpand(true)
        .build();

    let list_stack = gtk::Stack::builder()
        .transition_type(gtk::StackTransitionType::Crossfade)
        .build();
    list_stack.add_child(&scrolled_window);
    list_stack.add_child(&status_page);

    let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    main_vbox.append(&top_box);
    main_vbox.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    main_vbox.append(&list_stack);

    // ROOT STACK (Switches between HOME, ADD, EDIT)
    let root_stack = gtk::Stack::builder()
        .transition_type(gtk::StackTransitionType::SlideLeftRight)
        .build();
    
    // Add "Home" page
    root_stack.add_named(&main_vbox, Some("home"));

    // Pages for Add/Edit (containers)
    let add_page_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root_stack.add_named(&add_page_container, Some("add"));

    let edit_page_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root_stack.add_named(&edit_page_container, Some("edit"));

    let window_content = gtk::Box::builder()
        .css_classes(["window-content"])
        .vexpand(true)
        .hexpand(true)
        .build();
    window_content.append(&root_stack);

    let toast_overlay = adw::ToastOverlay::new();
    toast_overlay.set_child(Some(&window_content));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(700)
        .default_height(500)
        .content(&toast_overlay)
        .decorated(false)
        .startup_id("hyprkcs-menu")
        .build();

    // Initialize Layer Shell
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    window.add_css_class("menu-window");

    let controller = gtk::EventControllerKey::new();
    controller.set_propagation_phase(gtk::PropagationPhase::Capture);
    let search_entry_focus = search_entry.clone();
    let window_clone = window.clone();
    let root_stack_c = root_stack.clone();

    let selection_model_key = selection_model.clone();
    let model_key = model.clone();
    let toast_overlay_key = toast_overlay.clone();
    let edit_page_container_key = edit_page_container.clone();

    controller.connect_key_pressed(move |_, key, _, mods| {
        if mods.contains(gtk::gdk::ModifierType::CONTROL_MASK) && key == gtk::gdk::Key::f {
            search_entry_focus.grab_focus();
            return glib::Propagation::Stop;
        }

        let home_visible = root_stack_c.visible_child_name().as_deref() == Some("home");
        let search_focused = search_entry_focus.has_focus();

        if home_visible && !search_focused {
            if mods.is_empty() {
                match key {
                    gtk::gdk::Key::slash => {
                        search_entry_focus.grab_focus();
                        return glib::Propagation::Stop;
                    }
                    gtk::gdk::Key::Return => {
                       if let Some(obj) = selection_model_key.selected_item().and_downcast::<KeybindObject>() {
                           while let Some(child) = edit_page_container_key.first_child() {
                               edit_page_container_key.remove(&child);
                           }
                           let edit_view = create_edit_view(
                               &root_stack_c,
                               obj,
                               &model_key,
                               &toast_overlay_key,
                               &edit_page_container_key
                           );
                           edit_page_container_key.append(&edit_view);
                           root_stack_c.set_visible_child_name("edit");
                           return glib::Propagation::Stop;
                       }
                    }
                    _ => {}
                }
            }
        }

        if key == gtk::gdk::Key::Escape {
            if root_stack_c.visible_child_name().as_deref() != Some("home") {
                root_stack_c.set_visible_child_name("home");
                return glib::Propagation::Stop;
            }
            if !search_entry_focus.text().is_empty() {
                search_entry_focus.set_text("");
                return glib::Propagation::Stop;
            }
            window_clone.close();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    window.add_controller(controller);

    let model_store = model.clone();
    let toast_overlay_activate = toast_overlay.clone();
    let root_stack_edit = root_stack.clone();
    let edit_page_container_c = edit_page_container.clone();

    column_view.connect_activate(move |view, position| {
        let selection = view.model().unwrap().downcast::<gtk::SingleSelection>().unwrap();
        if let Some(obj) = selection.item(position).and_downcast::<KeybindObject>() {
            // Clear previous edit form
             while let Some(child) = edit_page_container_c.first_child() {
                edit_page_container_c.remove(&child);
            }
            
            let edit_view = create_edit_view(
                &root_stack_edit,
                obj,
                &model_store,
                &toast_overlay_activate,
                &edit_page_container_c
            );
            edit_page_container_c.append(&edit_view);
            root_stack_edit.set_visible_child_name("edit");
        }
    });

    let model_clone_add = model.clone();
    let toast_overlay_add = toast_overlay.clone();
    let root_stack_add = root_stack.clone();
    let add_page_container_c = add_page_container.clone();

    add_button.connect_clicked(move |_| {
         // Clear previous add form (optional but good for reset)
         while let Some(child) = add_page_container_c.first_child() {
            add_page_container_c.remove(&child);
        }

        let add_view = create_add_view(
            &root_stack_add,
            &model_clone_add,
            &toast_overlay_add
        );
        add_page_container_c.append(&add_view);
        root_stack_add.set_visible_child_name("add");
    });

    let toast_overlay_backup = toast_overlay.clone();
    backup_button.connect_clicked(move |_| {
        match parser::get_config_path() {
            Ok(path) => {
                let now = Local::now();
                let params = now.format("%Y-%m-%d_%H-%M-%S").to_string();
                let backup_path = path.with_extension(format!("conf.{}.bak", params));
                
                if let Err(e) = fs::copy(&path, &backup_path) {
                     let toast = adw::Toast::new(&format!("Backup failed: {}", e));
                     toast_overlay_backup.add_toast(toast);
                } else {
                     let toast = adw::Toast::new(&format!("Config backed up to {:?}", backup_path.file_name().unwrap()));
                     toast_overlay_backup.add_toast(toast);
                }
            },
            Err(e) => {
                 let toast = adw::Toast::new(&format!("Could not find config path: {}", e));
                 toast_overlay_backup.add_toast(toast);
            }
        }
    });

    let status_page_ref = status_page.clone();
    let list_stack_ref = list_stack.clone();
    let scrolled_ref = scrolled_window.clone();

    filter_model.connect_items_changed(move |m, _, _, _| {
        let has_items = m.n_items() > 0;
        status_page_ref.set_visible(!has_items);
        scrolled_ref.set_visible(has_items);
        if has_items {
            list_stack_ref.set_visible_child(&scrolled_ref);
        } else {
            list_stack_ref.set_visible_child(&status_page_ref);
        }
    });

    let filter_func = move |text: String, category: u32| {
        let matcher = SkimMatcherV2::default();

        filter.set_filter_func(move |obj| {
            let kb = obj.downcast_ref::<KeybindObject>().unwrap();
            let mods = kb.property::<String>("mods");
            let key = kb.property::<String>("key");
            let dispatcher = kb.property::<String>("dispatcher").to_lowercase();
            let args = kb.property::<String>("args").to_lowercase();
            
            // Category Filter
            let category_match = match category {
                0 => true, // All
                1 => dispatcher.contains("workspace") || dispatcher.contains("movetoworkspace"),
                2 => dispatcher.contains("window") || dispatcher.contains("active") || dispatcher.contains("focus") || dispatcher.contains("fullscreen") || dispatcher.contains("group") || dispatcher.contains("split") || dispatcher.contains("pin"),
                3 => args.contains("volume") || args.contains("brightness") || args.contains("playerctl") || dispatcher.contains("audio"),
                4 => dispatcher == "exec", // Custom/Script
                _ => true,
            };

            if !category_match {
                return false;
            }

            if text.is_empty() {
                return true;
            }

            matcher.fuzzy_match(&mods, &text).is_some() ||
            matcher.fuzzy_match(&key, &text).is_some() ||
            matcher.fuzzy_match(&dispatcher, &text).is_some() ||
            matcher.fuzzy_match(&args, &text).is_some()
        });
    };

    let filter_func_1 = std::rc::Rc::new(filter_func);
    let filter_func_2 = filter_func_1.clone();
    
    let dropdown_ref = category_dropdown.clone();
    search_entry.connect_search_changed(move |entry| {
        let text = entry.text().to_string();
        let cat = dropdown_ref.selected();
        filter_func_1(text, cat);
    });

    let search_entry_ref = search_entry.clone();
    category_dropdown.connect_selected_notify(move |dropdown| {
        let text = search_entry_ref.text().to_string();
        let cat = dropdown.selected();
        filter_func_2(text, cat);
    });

    window.present();
    search_entry.grab_focus();
}
