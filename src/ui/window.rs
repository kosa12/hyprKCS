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
use std::sync::Arc;

type FilterCallback = std::rc::Rc<std::cell::RefCell<Option<Box<dyn Fn()>>>>;

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

    let refresh_filter_callback: FilterCallback = std::rc::Rc::new(std::cell::RefCell::new(None));

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

    let refresh_c = std::rc::Rc::downgrade(&refresh_filter_callback);
    factory_fav.connect_setup(move |_, list_item| {
        let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
        let btn = gtk::Button::builder()
            .css_classes(["flat", "circular"])
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .build();

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
                    if let Some(callback_rc) = refresh_c.upgrade() {
                        if let Some(callback) = callback_rc.borrow().as_ref() {
                            callback();
                        }
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
        let is_mods = property_name == "mods";

        let prop_name_setup = prop_name.clone();
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

            match prop_name_setup.as_str() {
                "key" => label.add_css_class("key-label"),
                "mods" => label.add_css_class("mod-label"),
                "dispatcher" => label.add_css_class("dispatcher-label"),
                "args" => label.add_css_class("args-label"),
                "submap" => label.add_css_class("submap-label"),
                "description" => label.add_css_class("description-label"),
                _ => {}
            }

            if is_mods {
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

            let (label, icon_opt, broken_icon_opt) = if is_mods {
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

    if config.show_description
        && (config.default_sort == "description" || config.default_sort == "desc")
    {
        default_sort_col = Some(col_desc.clone());
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
    let search_entry_weak = search_entry.downgrade();
    let window_weak = window.downgrade();
    let root_stack_weak = root_stack.downgrade();
    let column_view_weak = column_view.downgrade();

    let selection_model_weak = selection_model.downgrade();
    let model_key = model.clone();
    let toast_overlay_weak = toast_overlay.downgrade();
    let edit_page_container_weak = edit_page_container.downgrade();

    controller.connect_key_pressed(move |_, key, _, mods| {
        let search_entry = match search_entry_weak.upgrade() {
            Some(w) => w,
            None => return glib::Propagation::Proceed,
        };
        let root_stack = match root_stack_weak.upgrade() {
            Some(w) => w,
            None => return glib::Propagation::Proceed,
        };
        let column_view = match column_view_weak.upgrade() {
            Some(w) => w,
            None => return glib::Propagation::Proceed,
        };
        let selection_model = match selection_model_weak.upgrade() {
            Some(w) => w,
            None => return glib::Propagation::Proceed,
        };
        let edit_page_container = match edit_page_container_weak.upgrade() {
            Some(w) => w,
            None => return glib::Propagation::Proceed,
        };
        let toast_overlay = match toast_overlay_weak.upgrade() {
            Some(w) => w,
            None => return glib::Propagation::Proceed,
        };

        if mods.contains(gtk::gdk::ModifierType::CONTROL_MASK) && key == gtk::gdk::Key::f {
            search_entry.grab_focus();
            return glib::Propagation::Stop;
        }

        let home_visible = root_stack.visible_child_name().as_deref() == Some("home");
        let search_focused = search_entry.has_focus();

        if home_visible && search_focused && key == gtk::gdk::Key::Down {
            column_view.grab_focus();
            return glib::Propagation::Stop;
        }

        if home_visible && !search_focused && mods.is_empty() {
            match key {
                gtk::gdk::Key::slash => {
                    search_entry.grab_focus();
                    return glib::Propagation::Stop;
                }
                gtk::gdk::Key::Return => {
                    if let Some(obj) = selection_model
                        .selected_item()
                        .and_downcast::<KeybindObject>()
                    {
                        while let Some(child) = edit_page_container.first_child() {
                            edit_page_container.remove(&child);
                        }
                        let edit_view = create_edit_view(
                            &root_stack,
                            obj,
                            &model_key,
                            &column_view,
                            &selection_model,
                            &toast_overlay,
                            &edit_page_container,
                        );
                        edit_page_container.append(&edit_view);
                        root_stack.set_visible_child_name("edit");
                        return glib::Propagation::Stop;
                    }
                }
                _ => {}
            }
        }

        if key == gtk::gdk::Key::Escape {
            if root_stack.visible_child_name().as_deref() != Some("home") {
                root_stack.set_visible_child_name("home");
                return glib::Propagation::Stop;
            }
            if !search_entry.text().is_empty() {
                search_entry.set_text("");
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
    let root_stack_weak = root_stack.downgrade();
    let edit_page_container_weak = edit_page_container.downgrade();

    column_view.connect_activate(move |view, position| {
        let root_stack = match root_stack_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let edit_page_container = match edit_page_container_weak.upgrade() {
            Some(w) => w,
            None => return,
        };

        let selection = view
            .model()
            .unwrap()
            .downcast::<gtk::SingleSelection>()
            .unwrap();
        if let Some(obj) = selection.item(position).and_downcast::<KeybindObject>() {
            // Clear previous edit form
            while let Some(child) = edit_page_container.first_child() {
                edit_page_container.remove(&child);
            }

            let edit_view = create_edit_view(
                &root_stack,
                obj,
                &model_store,
                view,
                &selection,
                &toast_overlay_activate,
                &edit_page_container,
            );
            edit_page_container.append(&edit_view);
            root_stack.set_visible_child_name("edit");
        }
    });

    let model_clone_add = model.clone();
    let toast_overlay_add = toast_overlay.clone();
    let root_stack_weak = root_stack.downgrade();
    let add_page_container_weak = add_page_container.downgrade();

    add_button.connect_clicked(move |_| {
        let root_stack = match root_stack_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let add_page_container = match add_page_container_weak.upgrade() {
            Some(w) => w,
            None => return,
        };

        // Clear previous add form (optional but good for reset)
        while let Some(child) = add_page_container.first_child() {
            add_page_container.remove(&child);
        }

        let add_view = create_add_view(&root_stack, &model_clone_add, &toast_overlay_add);
        add_page_container.append(&add_view);
        root_stack.set_visible_child_name("add");
    });

    let model_bulk = model.clone();
    let toast_bulk = toast_overlay.clone();
    let stack_weak = root_stack.downgrade();
    let wizard_container_weak = wizard_page_container.downgrade();

    bulk_button.connect_clicked(move |_| {
        let stack = match stack_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let wizard_container = match wizard_container_weak.upgrade() {
            Some(w) => w,
            None => return,
        };

        while let Some(child) = wizard_container.first_child() {
            wizard_container.remove(&child);
        }
        let view = create_bulk_replace_wizard(&stack, &model_bulk, &toast_bulk);
        wizard_container.append(&view);
        stack.set_visible_child_name("wizard");
    });

    let toast_overlay_weak = toast_overlay.downgrade();
    backup_button.connect_clicked(move |_| {
        let toast_overlay = match toast_overlay_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        match crate::ui::utils::perform_backup(true) {
            Ok(msg) => {
                let toast = adw::Toast::builder()
                    .title(&msg)
                    .timeout(crate::config::constants::TOAST_TIMEOUT)
                    .build();
                toast_overlay.add_toast(toast);
            }
            Err(e) => {
                let toast = adw::Toast::builder()
                    .title(format!("Backup failed: {}", e))
                    .timeout(crate::config::constants::TOAST_TIMEOUT)
                    .build();
                toast_overlay.add_toast(toast);
            }
        }
    });

    // Logic to update conflict button visibility
    let update_conflict_btn = {
        let conflict_button_weak = conflict_button.downgrade();
        move |model: &gio::ListStore| {
            let conflict_button = match conflict_button_weak.upgrade() {
                Some(btn) => btn,
                None => return,
            };
            let mut conflict_count = 0;
            for obj in model.snapshot() {
                if let Some(obj) = obj.downcast_ref::<KeybindObject>() {
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

    // HACK: ListStore doesn't expose "on content changed" easily for deep property changes unless we bind to them.
    // However, we reload the whole model on add/edit/delete, triggering `items-changed`.
    // We can hook into that.
    model.connect_items_changed(move |m, _, _, _| {
        update_conflict_btn_c(m);
    });

    let model_wizard = model.clone();
    let stack_weak = root_stack.downgrade();
    let toast_wizard = toast_overlay.clone();
    let wizard_container_weak = wizard_page_container.downgrade();
    let selection_model_weak = selection_model.downgrade();

    let column_view_weak = column_view.downgrade();
    conflict_button.connect_clicked(move |_| {
        let stack = match stack_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let wizard_container = match wizard_container_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let column_view = match column_view_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let selection_model = match selection_model_weak.upgrade() {
            Some(w) => w,
            None => return,
        };

        while let Some(child) = wizard_container.first_child() {
            wizard_container.remove(&child);
        }

        let wizard_view = create_conflict_wizard(
            &stack,
            &model_wizard,
            &column_view,
            &selection_model,
            &toast_wizard,
            &wizard_container,
            0,
        );
        wizard_container.append(&wizard_view);
        stack.set_visible_child_name("wizard");
    });

    let status_page_weak = status_page.downgrade();
    let list_stack_weak = list_stack.downgrade();
    let scrolled_weak = scrolled_window.downgrade();

    // Use a signal handler id to allow disconnection if needed
    let _filter_items_changed_id = filter_model.connect_items_changed(move |m, _, _, _| {
        let status_page = match status_page_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let list_stack = match list_stack_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let scrolled = match scrolled_weak.upgrade() {
            Some(w) => w,
            None => return,
        };

        let has_items = m.n_items() > 0;
        status_page.set_visible(!has_items);
        scrolled.set_visible(has_items);
        if has_items {
            list_stack.set_visible_child(&scrolled);
        } else {
            list_stack.set_visible_child(&status_page);
        }
    });

    let matcher = Arc::new(SkimMatcherV2::default());

    let filter_func = move |text: String, category: u32| {
        let query = SearchQuery::parse(&text);
        let m = Arc::clone(&matcher);

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

    let stack_weak = root_stack.downgrade();
    let container_weak = settings_page_container.downgrade();
    let window_weak = window.downgrade();
    let col_desc_weak = col_desc.downgrade();
    let col_fav_weak = col_fav.downgrade();
    let col_args_weak = col_args.downgrade();
    let col_submap_weak = col_submap.downgrade();
    let col_key_weak = col_key.downgrade();
    let col_mods_weak = col_mods.downgrade();
    let col_disp_weak = col_disp.downgrade();
    let column_view_weak = column_view.downgrade();
    let model_settings = model.clone();
    let toast_overlay_weak = toast_overlay.downgrade();
    let restore_container_weak = restore_page_container.downgrade();
    let dropdown_weak = category_dropdown.downgrade();

    settings_button.connect_clicked(move |_| {
        let stack = match stack_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let container = match container_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let window = match window_weak.upgrade() {
            Some(w) => w,
            None => return,
        };

        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        let col_desc_w = col_desc_weak.clone();
        let col_fav_w = col_fav_weak.clone();
        let col_args_w = col_args_weak.clone();
        let col_submap_w = col_submap_weak.clone();

        let col_key_w = col_key_weak.clone();
        let col_mods_w = col_mods_weak.clone();
        let col_disp_w = col_disp_weak.clone();
        let col_args_sort_w = col_args_weak.clone();
        let col_submap_sort_w = col_submap_weak.clone();
        let col_view_w = column_view_weak.clone();

        let model_s = model_settings.clone();
        let model_s_restore = model_settings.clone();
        let toast_w = toast_overlay_weak.clone();
        let stack_w = stack_weak.clone();
        let restore_container_w = restore_container_weak.clone();
        let dropdown_w = dropdown_weak.clone();

        let toast_w_1 = toast_w.clone();
        let toast_w_2 = toast_w.clone();

        let view = crate::ui::settings::create_settings_view(
            &window,
            &stack,
            &model_s,
            std::rc::Rc::new(move |s| {
                if let Some(c) = col_desc_w.upgrade() {
                    c.set_visible(s)
                }
            }),
            std::rc::Rc::new(move |s| {
                if let Some(c) = col_fav_w.upgrade() {
                    c.set_visible(s);
                }
                // Update dropdown options
                if let Some(dropdown) = dropdown_w.upgrade() {
                    let mut cat_list =
                        vec!["All", "Workspace", "Window", "Media", "Custom", "Mouse"];
                    if s {
                        cat_list.push("Favorites");
                    }
                    // Preserve selection if possible, otherwise default to All
                    let selected = dropdown.selected();
                    let model = gtk::StringList::new(&cat_list);
                    dropdown.set_model(Some(&model));
                    if selected < model.n_items() {
                        dropdown.set_selected(selected);
                    } else {
                        dropdown.set_selected(0);
                    }
                }
            }),
            std::rc::Rc::new(move |s| {
                if let Some(c) = col_args_w.upgrade() {
                    c.set_visible(s)
                }
            }),
            std::rc::Rc::new(move |s| {
                if let Some(c) = col_submap_w.upgrade() {
                    c.set_visible(s)
                }
            }),
            std::rc::Rc::new(move |sort_key| {
                // Resolve weak refs
                let col_mods = match col_mods_w.upgrade() {
                    Some(c) => c,
                    None => return,
                };
                let col_disp = match col_disp_w.upgrade() {
                    Some(c) => c,
                    None => return,
                };
                let col_args = match col_args_sort_w.upgrade() {
                    Some(c) => c,
                    None => return,
                };
                let col_submap = match col_submap_sort_w.upgrade() {
                    Some(c) => c,
                    None => return,
                };
                let col_key = match col_key_w.upgrade() {
                    Some(c) => c,
                    None => return,
                };
                let col_view = match col_view_w.upgrade() {
                    Some(c) => c,
                    None => return,
                };

                let col = match sort_key.as_str() {
                    "mods" => Some(&col_mods),
                    "dispatcher" => Some(&col_disp),
                    "args" => Some(&col_args),
                    "submap" => Some(&col_submap),
                    _ => Some(&col_key), // Default key
                };
                if let Some(c) = col {
                    col_view.sort_by_column(Some(c), gtk::SortType::Ascending);
                }
            }),
            std::rc::Rc::new(move |msg| {
                if let Some(toast_overlay) = toast_w_1.upgrade() {
                    let toast = adw::Toast::builder()
                        .title(&msg)
                        .timeout(crate::config::constants::TOAST_TIMEOUT)
                        .build();
                    toast_overlay.add_toast(toast);
                }
            }),
            std::rc::Rc::new(move || {
                let stack = match stack_w.upgrade() {
                    Some(s) => s,
                    None => return,
                };
                let restore_container = match restore_container_w.upgrade() {
                    Some(c) => c,
                    None => return,
                };
                let toast_overlay = match toast_w_2.upgrade() {
                    Some(t) => t,
                    None => return,
                };

                while let Some(child) = restore_container.first_child() {
                    restore_container.remove(&child);
                }
                let restore_view = crate::ui::views::create_restore_view(
                    &stack,
                    &model_s_restore,
                    &toast_overlay,
                    &restore_container,
                );
                restore_container.append(&restore_view);
                stack.set_visible_child_name("restore");
            }),
        );
        container.append(&view);
        stack.set_visible_child_name("settings");
    });

    let stack_weak = root_stack.downgrade();
    let container_weak = keyboard_page_container.downgrade();
    let model_keyboard = model.clone();
    keyboard_button.connect_clicked(move |_| {
        let stack = match stack_weak.upgrade() {
            Some(w) => w,
            None => return,
        };
        let container = match container_weak.upgrade() {
            Some(w) => w,
            None => return,
        };

        while let Some(child) = container.first_child() {
            container.remove(&child);
        }
        let view = crate::ui::views::create_keyboard_view(&stack, &model_keyboard);
        container.append(&view);
        stack.set_visible_child_name("keyboard");
    });

    window.present();
    search_entry.grab_focus();
}
