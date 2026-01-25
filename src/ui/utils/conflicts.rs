use crate::parser;
use crate::ui::utils::components::{create_destructive_button, create_pill_button};
use crate::ui::utils::keybinds::normalize;
use gtk::prelude::*;
use gtk4 as gtk;
use std::collections::HashMap;

pub struct ConflictInfo {
    pub dispatcher: String,
    pub args: String,
    pub file: String,
    pub line: usize,
}

pub struct ConflictPanel {
    pub container: gtk::Box,
    pub target_label: gtk::Label,
    pub suggestions_box: gtk::Box,
    pub back_btn: gtk::Button,
    pub proceed_btn: gtk::Button,
}

pub fn create_conflict_panel(proceed_label: &str) -> ConflictPanel {
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(24)
        .valign(gtk::Align::Center)
        .halign(gtk::Align::Center)
        .build();

    let icon = gtk::Image::builder()
        .icon_name("dialog-warning-symbolic")
        .visible(true)
        .pixel_size(48)
        .css_classes(["error-icon"])
        .tooltip_text("Conflicting keybind")
        .build();
    container.append(&icon);

    let title = gtk::Label::builder()
        .label("Keybind Conflict")
        .css_classes(["title-2"])
        .build();
    container.append(&title);

    let details = gtk::Label::builder()
        .label("This keybind is already in use by:")
        .justify(gtk::Justification::Center)
        .build();
    container.append(&details);

    let target_label = gtk::Label::builder()
        .label("...")
        .css_classes(["accent", "heading"])
        .wrap(true)
        .max_width_chars(40)
        .justify(gtk::Justification::Center)
        .build();
    container.append(&target_label);

    let suggestion_label = gtk::Label::builder()
        .label("Suggested Alternatives:")
        .margin_top(12)
        .build();
    container.append(&suggestion_label);

    let suggestions_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Center)
        .build();
    container.append(&suggestions_box);

    let button_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Center)
        .margin_top(12)
        .build();

    let back_btn = create_pill_button("Back", None);
    let proceed_btn = create_destructive_button(proceed_label, None);

    button_box.append(&back_btn);
    button_box.append(&proceed_btn);
    container.append(&button_box);

    ConflictPanel {
        container,
        target_label,
        suggestions_box,
        back_btn,
        proceed_btn,
    }
}

fn resolve(input: &str, vars: &HashMap<String, String>) -> String {
    if !input.contains('$') {
        return input.to_string();
    }
    let mut result = input.to_string();
    let mut sorted_keys: Vec<_> = vars.keys().cloned().collect();
    sorted_keys.sort_by_key(|b| std::cmp::Reverse(b.len()));

    for key in sorted_keys {
        if result.contains(&key) {
            result = result.replace(&key, &vars[&key]);
        }
    }
    result
}

pub fn check_conflict(
    target_mods: &str,
    target_key: &str,
    target_submap: Option<&str>,
    ignore_line: Option<usize>,
) -> Option<ConflictInfo> {
    let keybinds = parser::parse_config().unwrap_or_default();
    let variables = parser::get_variables().unwrap_or_default();

    let resolved_mods = resolve(target_mods, &variables);
    let resolved_key = resolve(target_key, &variables);

    let (norm_mods, norm_key) = normalize(&resolved_mods, &resolved_key);
    let target_submap = target_submap.unwrap_or("").trim();

    for kb in keybinds {
        if let Some(ignored) = ignore_line {
            if kb.line_number == ignored {
                continue;
            }
        }

        let (kb_mods, kb_key) = normalize(&kb.clean_mods, &kb.key);
        let kb_submap = kb.submap.as_deref().unwrap_or("").trim();

        if norm_mods == kb_mods && norm_key == kb_key && target_submap == kb_submap {
            return Some(ConflictInfo {
                dispatcher: kb.dispatcher.to_string(),
                args: kb.args.to_string(),
                file: kb.file_path.to_string_lossy().to_string(),
                line: kb.line_number,
            });
        }
    }

    None
}

pub fn generate_suggestions(
    target_mods: &str,
    target_key: &str,
    target_submap: Option<&str>,
) -> Vec<(String, String)> {
    let mut suggestions = Vec::new();
    let keybinds = parser::parse_config().unwrap_or_default();
    let variables = parser::get_variables().unwrap_or_default();

    let target_submap = target_submap.unwrap_or("").trim();

    let resolved_mods = resolve(target_mods, &variables);
    let resolved_key = resolve(target_key, &variables);
    let (norm_mods, _norm_key) = normalize(&resolved_mods, &resolved_key);

    let potential_mods = ["SHIFT", "CTRL", "ALT", "SUPER"];

    let is_free = |mods: &str, key: &str| -> bool {
        let r_mods = resolve(mods, &variables);
        let r_key = resolve(key, &variables);
        let (n_mods, n_key) = normalize(&r_mods, &r_key);

        !keybinds.iter().any(|kb| {
            let (k_mods, k_key) = normalize(&kb.clean_mods, &kb.key);
            let k_submap = kb.submap.as_deref().unwrap_or("").trim();
            k_mods == n_mods && k_key == n_key && k_submap == target_submap
        })
    };

    for &pm in &potential_mods {
        if !norm_mods.contains(pm) {
            let new_mods = if target_mods.is_empty() {
                pm.to_string()
            } else {
                format!("{} {}", target_mods, pm)
            };

            if is_free(&new_mods, target_key) {
                suggestions.push((new_mods, target_key.to_string()));
            }
        }
    }

    // 2. Try replacing modifiers (if collision exists)
    for &pm in &potential_mods {
        if !norm_mods.contains(pm) {
            let simple_mod = pm.to_string();
            if is_free(&simple_mod, target_key) {
                suggestions.push((simple_mod, target_key.to_string()));
            }
        }
    }

    // Limit and deduplicate
    suggestions.truncate(3);
    suggestions
}
