use crate::config::StyleConfig;
use crate::keybind_object::KeybindObject;
use crate::ui::utils::collect_submaps;
use crate::ui::wizards::create_add_submap_wizard;
use gtk::gio;
use gtk4 as gtk;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_submaps_page(
    model: &gio::ListStore,
    config: Rc<RefCell<StyleConfig>>,
    stack: &gtk::Stack,
    toast_overlay: &adw::ToastOverlay,
    on_focus_submap: Rc<dyn Fn(Option<String>)>,
) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder()
        .title("Submaps")
        .icon_name("view-list-bullet-symbolic")
        .build();

    let group = adw::PreferencesGroup::builder()
        .title("Manage Submaps")
        .description("View and manage your Hyprland modes (submaps).")
        .build();

    let add_row = adw::ActionRow::builder()
        .title("Add New Submap")
        .subtitle("Create a new Hyprland mode")
        .activatable(true)
        .build();

    let add_icon = gtk::Image::from_icon_name("list-add-symbolic");
    add_row.add_prefix(&add_icon);

    let stack_c = stack.clone();
    let model_c = model.clone();
    let toast_c = toast_overlay.clone();
    let config_c_add = config.clone();

    add_row.connect_activated(move |row| {
        let root = row.root();
        if root.is_some() {
            let default_submap = config_c_add.borrow().default_submap.clone();

            // We use the root stack to switch to the wizard page
            // Find the wizard container in the root stack
            if let Some(wizard_container) =
                stack_c.child_by_name("wizard").and_downcast::<gtk::Box>()
            {
                while let Some(child) = wizard_container.first_child() {
                    wizard_container.remove(&child);
                }
                let wizard_view =
                    create_add_submap_wizard(&stack_c, &model_c, &toast_c, default_submap);
                wizard_container.append(&wizard_view);
                stack_c.set_visible_child_name("wizard");
            }
        }
    });
    group.add(&add_row);

    // --- Default Submap Row ---
    // Extract unique submaps
    let mut submaps_set = std::collections::HashSet::new();
    for obj in model.snapshot() {
        if let Some(obj) = obj.downcast_ref::<KeybindObject>() {
            if let Some(s) = obj.with_data(|d| d.submap.as_ref().map(|r| r.to_string())) {
                submaps_set.insert(s);
            }
        }
    }
    let mut sorted_submaps: Vec<String> = submaps_set.into_iter().collect();
    sorted_submaps.sort();

    let mut display_items = vec!["All Submaps".to_string()];
    display_items.extend(sorted_submaps.clone());

    let submap_model = gtk::StringList::new(
        &display_items
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>(),
    );

    let default_submap_row = adw::ComboRow::builder()
        .title("Default Submap")
        .subtitle("Select the submap to show on startup")
        .model(&submap_model)
        .build();

    // Set initial selection
    let current_default = config.borrow().default_submap.clone();
    if let Some(def) = current_default {
        for (i, item) in display_items.iter().enumerate() {
            if *item == def {
                default_submap_row.set_selected(i as u32);
                break;
            }
        }
    } else {
        default_submap_row.set_selected(0);
    }

    let c_submap = config.clone();
    default_submap_row.connect_selected_notify(move |row| {
        let idx = row.selected();
        let selected_val = if let Some(m) = row.model() {
            if let Some(s) = m.item(idx).and_downcast::<gtk::StringObject>() {
                s.string().to_string()
            } else {
                "All Submaps".to_string()
            }
        } else {
            "All Submaps".to_string()
        };

        let new_val = if selected_val == "All Submaps" {
            None
        } else {
            Some(selected_val)
        };

        c_submap.borrow_mut().default_submap = new_val;
        let _ = c_submap.borrow().save();
    });

    group.add(&default_submap_row);

    let submaps = collect_submaps(model);

    if submaps.is_empty() {
        let row = adw::ActionRow::builder()
            .title("No Submaps Detected")
            .subtitle("Submaps are defined using 'submap = name' in your config.")
            .build();
        group.add(&row);
    } else {
        for name in submaps {
            // Count binds in this submap
            let mut count = 0;
            for obj in model.snapshot() {
                if let Some(obj) = obj.downcast_ref::<KeybindObject>() {
                    if obj.with_data(|d| {
                        d.submap.as_ref().map(|s| s.to_string()) == Some(name.clone())
                    }) {
                        count += 1;
                    }
                }
            }

            let row = adw::ActionRow::builder()
                .title(&name)
                .subtitle(format!("{} keybinds", count))
                .activatable(true) // Make clickable
                .build();

            let icon = gtk::Image::from_icon_name("go-next-symbolic");
            row.add_suffix(&icon);

            let on_focus = on_focus_submap.clone();
            let name_clone = name.clone();
            row.connect_activated(move |_| {
                on_focus(Some(name_clone.clone()));
            });

            group.add(&row);
        }
    }

    page.add(&group);
    page
}
