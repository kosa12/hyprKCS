use crate::config::StyleConfig;
use gtk::glib;
use gtk4 as gtk;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_ui_elements_page(
    config: Rc<RefCell<StyleConfig>>,
    on_desc_toggle: Rc<dyn Fn(bool)>,
    on_fav_toggle: Rc<dyn Fn(bool)>,
    on_args_toggle: Rc<dyn Fn(bool)>,
    on_submap_toggle: Rc<dyn Fn(bool)>,
    on_sort_change: Rc<dyn Fn(String)>,
) -> adw::PreferencesPage {
    let page_ui = adw::PreferencesPage::builder().build();
    let group_cols = adw::PreferencesGroup::builder()
        .title("Table Columns")
        .build();

    // Submaps
    let sub_switch = gtk::Switch::builder()
        .active(config.borrow().show_submaps)
        .valign(gtk::Align::Center)
        .build();
    let sub_row = adw::ActionRow::builder()
        .title("Show Submaps")
        .activatable_widget(&sub_switch)
        .build();
    sub_row.add_suffix(&sub_switch);
    let c = config.clone();
    let on_sub = on_submap_toggle.clone();
    sub_switch.connect_state_set(move |_, s| {
        c.borrow_mut().show_submaps = s;
        let _ = c.borrow().save();
        on_sub(s);
        glib::Propagation::Proceed
    });
    group_cols.add(&sub_row);

    // Args
    let args_switch = gtk::Switch::builder()
        .active(config.borrow().show_args)
        .valign(gtk::Align::Center)
        .build();
    let args_row = adw::ActionRow::builder()
        .title("Show Arguments")
        .activatable_widget(&args_switch)
        .build();
    args_row.add_suffix(&args_switch);
    let c = config.clone();
    let on_args = on_args_toggle.clone();
    args_switch.connect_state_set(move |_, s| {
        c.borrow_mut().show_args = s;
        let _ = c.borrow().save();
        on_args(s);
        glib::Propagation::Proceed
    });
    group_cols.add(&args_row);

    // Favorites
    let fav_switch = gtk::Switch::builder()
        .active(config.borrow().show_favorites)
        .valign(gtk::Align::Center)
        .build();
    let fav_row = adw::ActionRow::builder()
        .title("Show Favorites")
        .activatable_widget(&fav_switch)
        .build();
    fav_row.add_suffix(&fav_switch);
    let c = config.clone();
    let on_fav = on_fav_toggle.clone();
    fav_switch.connect_state_set(move |_, s| {
        c.borrow_mut().show_favorites = s;
        let _ = c.borrow().save();
        on_fav(s);
        glib::Propagation::Proceed
    });
    group_cols.add(&fav_row);

    // Description
    let desc_switch = gtk::Switch::builder()
        .active(config.borrow().show_description)
        .valign(gtk::Align::Center)
        .build();
    let desc_row = adw::ActionRow::builder()
        .title("Show Description")
        .activatable_widget(&desc_switch)
        .build();
    desc_row.add_suffix(&desc_switch);
    let c = config.clone();
    let on_toggle = on_desc_toggle.clone();
    desc_switch.connect_state_set(move |_, s| {
        c.borrow_mut().show_description = s;
        let _ = c.borrow().save();
        on_toggle(s);
        glib::Propagation::Proceed
    });
    group_cols.add(&desc_row);

    let group_sort = adw::PreferencesGroup::builder().title("Sorting").build();

    // Default Sort
    let sort_opts = ["Key", "Modifiers", "Action", "Arguments", "Submap"];
    let sort_list = gtk::StringList::new(&sort_opts);

    // Map current string to index
    let current_sort = config.borrow().default_sort.to_lowercase();
    let selected_idx = if current_sort.contains("mod") {
        1
    } else if current_sort.contains("disp") || current_sort.contains("action") {
        2
    } else if current_sort.contains("arg") {
        3
    } else if current_sort.contains("sub") {
        4
    } else {
        0
    }; // Default Key

    let sort_drop = gtk::DropDown::builder()
        .model(&sort_list)
        .selected(selected_idx)
        .valign(gtk::Align::Center)
        .build();
    let sort_row = adw::ActionRow::builder()
        .title("Default Sort Column")
        .build();
    sort_row.add_suffix(&sort_drop);

    let c = config.clone();
    let on_sort = on_sort_change.clone();
    sort_drop.connect_selected_notify(move |d| {
        let val = match d.selected() {
            1 => "mods",
            2 => "dispatcher",
            3 => "args",
            4 => "submap",
            _ => "key",
        };
        c.borrow_mut().default_sort = val.to_string();
        let _ = c.borrow().save();
        on_sort(val.to_string());
    });
    group_sort.add(&sort_row);

    page_ui.add(&group_cols);
    page_ui.add(&group_sort);

    page_ui
}
