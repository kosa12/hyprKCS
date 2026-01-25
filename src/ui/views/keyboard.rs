use crate::config::StyleConfig;
use crate::keybind_object::KeybindObject;
use crate::ui::utils::normalize;
use crate::ui::views::keyboard_layouts::{get_layout_rows, KeyDef, ROW_ARROWS, ROW_FUNC};
use gtk::gio;
use gtk::prelude::*;
use gtk4 as gtk;
use std::collections::{HashMap, HashSet};

pub fn create_keyboard_view(stack: &gtk::Stack, model: &gio::ListStore) -> gtk::Box {
    let config = StyleConfig::load();
    let layout = config.keyboard_layout.to_uppercase();

    // Select Rows based on layout
    let (row1, row2, row3, row4, row5) = get_layout_rows(&layout);

    let container = gtk::Box::new(gtk::Orientation::Vertical, 8);
    container.set_margin_top(8);
    container.set_margin_bottom(8);
    container.set_margin_start(12);
    container.set_margin_end(12);
    container.set_halign(gtk::Align::Fill);
    container.set_valign(gtk::Align::Fill);

    // Title / Back Button
    let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    header_box.set_halign(gtk::Align::Center);
    let back_btn = gtk::Button::builder()
        .icon_name("go-previous-symbolic")
        .css_classes(["flat", "circular"])
        .tooltip_text("Back to List")
        .build();

    let stack_weak = stack.downgrade();
    back_btn.connect_clicked(move |_| {
        if let Some(s) = stack_weak.upgrade() {
            s.set_visible_child_name("home");
        }
    });

    let title = gtk::Label::builder()
        .label(&format!("Visual Keyboard Map ({})", layout))
        .css_classes(["title-2"])
        .build();

    header_box.append(&back_btn);
    header_box.append(&title);
    container.append(&header_box);

    // Modifier Toggles
    let mod_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    mod_box.set_halign(gtk::Align::Center);
    mod_box.set_margin_bottom(4);

    mod_box.append(&gtk::Label::new(Some("Modifiers:")));

    let btn_super = gtk::ToggleButton::builder()
        .label("SUPER")
        .css_classes(["small", "mod-toggle"])
        .build();
    let btn_shift = gtk::ToggleButton::builder()
        .label("SHIFT")
        .css_classes(["small", "mod-toggle"])
        .build();
    let btn_ctrl = gtk::ToggleButton::builder()
        .label("CTRL")
        .css_classes(["small", "mod-toggle"])
        .build();
    let btn_alt = gtk::ToggleButton::builder()
        .label("ALT")
        .css_classes(["small", "mod-toggle"])
        .build();

    btn_super.set_active(true);

    mod_box.append(&btn_super);
    mod_box.append(&btn_shift);
    mod_box.append(&btn_ctrl);
    mod_box.append(&btn_alt);

    container.append(&mod_box);

    // Keyboard Grid
    let grid = gtk::Grid::builder()
        .column_homogeneous(true)
        .row_homogeneous(true)
        .column_spacing(2)
        .row_spacing(2)
        .hexpand(true)
        .vexpand(true)
        .halign(gtk::Align::Fill)
        .valign(gtk::Align::Fill)
        .build();

    grid.add_css_class("keyboard-container");

    let mut row_idx = 0;

    let add_row = |keys: &[KeyDef], r_idx: i32, g: &gtk::Grid| {
        let mut col_idx = 0;
        for k in keys {
            let width_cells = (k.width * 4.0).round() as i32;
            let btn = gtk::Button::builder()
                .label(k.label)
                .css_classes(["keyboard-key"])
                .hexpand(true)
                .vexpand(true)
                .build();

            // Store normalized key name
            let (_, norm_key) = normalize("", k.hypr_name);
            btn.set_widget_name(&norm_key);

            g.attach(&btn, col_idx, r_idx, width_cells, 1);
            col_idx += width_cells;
        }
    };

    add_row(ROW_FUNC, row_idx, &grid);
    row_idx += 1;
    add_row(row1, row_idx, &grid);
    row_idx += 1;
    add_row(row2, row_idx, &grid);
    row_idx += 1;
    add_row(row3, row_idx, &grid);
    row_idx += 1;
    add_row(row4, row_idx, &grid);
    row_idx += 1;
    add_row(row5, row_idx, &grid);
    row_idx += 1;

    // Arrow keys (Skip for Ortholinear as they are integrated)
    if !layout.contains("ORTHO") {
        // Total columns approx 60.
        // Arrows are 4 keys = 4 width each = 16 cols.
        // Centered start = (60 - 16) / 2 = 22.
        let arrow_start_col = 22;
        let mut arrow_col = arrow_start_col;
        for k in ROW_ARROWS {
            let width_cells = 4; // 1.0 * 4
            let btn = gtk::Button::builder()
                .label(k.label)
                .css_classes(["keyboard-key"])
                .hexpand(true)
                .vexpand(true)
                .build();
            let (_, norm_key) = normalize("", k.hypr_name);
            btn.set_widget_name(&norm_key);

            grid.attach(&btn, arrow_col, row_idx, width_cells, 1);
            arrow_col += width_cells;
        }
    }

    container.append(&grid);

    // Details Label
    let details_label = gtk::Label::builder()
        .label("Hover over a highlighted key to see the action")
        .css_classes(["dim-label"])
        .margin_top(12)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();
    container.append(&details_label);

    // Logic
    let update_keys = {
        let model = model.clone();
        let grid_ref = grid.clone();

        move |active_mods: &[String]| {
            let mut bound_keys: HashSet<String> = HashSet::new();
            let mut key_actions: HashMap<String, String> = HashMap::new();

            let (target_mods, _) = normalize(&active_mods.join(" "), "");

            for i in 0..model.n_items() {
                if let Some(obj) = model.item(i).and_downcast::<KeybindObject>() {
                    let mods_str = obj.property::<String>("clean-mods");
                    let key_str = obj.property::<String>("key");
                    let disp = obj.property::<String>("dispatcher");
                    let args = obj.property::<String>("args");
                    let submap = obj.property::<String>("submap");

                    let is_global = submap.is_empty()
                        || submap.eq_ignore_ascii_case("global")
                        || submap.eq_ignore_ascii_case("reset");

                    if !is_global {
                        continue;
                    }

                    let (kb_mods, kb_key) = normalize(&mods_str, &key_str);

                    if kb_mods == target_mods {
                        bound_keys.insert(kb_key.clone());
                        let action = if args.is_empty() {
                            disp
                        } else {
                            format!("{} ({})", disp, args)
                        };
                        key_actions.insert(kb_key, action);
                    }
                }
            }

            // Iterate buttons in grid
            let mut child = grid_ref.first_child();
            while let Some(widget) = child {
                if let Some(btn) = widget.downcast_ref::<gtk::Button>() {
                    let key_name = btn.widget_name().to_string();

                    if bound_keys.contains(&key_name) {
                        btn.add_css_class("accent");
                        if let Some(action) = key_actions.get(&key_name) {
                            btn.set_tooltip_text(Some(action));
                        }
                    } else {
                        btn.remove_css_class("accent");
                        btn.set_tooltip_text(None);
                    }
                }
                child = widget.next_sibling();
            }
        }
    };

    let update_rc = std::rc::Rc::new(update_keys);

    let on_toggle = {
        let btn_super = btn_super.clone();
        let btn_shift = btn_shift.clone();
        let btn_ctrl = btn_ctrl.clone();
        let btn_alt = btn_alt.clone();
        let update_fn = update_rc.clone();

        move || {
            let mut active = Vec::new();
            if btn_super.is_active() {
                active.push("SUPER".to_string());
            }
            if btn_shift.is_active() {
                active.push("SHIFT".to_string());
            }
            if btn_ctrl.is_active() {
                active.push("CTRL".to_string());
            }
            if btn_alt.is_active() {
                active.push("ALT".to_string());
            }
            update_fn(&active);
        }
    };

    let on_toggle = std::rc::Rc::new(on_toggle);

    let ot1 = on_toggle.clone();
    btn_super.connect_toggled(move |_| ot1());

    let ot2 = on_toggle.clone();
    btn_shift.connect_toggled(move |_| ot2());

    let ot3 = on_toggle.clone();
    btn_ctrl.connect_toggled(move |_| ot3());

    let ot4 = on_toggle.clone();
    btn_alt.connect_toggled(move |_| ot4());

    // Initial Trigger
    on_toggle();

    container
}
