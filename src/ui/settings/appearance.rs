use crate::config::StyleConfig;
use gtk::glib;
use gtk4 as gtk;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_appearance_page(
    config: Rc<RefCell<StyleConfig>>,
    on_show_toast: Rc<dyn Fn(String)>,
) -> adw::PreferencesPage {
    let page_app = adw::PreferencesPage::builder().build();
    let group_font = adw::PreferencesGroup::builder()
        .title("Typography and Borders")
        .build();

    // Theme Selector
    let theme_opts = ["Adwaita", "Omarchy"];
    let theme_list = gtk::StringList::new(&theme_opts);
    let current_theme = config.borrow().theme.clone();
    let theme_idx = if current_theme == "Omarchy" { 1 } else { 0 };

    let theme_drop = gtk::DropDown::builder()
        .model(&theme_list)
        .selected(theme_idx)
        .valign(gtk::Align::Center)
        .build();
    let theme_row = adw::ActionRow::builder()
        .title("Theme")
        .subtitle("Application styling")
        .build();
    theme_row.add_suffix(&theme_drop);

    let c = config.clone();
    let toast = on_show_toast.clone();
    theme_drop.connect_selected_notify(move |d| {
        let is_omarchy = d.selected() == 1;
        let new_theme = if is_omarchy { "Omarchy" } else { "Adwaita" };

        if is_omarchy {
            let mut found = false;
            if let Some(config_dir) = dirs::config_dir() {
                let paths = [
                    config_dir.join("omarchy/colors.toml"),
                    config_dir.join("hypr/colors.toml"),
                ];
                for path in &paths {
                    if path.exists() {
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                toast(
                    "Omarchy colors.toml not found in ~/.config/omarchy/ or ~/.config/hypr/"
                        .to_string(),
                );
                d.set_selected(0);
                return;
            }
        }

        c.borrow_mut().theme = new_theme.to_string();
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_font.add(&theme_row);

    // Font Size
    let font_entry = gtk::Entry::builder()
        .text(config.borrow().font_size.as_deref().unwrap_or("0.9rem"))
        .valign(gtk::Align::Center)
        .width_chars(10)
        .build();
    let font_row = adw::ActionRow::builder()
        .title("Font Size")
        .subtitle("CSS value (e.g. 12px, 1rem)")
        .build();
    font_row.add_suffix(&font_entry);
    let c = config.clone();
    font_entry.connect_changed(move |e| {
        c.borrow_mut().font_size = Some(e.text().to_string());
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_font.add(&font_row);

    // Border Size
    let b_size_entry = gtk::Entry::builder()
        .text(config.borrow().border_size.as_deref().unwrap_or("1px"))
        .valign(gtk::Align::Center)
        .width_chars(10)
        .build();
    let b_size_row = adw::ActionRow::builder()
        .title("Border Size")
        .subtitle("CSS value (e.g. 2px)")
        .build();
    b_size_row.add_suffix(&b_size_entry);
    let c = config.clone();
    b_size_entry.connect_changed(move |e| {
        c.borrow_mut().border_size = Some(e.text().to_string());
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_font.add(&b_size_row);

    // Border Radius
    let b_rad_entry = gtk::Entry::builder()
        .text(config.borrow().border_radius.as_deref().unwrap_or("12px"))
        .valign(gtk::Align::Center)
        .width_chars(10)
        .build();
    let b_rad_row = adw::ActionRow::builder()
        .title("Border Radius")
        .subtitle("CSS value (e.g. 10px)")
        .build();
    b_rad_row.add_suffix(&b_rad_entry);
    let c = config.clone();
    b_rad_entry.connect_changed(move |e| {
        c.borrow_mut().border_radius = Some(e.text().to_string());
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_font.add(&b_rad_row);

    // Keyboard Layout
    let layout_opts = ["ANSI", "ISO", "JIS", "ABNT2", "Hungarian", "Ortholinear"];
    let layout_list = gtk::StringList::new(&layout_opts);

    // Map current string to index
    let current_layout = config.borrow().keyboard_layout.to_uppercase();
    let layout_idx = match current_layout.as_str() {
        "ISO" => 1,
        "JIS" => 2,
        "ABNT2" => 3,
        "HU" | "HUNGARIAN" => 4,
        "ORTHO" | "ORTHOLINEAR" => 5,
        _ => 0,
    };

    let layout_drop = gtk::DropDown::builder()
        .model(&layout_list)
        .selected(layout_idx)
        .valign(gtk::Align::Center)
        .build();
    let layout_row = adw::ActionRow::builder()
        .title("Keyboard Layout")
        .subtitle("Visual keyboard map type")
        .build();
    layout_row.add_suffix(&layout_drop);

    let c = config.clone();
    layout_drop.connect_selected_notify(move |d| {
        let val = match d.selected() {
            1 => "ISO",
            2 => "JIS",
            3 => "ABNT2",
            4 => "HUNGARIAN",
            5 => "ORTHOLINEAR",
            _ => "ANSI",
        };
        c.borrow_mut().keyboard_layout = val.to_string();
        let _ = c.borrow().save();
    });
    group_font.add(&layout_row);

    // Alternating Colors
    let alt_switch = gtk::Switch::builder()
        .active(config.borrow().alternating_row_colors)
        .valign(gtk::Align::Center)
        .build();
    let alt_row = adw::ActionRow::builder()
        .title("Alternating Row Colors")
        .activatable_widget(&alt_switch)
        .build();
    alt_row.add_suffix(&alt_switch);
    let c = config.clone();
    alt_switch.connect_state_set(move |_, s| {
        c.borrow_mut().alternating_row_colors = s;
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
        glib::Propagation::Proceed
    });
    group_font.add(&alt_row);

    page_app.add(&group_font);

    page_app
}
