use crate::config::StyleConfig;

use gtk4 as gtk;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_window_page(
    config: Rc<RefCell<StyleConfig>>,
    window: &adw::ApplicationWindow,
) -> adw::PreferencesPage {
    let page_window = adw::PreferencesPage::builder().build();
    let group_dims = adw::PreferencesGroup::builder().title("Dimensions").build();

    // Width
    let width_adj = gtk::Adjustment::new(
        config.borrow().width as f64,
        100.0,
        3840.0,
        10.0,
        100.0,
        0.0,
    );
    let width_spin = gtk::SpinButton::builder()
        .adjustment(&width_adj)
        .valign(gtk::Align::Center)
        .build();
    let width_row = adw::ActionRow::builder()
        .title("Window Width")
        .subtitle("px")
        .build();
    width_row.add_suffix(&width_spin);
    let c = config.clone();
    let window_c = window.clone();
    width_spin.connect_value_changed(move |s| {
        let val = s.value() as i32;
        {
            c.borrow_mut().width = val;
            let _ = c.borrow().save();
        }
        let h = c.borrow().height;
        window_c.set_default_size(val, h);
        window_c.set_size_request(val, h);
        crate::ui::style::reload_style();
    });
    group_dims.add(&width_row);

    // Height
    let height_adj = gtk::Adjustment::new(
        config.borrow().height as f64,
        100.0,
        2160.0,
        10.0,
        100.0,
        0.0,
    );
    let height_spin = gtk::SpinButton::builder()
        .adjustment(&height_adj)
        .valign(gtk::Align::Center)
        .build();
    let height_row = adw::ActionRow::builder()
        .title("Window Height")
        .subtitle("px")
        .build();
    height_row.add_suffix(&height_spin);
    let c = config.clone();
    let window_c = window.clone();
    height_spin.connect_value_changed(move |s| {
        let val = s.value() as i32;
        {
            c.borrow_mut().height = val;
            let _ = c.borrow().save();
        }
        let w = c.borrow().width;
        window_c.set_default_size(w, val);
        window_c.set_size_request(w, val);
        crate::ui::style::reload_style();
    });
    group_dims.add(&height_row);

    // Monitor Margin
    let margin_adj = gtk::Adjustment::new(
        config.borrow().monitor_margin as f64,
        0.0,
        500.0,
        1.0,
        10.0,
        0.0,
    );
    let margin_spin = gtk::SpinButton::builder()
        .adjustment(&margin_adj)
        .valign(gtk::Align::Center)
        .build();
    let margin_row = adw::ActionRow::builder()
        .title("Monitor Margin")
        .subtitle("Spacing from screen edges (px)")
        .build();
    margin_row.add_suffix(&margin_spin);
    let c = config.clone();
    margin_spin.connect_value_changed(move |s| {
        c.borrow_mut().monitor_margin = s.value() as i32;
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_dims.add(&margin_row);

    // Row Padding
    let pad_adj =
        gtk::Adjustment::new(config.borrow().row_padding as f64, 0.0, 50.0, 1.0, 5.0, 0.0);
    let pad_spin = gtk::SpinButton::builder()
        .adjustment(&pad_adj)
        .valign(gtk::Align::Center)
        .build();
    let pad_row = adw::ActionRow::builder()
        .title("Row Padding")
        .subtitle("Spacing between list rows (px)")
        .build();
    pad_row.add_suffix(&pad_spin);
    let c = config.clone();
    pad_spin.connect_value_changed(move |s| {
        c.borrow_mut().row_padding = s.value() as i32;
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_dims.add(&pad_row);

    let group_style = adw::PreferencesGroup::builder()
        .title("Window Style")
        .build();

    // Opacity
    let op_adj = gtk::Adjustment::new(
        config.borrow().opacity.unwrap_or(1.0),
        0.1,
        1.0,
        0.05,
        0.1,
        0.0,
    );
    let op_spin = gtk::SpinButton::builder()
        .adjustment(&op_adj)
        .digits(2)
        .valign(gtk::Align::Center)
        .build();
    let op_row = adw::ActionRow::builder()
        .title("Opacity")
        .subtitle("0.0 - 1.0")
        .build();
    op_row.add_suffix(&op_spin);
    let c = config.clone();
    op_spin.connect_value_changed(move |s| {
        c.borrow_mut().opacity = Some(s.value());
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_style.add(&op_row);

    // Shadow Size
    let shadow_entry = gtk::Entry::builder()
        .text(&config.borrow().shadow_size)
        .valign(gtk::Align::Center)
        .width_chars(25)
        .build();
    let shadow_row = adw::ActionRow::builder()
        .title("Shadow Size")
        .subtitle("CSS box-shadow format")
        .build();
    shadow_row.add_suffix(&shadow_entry);
    let c = config.clone();
    shadow_entry.connect_changed(move |e| {
        c.borrow_mut().shadow_size = e.text().to_string();
        let _ = c.borrow().save();
        crate::ui::style::reload_style();
    });
    group_style.add(&shadow_row);

    page_window.add(&group_dims);
    page_window.add(&group_style);

    page_window
}
