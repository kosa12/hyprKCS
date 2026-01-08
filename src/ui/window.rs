use gtk4 as gtk;
use gtk::{gio, glib, prelude::*};
use libadwaita as adw;
use crate::parser;
use crate::keybind_object::KeybindObject;
use crate::ui::dialogs::{show_edit_dialog, show_add_dialog};

pub fn build_ui(app: &adw::Application) {
    let keybinds = parser::parse_config().unwrap_or_else(|err| {
        eprintln!("Error parsing config: {}", err);
        vec![]
    });

    // Detect conflicts
    let mut counts = std::collections::HashMap::new();
    for kb in &keybinds {
        let key = (kb.clean_mods.to_lowercase(), kb.key.to_lowercase());
        *counts.entry(key).or_insert(0) += 1;
    }

    let model = gio::ListStore::new::<KeybindObject>();
    for kb in keybinds {
        let count = counts.get(&(kb.clean_mods.to_lowercase(), kb.key.to_lowercase())).unwrap_or(&0);
        let is_conflicted = *count > 1;
        model.append(&KeybindObject::new(kb, is_conflicted));
    }

    let filter = gtk::CustomFilter::new(|_obj| true);
    let filter_model = gtk::FilterListModel::new(Some(model.clone()), Some(filter.clone()));
    let selection_model = gtk::SingleSelection::new(Some(filter_model.clone()));

    let column_view = gtk::ColumnView::new(Some(selection_model));
    column_view.set_show_row_separators(false); 
    column_view.set_show_column_separators(false);
    column_view.set_vexpand(true);

    let create_column = |title: &str, property_name: &str| {
        let factory = gtk::SignalListItemFactory::new();
        let prop_name = property_name.to_string();
        let prop_name_css = property_name.to_string();
        
        factory.connect_setup(move |_, list_item| {
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
            
            keybind.bind_property(&prop_name, &label, "label").sync_create().build();
            keybind.bind_property(&prop_name, &label, "tooltip-text").sync_create().build();

            if let Some(icon) = icon_opt {
                keybind.bind_property("is-conflicted", &icon, "visible").sync_create().build();
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

    let header_bar = adw::HeaderBar::new();
    let add_button = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Add New Keybind")
        .build();
    header_bar.pack_start(&add_button);

    let search_bar = gtk::SearchBar::builder()
        .valign(gtk::Align::Start)
        .key_capture_widget(&column_view)
        .build();
    
    let search_entry = gtk::SearchEntry::builder()
        .placeholder_text("Type to search keybinds...")
        .hexpand(true)
        .build();
    
    let clamp = adw::Clamp::builder()
        .maximum_size(600)
        .child(&search_entry)
        .build();

    search_bar.set_child(Some(&clamp));
    search_bar.set_search_mode(true);

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
    main_vbox.append(&header_bar);
    main_vbox.append(&search_bar);
    main_vbox.append(&list_stack);

    let toast_overlay = adw::ToastOverlay::new();
    toast_overlay.set_child(Some(&main_vbox));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("hyprKCS")
        .default_width(800)
        .default_height(600)
        .content(&toast_overlay)
        .build();

    let controller = gtk::EventControllerKey::new();
    let search_entry_focus = search_entry.clone();
    let window_clone = window.clone();
    controller.connect_key_pressed(move |_, key, _, mods| {
        if mods.contains(gtk::gdk::ModifierType::CONTROL_MASK) && key == gtk::gdk::Key::f {
            search_entry_focus.grab_focus();
            return glib::Propagation::Stop;
        }
        if key == gtk::gdk::Key::Escape {
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

    let window_activate = window.clone();
    let model_store = model.clone();
    let toast_overlay_activate = toast_overlay.clone();
    column_view.connect_activate(move |view, position| {
        let selection = view.model().unwrap().downcast::<gtk::SingleSelection>().unwrap();
        if let Some(obj) = selection.item(position).and_downcast::<KeybindObject>() {
            show_edit_dialog(
                &window_activate, 
                &obj.property::<String>("mods"), 
                &obj.property::<String>("key"), 
                &obj.property::<String>("dispatcher"), 
                &obj.property::<String>("args"), 
                obj.property::<u64>("line-number") as usize, 
                obj, 
                &model_store,
                toast_overlay_activate.clone()
            );
        }
    });

    let model_clone_add = model.clone();
    let toast_overlay_add = toast_overlay.clone();
    let window_add = window.clone();
    add_button.connect_clicked(move |_| {
        show_add_dialog(&window_add, model_clone_add.clone(), toast_overlay_add.clone());
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

    search_entry.connect_search_changed(move |entry| {
        let text = entry.text().to_string().to_lowercase();
        filter.set_filter_func(move |obj| {
            let kb = obj.downcast_ref::<KeybindObject>().unwrap();
            let mods = kb.property::<String>("mods").to_lowercase();
            let key = kb.property::<String>("key").to_lowercase();
            let dispatcher = kb.property::<String>("dispatcher").to_lowercase();
            let args = kb.property::<String>("args").to_lowercase();
            text.is_empty() || mods.contains(&text) || key.contains(&text) || dispatcher.contains(&text) || args.contains(&text)
        });
    });

    window.present();
}
