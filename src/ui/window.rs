use crate::config::favorites::{load_favorites, save_favorites, toggle_favorite, FavoriteKeybind};
use crate::config::StyleConfig;
use crate::keybind_object::KeybindObject;
use crate::ui::utils::{create_flat_button, reload_keybinds, SearchQuery};
use crate::ui::views::{create_add_view, create_edit_view};
use crate::ui::wizards::{create_bulk_replace_wizard, create_conflict_wizard};
use fuzzy_matcher::skim::SkimMatcherV2;
use gtk::{gio, glib, prelude::*};
use gtk4 as gtk;
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use libadwaita as adw;

pub fn build_ui(app: &adw::Application) {
    if let Some(window) = app.active_window() {
        window.present();
        return;
    }

    let config = StyleConfig::load();
    let model = gio::ListStore::new::<KeybindObject>();
    reload_keybinds(&model);

    let filter = gtk::CustomFilter::new(|_obj| true);
    let filter_model = gtk::FilterListModel::new(Some(model.clone()), Some(filter.clone()));

    // Shared callback for refreshing the filter
    let refresh_filter_callback: std::rc::Rc<std::cell::RefCell<Option<Box<dyn Fn()>>>> =
        std::rc::Rc::new(std::cell::RefCell::new(None));

    let column_view = gtk::ColumnView::new(None::<gtk::SelectionModel>);
    let sort_model = gtk::SortListModel::new(Some(filter_model.clone()), column_view.sorter());
    let selection_model = gtk::SingleSelection::new(Some(sort_model.clone()));
    column_view.set_model(Some(&selection_model));

    column_view.set_show_row_separators(false);
    column_view.set_show_column_separators(false);
    column_view.set_vexpand(true);

    // --- Favorites Column ---
    let col_fav = gtk::ColumnViewColumn::builder()
        .title("")
        .expand(false)
        .fixed_width(40)
        .build();

    let factory_fav = gtk::SignalListItemFactory::new();

    let refresh_c = refresh_filter_callback.clone();
    factory_fav.connect_setup(move |_, list_item| {
        let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
        let btn = gtk::Button::builder()
            .css_classes(["flat", "circular"])
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .build();

        // Handle Click
        let list_item_weak = list_item.downgrade();
        let refresh_c = refresh_c.clone();
        btn.connect_clicked(move |b| {
            if let Some(list_item) = list_item_weak.upgrade() {
                if let Some(obj) = list_item.item().and_downcast::<KeybindObject>() {
                    let item = obj.with_data(|d| FavoriteKeybind {
                        mods: d.clean_mods.to_string(),
                        key: d.key.to_string(),
                        submap: d.submap.as_ref().map(|s| s.to_string()).unwrap_or_default(),
                        dispatcher: d.dispatcher.to_string(),
                        args: d.args.as_ref().map(|s| s.to_string()).unwrap_or_default(),
                    });

                    let mut favs = load_favorites();
                    let new_state = toggle_favorite(&mut favs, item);
                    let _ = save_favorites(&favs);

                    obj.set_property("is-favorite", new_state);

                    b.set_icon_name(if new_state {
                        "starred-symbolic"
                    } else {
                        "non-starred-symbolic"
                    });
                    if new_state {
                        b.add_css_class("warning");
                    } else {
                        b.remove_css_class("warning");
                    }

                    // Trigger filter refresh to update list immediately if filtering by favorites
                    if let Some(callback) = refresh_c.borrow().as_ref() {
                        callback();
                    }
                }
            }
        });

        list_item.set_child(Some(&btn));
    });

    factory_fav.connect_bind(move |_, list_item| {
        let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
        let btn = list_item.child().and_downcast::<gtk::Button>().unwrap();
        let keybind = list_item.item().and_downcast::<KeybindObject>().unwrap();

        let is_fav = keybind.with_data(|d| d.is_favorite);

        btn.set_icon_name(if is_fav {
            "starred-symbolic"
        } else {
            "non-starred-symbolic"
        });
        if is_fav {
            btn.add_css_class("warning");
        } else {
            btn.remove_css_class("warning");
        }
    });

    col_fav.set_factory(Some(&factory_fav));
    col_fav.set_visible(config.show_favorites);
    column_view.append_column(&col_fav);
    // -------------------------

    let create_column = move |title: &str, property_name: &str, sort_prop: Option<&str>| {
        let factory = gtk::SignalListItemFactory::new();
        let prop_name = property_name.to_string();
        let prop_name_css = property_name.to_string();

        let prop_name_css_clone = prop_name_css.clone();
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

            match prop_name_css_clone.as_str() {
                "key" => label.add_css_class("key-label"),
                "mods" => label.add_css_class("mod-label"),
                "dispatcher" => label.add_css_class("dispatcher-label"),
                "args" => label.add_css_class("args-label"),
                "submap" => label.add_css_class("submap-label"),
                "description" => label.add_css_class("description-label"),
                _ => {}
            }

            if prop_name_css_clone == "mods" {
                let box_layout = gtk::Box::new(gtk::Orientation::Horizontal, 8);

                // Conflict Icon (Yellow Warning)
                let warning_icon = gtk::Image::builder()
                    .icon_name("dialog-warning-symbolic")
                    .visible(false)
                    .css_classes(["error-icon"])
                    .tooltip_text("Conflicting keybind")
                    .build();

                // Broken Icon (Red Error)
                let broken_icon = gtk::Image::builder()
                    .icon_name("dialog-error-symbolic")
                    .visible(false)
                    .css_classes(["destructive-action"])
                    .tooltip_text("Broken keybind")
                    .build();

                box_layout.append(&broken_icon);
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

            let (label, icon_opt, broken_icon_opt) = if prop_name == "mods" {
                let box_layout = list_item.child().and_downcast::<gtk::Box>().unwrap();
                let broken_icon = box_layout
                    .first_child()
                    .and_downcast::<gtk::Image>()
                    .unwrap();
                let warning_icon = broken_icon
                    .next_sibling()
                    .and_downcast::<gtk::Image>()
                    .unwrap();
                let label = warning_icon
                    .next_sibling()
                    .and_downcast::<gtk::Label>()
                    .unwrap();
                (label, Some(warning_icon), Some(broken_icon))
            } else {
                let label = list_item.child().and_downcast::<gtk::Label>().unwrap();
                (label, None, None)
            };

            keybind.with_data(|data| {
                let text = match prop_name.as_str() {
                    "mods" => data.mods.as_ref(),
                    "key" => data.key.as_ref(),
                    "dispatcher" => data.dispatcher.as_ref(),
                    "args" => data.args.as_deref().unwrap_or(""),
                    "submap" => data.submap.as_deref().unwrap_or(""),
                    "description" => data.description.as_deref().unwrap_or(""),
                    "clean-mods" => data.clean_mods.as_ref(),
                    _ => "",
                };
                label.set_label(text);
                label.set_tooltip_text(Some(text));

                if prop_name == "submap" {
                    label.set_visible(data.submap.is_some());
                }

                if let Some(icon) = icon_opt {
                    icon.set_visible(data.is_conflicted);
                    if let Some(reason) = data.conflict_reason.as_deref() {
                        icon.set_tooltip_text(Some(reason));
                    }
                }

                if let Some(icon) = broken_icon_opt {
                    icon.set_visible(data.is_broken);
                    if let Some(reason) = data.broken_reason.as_deref() {
                        icon.set_tooltip_text(Some(reason));
                    }
                }
            });
        });

        let column = gtk::ColumnViewColumn::builder()
            .title(title)
            .factory(&factory)
            .expand(true)
            .build();

        if let Some(sp) = sort_prop {
            let sorter = gtk::StringSorter::builder()
                .expression(gtk::PropertyExpression::new(
                    KeybindObject::static_type(),
                    None::<gtk::Expression>,
                    sp,
                ))
                .build();
            column.set_sorter(Some(&sorter));
        }

        column
    };

    let col_mods = create_column("Modifiers", "mods", Some("clean-mods"));
    let col_key = create_column("Key", "key", Some("key"));
    let col_disp = create_column("Action", "dispatcher", Some("dispatcher"));

    column_view.append_column(&col_mods);
    column_view.append_column(&col_key);
    column_view.append_column(&col_disp);

    let mut default_sort_col = match config.default_sort.as_str() {
        "mods" | "modifiers" => Some(col_mods.clone()),
        "key" => Some(col_key.clone()),
        "dispatcher" | "action" => Some(col_disp.clone()),
        _ => None,
    };

    let col_args = create_column("Arguments", "args", Some("args"));
    col_args.set_visible(config.show_args);
    column_view.append_column(&col_args);
    if config.default_sort == "args" || config.default_sort == "arguments" {
        default_sort_col = Some(col_args.clone());
    }

    let col_desc = create_column("Description", "description", Some("description"));
    column_view.append_column(&col_desc);
    col_desc.set_visible(config.show_description);

    if config.show_description {
        if config.default_sort == "description" || config.default_sort == "desc" {
            default_sort_col = Some(col_desc.clone());
        }
    }

    let col_submap = create_column("Submap", "submap", Some("submap"));
    col_submap.set_visible(config.show_submaps);
    column_view.append_column(&col_submap);
    if config.default_sort == "submap" {
        default_sort_col = Some(col_submap.clone());
    }

    if let Some(col) = default_sort_col {
        column_view.sort_by_column(Some(&col), gtk::SortType::Ascending);
    }

    // Ensure the first row is selected/focused after sorting
    selection_model.set_selected(0);
    column_view.scroll_to(
        0,
        None::<&gtk::ColumnViewColumn>,
        gtk::ListScrollFlags::FOCUS | gtk::ListScrollFlags::SELECT,
        None::<gtk::ScrollInfo>,
    );

    // Compact Top Bar Layout
    let search_entry = gtk::SearchEntry::builder()
        .placeholder_text("Search keybinds...")
        .hexpand(true)
        .build();

    let add_button = create_flat_button("list-add-symbolic", "Add New Keybind");
    let bulk_button = create_flat_button("edit-find-replace-symbolic", "Bulk Replace");
    let backup_button = create_flat_button("document-save-symbolic", "Backup Current Config");
    let settings_button = create_flat_button("emblem-system-symbolic", "Settings");
    let keyboard_button = create_flat_button("input-keyboard-symbolic", "Visual Keyboard");

    let conflict_button = gtk::Button::builder()
        .icon_name("dialog-warning-symbolic")
        .label("Resolve Conflicts")
        .css_classes(["destructive-action"])
        .visible(false)
        .build();

    let mut cat_list = vec!["All", "Workspace", "Window", "Media", "Custom", "Mouse"];
    if config.show_favorites {
        cat_list.push("Favorites");
    }
    let categories = gtk::StringList::new(&cat_list);
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
    top_box.append(&conflict_button);
    top_box.append(&add_button);
    top_box.append(&bulk_button);
    top_box.append(&backup_button);
    top_box.append(&keyboard_button);
    top_box.append(&settings_button);

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

    // ROOT STACK (Switches between HOME, ADD, EDIT, WIZARD)
    let root_stack = gtk::Stack::builder()
        .transition_type(gtk::StackTransitionType::SlideLeftRight)
        .build();

    // Add "Home" page
    root_stack.add_named(&main_vbox, Some("home"));

    // Pages for Add/Edit/Wizard (containers)
    let add_page_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root_stack.add_named(&add_page_container, Some("add"));

    let edit_page_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root_stack.add_named(&edit_page_container, Some("edit"));

    let wizard_page_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root_stack.add_named(&wizard_page_container, Some("wizard"));

    let restore_page_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root_stack.add_named(&restore_page_container, Some("restore"));

    let settings_page_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root_stack.add_named(&settings_page_container, Some("settings"));

    let keyboard_page_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root_stack.add_named(&keyboard_page_container, Some("keyboard"));

    let window_content = gtk::Box::builder()
        .css_classes(["window-content"])
        .vexpand(true)
        .hexpand(true)
        .build();
    window_content.append(&root_stack);

    let toast_overlay = adw::ToastOverlay::new();
    toast_overlay.set_child(Some(&window_content));

    // Log configuration errors to stderr
    for error in &config.errors {
        eprintln!("[Config Error] {}", error);
    }

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .content(&toast_overlay)
        .decorated(false)
        .startup_id("hyprkcs-menu")
        .build();

    // Enforce size from config
    window.set_default_size(config.width, config.height);
    // For layer shell floating surfaces, size request is often needed
    window.set_size_request(config.width, config.height);

    // Initialize Layer Shell
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    window.add_css_class("menu-window");

    let controller = gtk::EventControllerKey::new();
    controller.set_propagation_phase(gtk::PropagationPhase::Capture);
    let search_entry_focus = search_entry.clone();
    let window_weak = window.downgrade();
    let root_stack_c = root_stack.clone();
    let column_view_focus = column_view.clone();

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

        if home_visible && search_focused {
            if key == gtk::gdk::Key::Down {
                column_view_focus.grab_focus();
                return glib::Propagation::Stop;
            }
        }

        if home_visible && !search_focused {
            if mods.is_empty() {
                match key {
                    gtk::gdk::Key::slash => {
                        search_entry_focus.grab_focus();
                        return glib::Propagation::Stop;
                    }
                    gtk::gdk::Key::Return => {
                        if let Some(obj) = selection_model_key
                            .selected_item()
                            .and_downcast::<KeybindObject>()
                        {
                            while let Some(child) = edit_page_container_key.first_child() {
                                edit_page_container_key.remove(&child);
                            }
                            let edit_view = create_edit_view(
                                &root_stack_c,
                                obj,
                                &model_key,
                                &column_view_focus,
                                &selection_model_key,
                                &toast_overlay_key,
                                &edit_page_container_key,
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
            if let Some(w) = window_weak.upgrade() {
                w.close();
            }
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
        let selection = view
            .model()
            .unwrap()
            .downcast::<gtk::SingleSelection>()
            .unwrap();
        if let Some(obj) = selection.item(position).and_downcast::<KeybindObject>() {
            // Clear previous edit form
            while let Some(child) = edit_page_container_c.first_child() {
                edit_page_container_c.remove(&child);
            }

            let edit_view = create_edit_view(
                &root_stack_edit,
                obj,
                &model_store,
                view,
                &selection,
                &toast_overlay_activate,
                &edit_page_container_c,
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

        let add_view = create_add_view(&root_stack_add, &model_clone_add, &toast_overlay_add);
        add_page_container_c.append(&add_view);
        root_stack_add.set_visible_child_name("add");
    });

    let model_bulk = model.clone();
    let toast_bulk = toast_overlay.clone();
    let stack_bulk = root_stack.clone();
    let wizard_container_bulk = wizard_page_container.clone();

    bulk_button.connect_clicked(move |_| {
        while let Some(child) = wizard_container_bulk.first_child() {
            wizard_container_bulk.remove(&child);
        }
        let view = create_bulk_replace_wizard(&stack_bulk, &model_bulk, &toast_bulk);
        wizard_container_bulk.append(&view);
        stack_bulk.set_visible_child_name("wizard");
    });

    let toast_overlay_backup = toast_overlay.clone();
    backup_button.connect_clicked(move |_| match crate::ui::utils::perform_backup(true) {
        Ok(msg) => {
            let toast = adw::Toast::new(&msg);
            toast_overlay_backup.add_toast(toast);
        }
        Err(e) => {
            let toast = adw::Toast::new(&format!("Backup failed: {}", e));
            toast_overlay_backup.add_toast(toast);
        }
    });

    // Logic to update conflict button visibility
    let update_conflict_btn = {
        let conflict_button = conflict_button.clone();
        move |model: &gio::ListStore| {
            let mut conflict_count = 0;
            for i in 0..model.n_items() {
                if let Some(obj) = model.item(i).and_downcast::<KeybindObject>() {
                    if obj.with_data(|d| d.is_conflicted) {
                        conflict_count += 1;
                    }
                }
            }

            conflict_button.set_visible(conflict_count > 0);
            if conflict_count > 0 {
                conflict_button.set_label(&format!("Resolve Conflicts ({})", conflict_count));
            }
        }
    };

    // Initial check
    update_conflict_btn(&model);

    let update_conflict_btn_c = update_conflict_btn.clone();
    let _conflict_btn_model = model.clone();

    // HACK: ListStore doesn't expose "on content changed" easily for deep property changes unless we bind to them.
    // However, we reload the whole model on add/edit/delete, triggering `items-changed`.
    // We can hook into that.
    model.connect_items_changed(move |m, _, _, _| {
        update_conflict_btn_c(m);
    });

    let model_wizard = model.clone();
    let stack_wizard = root_stack.clone();
    let toast_wizard = toast_overlay.clone();
    let wizard_container_c = wizard_page_container.clone();

    let column_view_wizard = column_view.clone();
    conflict_button.connect_clicked(move |_| {
        while let Some(child) = wizard_container_c.first_child() {
            wizard_container_c.remove(&child);
        }

        let wizard_view = create_conflict_wizard(
            &stack_wizard,
            &model_wizard,
            &column_view_wizard,
            &selection_model,
            &toast_wizard,
            &wizard_container_c,
            0,
        );
        wizard_container_c.append(&wizard_view);
        stack_wizard.set_visible_child_name("wizard");
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

    let matcher = std::rc::Rc::new(SkimMatcherV2::default());

    let filter_func = move |text: String, category: u32| {
        let query = SearchQuery::parse(&text);
        let m = matcher.clone();

        filter.set_filter_func(move |obj| {
            let kb = obj.downcast_ref::<KeybindObject>().unwrap();
            kb.matches_query(&query, category, &*m)
        });
    };

    let filter_func_1 = std::rc::Rc::new(filter_func);
    let filter_func_2 = filter_func_1.clone();
    let filter_func_3 = filter_func_1.clone();

    // Populate the shared refresh callback
    let search_entry_refresh = search_entry.clone();
    let dropdown_refresh = category_dropdown.clone();
    *refresh_filter_callback.borrow_mut() = Some(Box::new(move || {
        let text = search_entry_refresh.text().to_string();
        let cat = dropdown_refresh.selected();
        filter_func_3(text, cat);
    }));

    let dropdown_ref = category_dropdown.clone();
    let timeout_handle: std::rc::Rc<std::cell::RefCell<Option<glib::SourceId>>> =
        std::rc::Rc::new(std::cell::RefCell::new(None));

    search_entry.connect_search_changed(move |entry| {
        if let Some(source) = timeout_handle.borrow_mut().take() {
            source.remove();
        }

        let text = entry.text().to_string();
        let cat = dropdown_ref.selected();
        let filter_func = filter_func_1.clone();
        let timeout_handle_clone = timeout_handle.clone();

        let source = glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
            filter_func(text.clone(), cat);
            *timeout_handle_clone.borrow_mut() = None;
            glib::ControlFlow::Break
        });
        *timeout_handle.borrow_mut() = Some(source);
    });

    let search_entry_ref = search_entry.clone();
    category_dropdown.connect_selected_notify(move |dropdown| {
        let text = search_entry_ref.text().to_string();
        let cat = dropdown.selected();
        filter_func_2(text, cat);
    });

    let stack_settings = root_stack.clone();
    let container_settings = settings_page_container.clone();
    let window_settings = window.clone();
    let col_desc_clone = col_desc.clone();
    let col_fav_clone = col_fav.clone();
    let col_args_clone = col_args.clone();
    let col_submap_clone = col_submap.clone();
    let col_key_clone = col_key.clone();
    let col_mods_clone = col_mods.clone();
    let col_disp_clone = col_disp.clone();
    let column_view_clone = column_view.clone();
    let model_settings = model.clone();
    let toast_overlay_settings = toast_overlay.clone();
    let restore_container_settings = restore_page_container.clone();
    let dropdown_settings = category_dropdown.clone();

    settings_button.connect_clicked(move |_| {
        while let Some(child) = container_settings.first_child() {
            container_settings.remove(&child);
        }
        let col_desc_c = col_desc_clone.clone();
        let col_fav_c = col_fav_clone.clone();
        let col_args_c = col_args_clone.clone();
        let col_submap_c = col_submap_clone.clone();

        let col_key_c = col_key_clone.clone();
        let col_mods_c = col_mods_clone.clone();
        let col_disp_c = col_disp_clone.clone();
        let col_args_sort_c = col_args_clone.clone();
        let col_submap_sort_c = col_submap_clone.clone();
        let col_view_c = column_view_clone.clone();
        let model_s = model_settings.clone();
        let model_s_restore = model_settings.clone();
        let toast_s = toast_overlay_settings.clone();
        let stack_s = stack_settings.clone();
        let restore_container_s = restore_container_settings.clone();
        let dropdown_s = dropdown_settings.clone();

        let toast_s_1 = toast_s.clone();
        let toast_s_2 = toast_s.clone();

        let view = crate::ui::settings::create_settings_view(
            &window_settings,
            &stack_settings,
            &model_s,
            std::rc::Rc::new(move |s| col_desc_c.set_visible(s)),
            std::rc::Rc::new(move |s| {
                col_fav_c.set_visible(s);
                // Update dropdown options
                let mut cat_list = vec!["All", "Workspace", "Window", "Media", "Custom", "Mouse"];
                if s {
                    cat_list.push("Favorites");
                }
                // Preserve selection if possible, otherwise default to All
                let selected = dropdown_s.selected();
                let model = gtk::StringList::new(&cat_list);
                dropdown_s.set_model(Some(&model));
                if selected < model.n_items() {
                    dropdown_s.set_selected(selected);
                } else {
                    dropdown_s.set_selected(0);
                }
            }),
            std::rc::Rc::new(move |s| col_args_c.set_visible(s)),
            std::rc::Rc::new(move |s| col_submap_c.set_visible(s)),
            std::rc::Rc::new(move |sort_key| {
                let col = match sort_key.as_str() {
                    "mods" => Some(&col_mods_c),
                    "dispatcher" => Some(&col_disp_c),
                    "args" => Some(&col_args_sort_c),
                    "submap" => Some(&col_submap_sort_c),
                    _ => Some(&col_key_c), // Default key
                };
                if let Some(c) = col {
                    col_view_c.sort_by_column(Some(c), gtk::SortType::Ascending);
                }
            }),
            std::rc::Rc::new(move |msg| {
                let toast = adw::Toast::new(&msg);
                toast_s_1.add_toast(toast);
            }),
            std::rc::Rc::new(move || {
                while let Some(child) = restore_container_s.first_child() {
                    restore_container_s.remove(&child);
                }
                let restore_view = crate::ui::views::create_restore_view(
                    &stack_s,
                    &model_s_restore,
                    &toast_s_2,
                    &restore_container_s,
                );
                restore_container_s.append(&restore_view);
                stack_s.set_visible_child_name("restore");
            }),
        );
        container_settings.append(&view);
        stack_settings.set_visible_child_name("settings");
    });

    let stack_keyboard = root_stack.clone();
    let container_keyboard = keyboard_page_container.clone();
    let model_keyboard = model.clone();
    keyboard_button.connect_clicked(move |_| {
        while let Some(child) = container_keyboard.first_child() {
            container_keyboard.remove(&child);
        }
        let view = crate::ui::views::create_keyboard_view(&stack_keyboard, &model_keyboard);
        container_keyboard.append(&view);
        stack_keyboard.set_visible_child_name("keyboard");
    });

    window.present();
    search_entry.grab_focus();
}
