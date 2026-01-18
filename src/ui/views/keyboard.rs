use crate::keybind_object::KeybindObject;
use crate::ui::utils::normalize;
use gtk::gio;
use gtk::prelude::*;
use gtk4 as gtk;
use std::collections::{HashMap, HashSet};

struct KeyDef {
    label: &'static str,
    hypr_name: &'static str,
    width: f64, // 1.0 is standard key
}

impl KeyDef {
    const fn new(label: &'static str, hypr_name: &'static str, width: f64) -> Self {
        Self {
            label,
            hypr_name,
            width,
        }
    }
}

const ROW_FUNC: &[KeyDef] = &[
    KeyDef::new("Esc", "Escape", 1.0),
    KeyDef::new("F1", "F1", 1.0),
    KeyDef::new("F2", "F2", 1.0),
    KeyDef::new("F3", "F3", 1.0),
    KeyDef::new("F4", "F4", 1.0),
    KeyDef::new("F5", "F5", 1.0),
    KeyDef::new("F6", "F6", 1.0),
    KeyDef::new("F7", "F7", 1.0),
    KeyDef::new("F8", "F8", 1.0),
    KeyDef::new("F9", "F9", 1.0),
    KeyDef::new("F10", "F10", 1.0),
    KeyDef::new("F11", "F11", 1.0),
    KeyDef::new("F12", "F12", 1.0),
    KeyDef::new("PrtSc", "Print", 1.0),
    KeyDef::new("Del", "Delete", 1.0),
];

// Standard ANSI Layout
const ROW_1: &[KeyDef] = &[
    KeyDef::new("`", "grave", 1.0),
    KeyDef::new("1", "1", 1.0),
    KeyDef::new("2", "2", 1.0),
    KeyDef::new("3", "3", 1.0),
    KeyDef::new("4", "4", 1.0),
    KeyDef::new("5", "5", 1.0),
    KeyDef::new("6", "6", 1.0),
    KeyDef::new("7", "7", 1.0),
    KeyDef::new("8", "8", 1.0),
    KeyDef::new("9", "9", 1.0),
    KeyDef::new("0", "0", 1.0),
    KeyDef::new("-", "minus", 1.0),
    KeyDef::new("=", "equal", 1.0),
    KeyDef::new("Bksp", "BackSpace", 2.0),
];

const ROW_2: &[KeyDef] = &[
    KeyDef::new("Tab", "Tab", 1.5),
    KeyDef::new("Q", "Q", 1.0),
    KeyDef::new("W", "W", 1.0),
    KeyDef::new("E", "E", 1.0),
    KeyDef::new("R", "R", 1.0),
    KeyDef::new("T", "T", 1.0),
    KeyDef::new("Y", "Y", 1.0),
    KeyDef::new("U", "U", 1.0),
    KeyDef::new("I", "I", 1.0),
    KeyDef::new("O", "O", 1.0),
    KeyDef::new("P", "P", 1.0),
    KeyDef::new("[", "bracketleft", 1.0),
    KeyDef::new("]", "bracketright", 1.0),
    KeyDef::new("\\\\", "backslash", 1.5),
];

const ROW_3: &[KeyDef] = &[
    KeyDef::new("Caps", "Caps_Lock", 1.75),
    KeyDef::new("A", "A", 1.0),
    KeyDef::new("S", "S", 1.0),
    KeyDef::new("D", "D", 1.0),
    KeyDef::new("F", "F", 1.0),
    KeyDef::new("G", "G", 1.0),
    KeyDef::new("H", "H", 1.0),
    KeyDef::new("J", "J", 1.0),
    KeyDef::new("K", "K", 1.0),
    KeyDef::new("L", "L", 1.0),
    KeyDef::new(";", "semicolon", 1.0),
    KeyDef::new("APOS", "apostrophe", 1.0),
    KeyDef::new("Enter", "Return", 2.25),
];

const ROW_4: &[KeyDef] = &[
    KeyDef::new("Shift", "Shift_L", 2.25),
    KeyDef::new("Z", "Z", 1.0),
    KeyDef::new("X", "X", 1.0),
    KeyDef::new("C", "C", 1.0),
    KeyDef::new("V", "V", 1.0),
    KeyDef::new("B", "B", 1.0),
    KeyDef::new("N", "N", 1.0),
    KeyDef::new("M", "M", 1.0),
    KeyDef::new(",", "comma", 1.0),
    KeyDef::new(".", "period", 1.0),
    KeyDef::new("/", "slash", 1.0),
    KeyDef::new("Shift", "Shift_R", 2.75),
];

const ROW_5: &[KeyDef] = &[
    KeyDef::new("Ctrl", "Control_L", 1.25),
    KeyDef::new("Sup", "Super_L", 1.25),
    KeyDef::new("Alt", "Alt_L", 1.25),
    KeyDef::new("Space", "space", 6.25),
    KeyDef::new("Alt", "Alt_R", 1.25),
    KeyDef::new("Sup", "Super_R", 1.25),
    KeyDef::new("Menu", "Menu", 1.25),
    KeyDef::new("Ctrl", "Control_R", 1.25),
];

const ROW_ARROWS: &[KeyDef] = &[
    KeyDef::new("<", "Left", 1.0),
    KeyDef::new("v", "Down", 1.0),
    KeyDef::new("^", "Up", 1.0),
    KeyDef::new(">", "Right", 1.0),
];

pub fn create_keyboard_view(stack: &gtk::Stack, model: &gio::ListStore) -> gtk::Box {
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

    let stack_clone = stack.clone();
    back_btn.connect_clicked(move |_| {
        stack_clone.set_visible_child_name("home");
    });

    let title = gtk::Label::builder()
        .label("Visual Keyboard Map")
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

    // Default to SUPER enabled as it's most common
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
    add_row(ROW_1, row_idx, &grid);
    row_idx += 1;
    add_row(ROW_2, row_idx, &grid);
    row_idx += 1;
    add_row(ROW_3, row_idx, &grid);
    row_idx += 1;
    add_row(ROW_4, row_idx, &grid);
    row_idx += 1;
    add_row(ROW_5, row_idx, &grid);
    row_idx += 1;

    // Arrow keys
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
